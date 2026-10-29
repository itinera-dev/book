// The shop's workflow, whole: three steps, a rescue step for the charge, a retry for shipping with
// the backup carrier on the second attempt, the audit and notify policies, and the services the
// steps use.

package ch02_big_picture

import org.itineradev.itinera.event.Event
import org.itineradev.itinera.event.JourneyInfo
import org.itineradev.itinera.event.StepFailed
import org.itineradev.itinera.event.StepGivenUp
import org.itineradev.itinera.event.StepHookSource
import org.itineradev.itinera.event.StepRescuing
import org.itineradev.itinera.event.StepRetrying
import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.journey.Contributor
import org.itineradev.itinera.journey.Failure
import org.itineradev.itinera.journey.JourneyStatus
import org.itineradev.itinera.policy.FromStep
import org.itineradev.itinera.policy.FromWorkflow
import org.itineradev.itinera.policy.HookReporter
import org.itineradev.itinera.policy.ItineraStepPolicy
import org.itineradev.itinera.policy.ItineraWorkflowPolicy
import org.itineradev.itinera.policy.OnStepSuccess
import org.itineradev.itinera.policy.OnSuccess
import org.itineradev.itinera.policy.OnWorkflowFailure
import org.itineradev.itinera.policy.OnWorkflowSuccess
import org.itineradev.itinera.policy.Role
import org.itineradev.itinera.policy.StepPolicy
import org.itineradev.itinera.policy.WorkflowPolicy
import org.itineradev.itinera.report.Reporter
import org.itineradev.itinera.service.Provided
import org.itineradev.itinera.service.Provider
import org.itineradev.itinera.service.Service
import org.itineradev.itinera.step.Attempt
import org.itineradev.itinera.step.Input
import org.itineradev.itinera.step.ItineraRescueStep
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Reason
import org.itineradev.itinera.step.RescueOutcome
import org.itineradev.itinera.step.RescueStep
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.ListedService
import org.itineradev.itinera.workflow.ListedStep
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow
import org.itineradev.itinera.workflow.WorkflowDescriptor

// ANCHOR: workflow
/**
 * The shop's workflow: reserve, charge and ship, auditing the charge, rescuing it when the card has
 * expired, and telling the customer how the order went.
 */
@Workflow(name = "orders", policies = [Notify::class], reporters = [Log::class])
// The steps run in this order, and nothing reorders them.
@WithStep(Reserve::class, name = "reserve")
// The charge runs at most twice: once, and once more after its rescue step succeeds.
// Every successful charge is audited.
@WithStep(Charge::class, name = "charge", rescue = RenewCard::class, policies = [Audit::class])
// Shipping may be tried once more after a failure, which, for a parcel the carrier refused, is
// worth trying again.
@WithStep(Ship::class, name = "ship", retryBudget = 1)
class Orders : ItineraWorkflow, Messages {
    /** One payment client for the journey, used by the charge and its rescue step. */
    @Provider(key = "payments") fun bank(): Provided<Payments> = provide(BANK)

    /** One stock store for the journey, shared by reserve and ship. */
    @Provider(key = "stock") fun warehouse(): Provided<StockStore> = provide(WAREHOUSE)

    /** A carrier for every attempt: the main carrier first, the backup carrier after. */
    @Provider(perAttempt = true)
    fun carrier(attempt: Attempt): Provided<Carrier> =
        provide(if (attempt.isFirst) MAIN_CARRIER else BACKUP_CARRIER)

    /** How the shop tells a customer something, for the notify policy: here, by printing it. */
    override fun tell(customer: String, message: String) {
        println("  to $customer: $message")
    }
}

// ANCHOR_END: workflow

// ANCHOR: services
/** What the bank answers to a charge: the receipt, or why it refused. */
sealed interface Answer {
    data class Paid(val receipt: String) : Answer

    data object ExpiredCard : Answer

    data object Refused : Answer
}

/** A payment client: it charges a card, and knows the card that replaced an expired one. */
interface Payments {
    fun charge(card: String, amount: Long): Answer

    fun renewedCard(card: String): String
}

/** A stock store: it reserves an order's goods, then hands them over for shipping. */
interface StockStore {
    fun reserve(order: String): Boolean

    fun handOver(order: String)
}

/** A carrier: it takes a parcel and gives back its tracking number, or `null` when it cannot. */
interface Carrier {
    fun send(order: String, weight: Long): String?
}

// ANCHOR_END: services

/**
 * The customers' bank, faked. A card ends with the year it expires, such as `ann-card-2028`; the
 * bank refuses a card that expired before this year, and any charge above its limit.
 */
class Bank(private val thisYear: Int, private val limit: Long) : Payments {
    override fun charge(card: String, amount: Long): Answer =
        when {
            card.substringAfterLast('-').toInt() < thisYear -> Answer.ExpiredCard
            amount > limit -> Answer.Refused
            else -> Answer.Paid("R-$card-$amount")
        }

    override fun renewedCard(card: String): String =
        "${card.substringBeforeLast('-')}-${thisYear + 3}"
}

/** The warehouse, faked: it lists the orders it has no stock for. */
class Warehouse(private val outOfStock: List<String>) : StockStore {
    override fun reserve(order: String): Boolean = order !in outOfStock

    override fun handOver(order: String) {}
}

/** A carrier, faked: it takes parcels up to a weight. */
class Van(private val trackingPrefix: String, private val maxWeight: Long) : Carrier {
    override fun send(order: String, weight: Long): String? =
        if (weight > maxWeight) null else "$trackingPrefix-$order"
}

val BANK = Bank(thisYear = 2026, limit = 1000)
val WAREHOUSE = Warehouse(outOfStock = listOf("A-5"))
val MAIN_CARRIER = Van(trackingPrefix = "T", maxWeight = 20)
val BACKUP_CARRIER = Van(trackingPrefix = "B", maxWeight = 50)

// ANCHOR: steps
/** Reserves the order's goods in the stock store. */
@Step
class Reserve(@Input private val order: String) : ItineraStep {
    fun run(@Service stock: StockStore): Outcome {
        if (!stock.reserve(order)) return fail("out-of-stock")
        return succeed()
    }
}

/**
 * Charges the card the price, less any discount. It only says how it went: what happens after a
 * declined card is not its business.
 */
@Step
class Charge(
    @Input private val card: String,
    @Input private val price: Long,
    @Input private val discount: Long?,
) : ItineraStep {
    fun run(@Service payments: Payments, contributor: Contributor): Outcome {
        val amount = price - (discount ?: 0)
        return when (val answer = payments.charge(card, amount)) {
            is Answer.Paid -> {
                contributor.contribute("charged", amount)
                contributor.contribute("receipt", answer.receipt)
                succeed()
            }
            Answer.ExpiredCard -> fail("expired-card")
            Answer.Refused -> fail("declined")
        }
    }
}

/**
 * The charge's rescue step: asks the customer's bank for the card that replaced an expired one. A
 * charge that failed for any other reason, or ended in an abnormal termination, which has no
 * reason, cannot be repaired this way.
 */
@RescueStep
class RenewCard(@Input private val card: String, private val reason: Reason?) : ItineraRescueStep {
    fun run(@Service payments: Payments, contributor: Contributor): RescueOutcome {
        if (reason?.code != "expired-card") return fail("nothing-to-renew")
        contributor.contribute("card", payments.renewedCard(card))
        return succeed()
    }
}

/**
 * Ships the order through the carrier it is given, without knowing which one it is. A carrier that
 * cannot take the parcel might be replaced on another try, so that failure is worth trying again.
 */
@Step
class Ship(@Input private val order: String, @Input private val weight: Long) : ItineraStep {
    fun run(
        @Service carrier: Carrier,
        @Service stock: StockStore,
        contributor: Contributor,
    ): Outcome {
        val tracking = carrier.send(order, weight) ?: return fail("parcel-refused")
        stock.handOver(order)
        contributor.contribute("tracking", tracking)
        return succeed()
    }
}

// ANCHOR_END: steps

// ANCHOR: policies
/** Audits every successful charge, in the journey's events, including one after a rescue. */
@StepPolicy(name = "audit")
class Audit : ItineraStepPolicy {
    @OnStepSuccess
    fun audited(
        @FromStep charged: Long,
        @FromStep receipt: String,
        reporter: HookReporter,
    ): OnSuccess {
        reporter.info("charged $charged, receipt $receipt")
        return keepDefault()
    }
}

/** A role the shop provides: telling a customer something. */
@Role
interface Messages {
    fun tell(customer: String, message: String)
}

/** Tells the customer how the order went, once, when the journey ends. */
@WorkflowPolicy(name = "notify")
class Notify : ItineraWorkflowPolicy {
    @OnWorkflowSuccess
    fun shipped(
        @FromWorkflow customer: String,
        @FromWorkflow order: String,
        @FromWorkflow tracking: String,
        @Role messages: Messages,
    ) {
        messages.tell(customer, "order $order is on its way: $tracking")
    }

    @OnWorkflowFailure
    fun failed(
        @FromWorkflow customer: String,
        @FromWorkflow order: String,
        @Role messages: Messages,
    ) {
        messages.tell(customer, "order $order could not be completed")
    }
}

// ANCHOR_END: policies

/**
 * The shop's log: every failure, every decision and what the hooks report, for every journey, in
 * one format.
 */
class Log : Reporter {
    override fun report(event: Event) {
        when (event) {
            is StepFailed ->
                println("  WARN ${event.step}, attempt ${event.attempt}: ${event.reason.code}")
            is StepRetrying -> println("  INFO ${event.step} tried again")
            is StepRescuing -> println("  INFO ${event.step} to be rescued by ${event.rescue}")
            is StepGivenUp -> println("  WARN ${event.step} given up: ${event.cause.cause}")
            is JourneyInfo -> {
                val hook = event.source
                if (hook is StepHookSource) println("  INFO ${hook.policy}: ${event.message}")
            }
            else -> {}
        }
    }
}

/** What the shop knows about an order when the journey starts. */
data class NewOrder(
    val order: String,
    val customer: String,
    val card: String,
    val price: Long,
    val discount: Long? = null,
    val weight: Long,
)

/** A service in the listing: its key, and the provider that serves it. */
fun served(listed: ListedService): String = "${listed.key} from ${listed.provider}"

/** The services of a listing's entry, separated by commas. */
fun services(listed: List<ListedService>): String = listed.joinToString(transform = ::served)

/**
 * One entry of the listing: a step's position and name, its policies, its services with the
 * provider that serves each, and its rescue step with its services.
 */
fun entry(listed: ListedStep): String = buildString {
    append("${listed.position}. ${listed.step}")
    if (listed.policies.isNotEmpty()) append(", with ${listed.policies.joinToString()}")
    append("; services: ${services(listed.services)}")
    listed.rescue?.let { append("; rescued by ${it.step}, services: ${services(it.services)}") }
}

/** Why a journey failed: its cause, and the reason a step gave when it gave one. */
fun why(failure: Failure): String =
    failure.reason?.let { "${failure.cause}, ${it.code}" } ?: failure.cause.toString()

fun main() {
    // ANCHOR: list
    // The workflow lists its steps, their policies, services and rescue step, without running
    // anything.
    for (listed in WorkflowDescriptor.of(Orders::class.java).listing()) {
        println(entry(listed))
    }
    // ANCHOR_END: list
    // ANCHOR: orders
    // The order, the customer, the card, the price, the discount when there is one, and the
    // parcel's weight in kg.
    val orders =
        listOf(
            NewOrder("A-1", "Ann", "ann-card-2028", price = 40, discount = 5, weight = 2),
            NewOrder("A-2", "Ben", "ben-card-2024", price = 25, weight = 1),
            NewOrder("A-3", "Cat", "cat-card-2027", price = 30, weight = 30),
            NewOrder("A-4", "Dan", "dan-card-2025", price = 1200, weight = 3),
            NewOrder("A-5", "Eve", "eve-card-2027", price = 20, weight = 2),
            NewOrder("A-6", "Fin", "fin-card-2028", price = 1500, weight = 4),
            NewOrder("A-7", "Gil", "gil-card-2029", price = 45, weight = 60),
        )
    // One executor runs every journey, one after another, each an instance of the workflow made
    // with the order's data.
    val executor = LocalExecutor()
    for (order in orders) {
        println("${order.order}:")
        val result = executor.run(Orders().instance().initialData(order).create())
        when (val status = result.status) {
            is JourneyStatus.Succeeded -> println("succeeded")
            is JourneyStatus.Failed -> println("failed: ${why(status.failure)}")
            is JourneyStatus.Aborted -> println("aborted")
        }
    }
    // ANCHOR_END: orders
}
