// Deciding where a step's data comes from: the warehouse describes each parcel as one value, and
// the workflow's input adapter gives the shipping only the weight it asks for.

package ch21_deciding_where_a_steps_data_comes_from

import org.itineradev.itinera.event.AttemptStarted
import org.itineradev.itinera.event.ContributionCommitted
import org.itineradev.itinera.event.ContributionsDiscarded
import org.itineradev.itinera.event.DataOverwritten
import org.itineradev.itinera.event.Event
import org.itineradev.itinera.event.GiveUpCause
import org.itineradev.itinera.event.InputAdapterSupplied
import org.itineradev.itinera.event.JourneyFailed
import org.itineradev.itinera.event.JourneyStarted
import org.itineradev.itinera.event.OptionalInputAbsent
import org.itineradev.itinera.event.ServiceProvided
import org.itineradev.itinera.event.StepAbnormalTermination
import org.itineradev.itinera.event.StepFailed
import org.itineradev.itinera.event.StepGivenUp
import org.itineradev.itinera.event.StepHookCalled
import org.itineradev.itinera.event.StepInfo
import org.itineradev.itinera.event.StepRescued
import org.itineradev.itinera.event.StepRescuing
import org.itineradev.itinera.event.StepRetrying
import org.itineradev.itinera.event.StepSkipped
import org.itineradev.itinera.event.StepSucceeded
import org.itineradev.itinera.event.WorkflowHookCalled
import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.journey.Contributor
import org.itineradev.itinera.journey.DataBag
import org.itineradev.itinera.journey.kotlin.output
import org.itineradev.itinera.journey.kotlin.read
import org.itineradev.itinera.policy.FromStep
import org.itineradev.itinera.policy.FromWorkflow
import org.itineradev.itinera.policy.ItineraStepPolicy
import org.itineradev.itinera.policy.ItineraWorkflowPolicy
import org.itineradev.itinera.policy.OnFailure
import org.itineradev.itinera.policy.OnStepAbnormalTermination
import org.itineradev.itinera.policy.OnStepRetry
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
import org.itineradev.itinera.step.StepName
import org.itineradev.itinera.step.StepReporter
import org.itineradev.itinera.workflow.IdGenerator
import org.itineradev.itinera.workflow.InputAdapter
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.Key
import org.itineradev.itinera.workflow.Supplied
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/**
 * A payment client: it charges an order's amount to a card, and gives back a receipt, or `null`
 * when the card has expired. It throws [NetworkDown] when its network fails. And it says which card
 * replaced one, or `null` when none did.
 */
interface Payments {
    fun charge(order: String, card: String, amount: Long): String?

    fun replacement(card: String): String?
}

/**
 * A carrier: it books a parcel of this weight in kg and gives back its tracking number, or throws
 * [BookingFailed] when its booking system fails, then says whether it collects the parcel.
 */
interface Carrier {
    fun book(order: String, weight: Long): String

    fun collects(weight: Long): Boolean
}

/** The error of the bank's network when it fails. */
class NetworkDown : Exception("the payment network is down")

/** The error of the carrier's booking system when it fails. */
class BookingFailed : Exception("the booking system failed")

/**
 * The shop's bank, faked: a card is its holder's name and the year it expires, and the bank
 * declines a card that expired before this year. Its network fails on any amount over 100. It
 * prints what it does.
 */
class Bank(private val thisYear: Int) : Payments {
    override fun charge(order: String, card: String, amount: Long): String? {
        if (amount > 100) {
            println("the bank's network fails while charging order $order")
            throw NetworkDown()
        }
        if (expiry(card) < thisYear) {
            println("the bank declines card $card: it has expired")
            return null
        }
        println("the bank charges $amount for order $order")
        return "R-$order"
    }

    /**
     * A card that expired in the last three years was replaced by its holder's card that expires in
     * three years. An older one was not.
     */
    override fun replacement(card: String): String? {
        if (expiry(card) + 3 < thisYear) {
            println("the bank says nothing replaced card $card")
            return null
        }
        val replacement = "${card.substringBeforeLast('-')}-${thisYear + 3}"
        println("the bank says card $card was replaced by $replacement")
        return replacement
    }
}

/** The year a card expires: the part of it after its last `-`. */
fun expiry(card: String): Int = card.substringAfterLast('-').toInt()

/**
 * The shop's main carrier, faked: its booking system fails on a parcel over 100 kg, and it collects
 * a parcel only up to 20 kg.
 */
class MainCarrier : Carrier {
    override fun book(order: String, weight: Long): String {
        if (weight > 100) {
            println("the main carrier's booking system fails on order $order")
            throw BookingFailed()
        }
        println("the main carrier books order $order")
        return "T-$order"
    }

    override fun collects(weight: Long): Boolean = weight <= 20
}

/**
 * The shop's backup carrier, faked: its booking system fails on a parcel over 100 kg, and it
 * collects a parcel only up to 80 kg.
 */
class BackupCarrier : Carrier {
    override fun book(order: String, weight: Long): String {
        if (weight > 100) {
            println("the backup carrier's booking system fails on order $order")
            throw BookingFailed()
        }
        println("the backup carrier books order $order")
        return "B-$order"
    }

    override fun collects(weight: Long): Boolean = weight <= 80
}

/**
 * Works out what an order costs: its price, less its discount if it has one, given back as the
 * amount to charge.
 */
@Step
class Price(@Input private val price: Long, @Input private val discount: Long?) : ItineraStep {
    fun run(contributor: Contributor): Outcome {
        contributor.contribute("amount", price - (discount ?: 0L))
        return succeed()
    }
}

/**
 * Charges an order's amount to its card through the payment client, and gives back its receipt, and
 * the gift card's code when the order is a gift card.
 */
@Step
class Charge(
    @Input private val order: String,
    @Input private val amount: Long,
    @Input private val card: String,
    @Input private val giftCard: Boolean,
) : ItineraStep {
    fun run(@Service payments: Payments, contributor: Contributor): Outcome {
        if (amount <= 0) return fail("nothing-to-charge")
        val receipt = payments.charge(order, card, amount) ?: return fail("card-expired")
        contributor.contribute("receipt", receipt)
        if (giftCard) contributor.contribute("code", "G-$order")
        return succeed()
    }
}

/**
 * Repairs a charge that failed because its card had expired: it asks the bank for the card that
 * replaced it, and gives it back as the order's card.
 */
@RescueStep
class RenewCard(@Input private val card: String, private val reason: Reason) : ItineraRescueStep {
    fun run(@Service payments: Payments, contributor: Contributor): RescueOutcome {
        if (reason.code != "card-expired") return fail("not-repairable")
        val replacement = payments.replacement(card) ?: return fail("not-replaced")
        contributor.contribute("card", replacement)
        return succeed()
    }
}

/**
 * Writes an audit line for every step it is attached to that succeeds: the amount the order was
 * charged, and the receipt the step gave back.
 */
@StepPolicy(name = "audit")
class Audit : ItineraStepPolicy {
    @OnStepSuccess
    fun charged(
        step: StepName,
        @FromWorkflow amount: Long,
        @FromStep receipt: String,
        contributor: Contributor,
    ): OnSuccess {
        contributor.contribute("audit", "$step took $amount, receipt $receipt")
        return keepDefault()
    }
}

/** What the shop does for its policies: tell a customer something. */
@Role
interface Messages {
    fun tell(customer: String, message: String)
}

/** Tells the customer how their order went, once its journey has ended. */
@WorkflowPolicy(name = "notify")
class Notify : ItineraWorkflowPolicy {
    @OnWorkflowSuccess
    fun succeeded(
        @FromWorkflow customer: String,
        @FromWorkflow order: String,
        @FromWorkflow tracking: String?,
        @FromWorkflow code: String?,
        @Role messages: Messages,
    ) {
        val message =
            when {
                tracking != null -> "Order $order is on its way, tracking $tracking."
                code != null -> "Order $order is paid: your gift card's code is $code."
                else -> "Order $order is complete."
            }
        messages.tell(customer, message)
    }

    @OnWorkflowFailure
    fun failed(
        @FromWorkflow customer: String,
        @FromWorkflow order: String,
        @Role messages: Messages,
    ) {
        messages.tell(customer, "Sorry, order $order could not be completed.")
    }
}

/**
 * Gives up a step that ends in an error at once: the bank's network is down, and neither a retry
 * nor a rescue can mend that.
 */
@StepPolicy(name = "outage")
class Outage : ItineraStepPolicy {
    @OnStepAbnormalTermination fun endedInError(): OnFailure = failWorkflow("payment-network-down")
}

/**
 * Gives up the shipping of a parcel over the shop's limit, rather than try it again: the limit is
 * the same on every attempt, whichever carrier comes.
 */
@StepPolicy(name = "heavyParcels")
class HeavyParcels : ItineraStepPolicy {
    @OnStepRetry
    fun beforeRetrying(reason: Reason?): OnFailure =
        if (reason?.code == "over-parcel-limit") failWorkflow("no-carrier-can-take-it")
        else keepDefault()
}

/**
 * Ships an order through the carrier, with its receipt in the parcel, gives back its tracking
 * number, and says when the parcel has left. A parcel the carrier won't collect may be taken on
 * another attempt. A gift card has nothing to ship, and the shop ships no parcel over 80 kg.
 */
@Step
class Ship(
    @Input private val order: String,
    @Input private val weight: Long,
    @Input private val receipt: String,
    @Input private val giftCard: Boolean,
) : ItineraStep {
    fun run(@Service carrier: Carrier, contributor: Contributor, reporter: StepReporter): Outcome {
        if (giftCard) return skipBecause("nothing-to-ship")
        if (weight > 80) return fail("over-parcel-limit")
        val tracking = carrier.book(order, weight)
        contributor.contribute("tracking", tracking)
        if (!carrier.collects(weight)) return fail("too-heavy")
        reporter.info("the parcel leaves with receipt $receipt")
        return succeed()
    }
}

/**
 * The shop's log: it writes a line for every event of a journey, starting with the workflow's name
 * and the journey's ID.
 */
class Log : Reporter {
    override fun report(event: Event) {
        val line = "log ${event.workflow} ${event.journeyId} #${event.sequence} ${event.kind}"
        when (val detail = detail(event)) {
            null -> println(line)
            else -> println("$line $detail")
        }
    }
}

/**
 * What the log says after an event's kind, for the kinds it says more about: the step the event is
 * about, and what it names.
 */
fun detail(event: Event): String? =
    when (event) {
        is JourneyStarted -> event.initialKeys.joinToString(", ")
        is AttemptStarted -> "${event.step}, attempt ${event.attempt}"
        is ServiceProvided ->
            "${event.step} gets ${event.key} from ${event.provider}, a ${event.implementation}"
        is StepSucceeded -> "${event.step}"
        is StepFailed -> "${event.step}: ${event.reason.code}"
        is StepAbnormalTermination -> "${event.step}: ${event.errorMessage}"
        is StepRetrying -> "${event.step}"
        is StepGivenUp -> givenUp(event)
        is ContributionCommitted -> event.key
        is StepRescuing -> "${event.step}: ${event.cause}, rescued by ${event.rescue}"
        is StepRescued -> "${event.step} by ${event.rescue}"
        is DataOverwritten -> event.key
        is JourneyFailed -> "${event.step}"
        is StepInfo -> "${event.step}: ${event.message}"
        is StepSkipped -> "${event.step}: ${event.reason?.code ?: "no reason"}"
        is ContributionsDiscarded -> "${event.step}"
        is StepHookCalled -> "${event.source.policy}: ${event.source.hook} of ${event.source.step}"
        is WorkflowHookCalled -> "${event.source.policy}: ${event.source.hook}"
        is OptionalInputAbsent -> event.key
        // ANCHOR: supplied
        is InputAdapterSupplied -> "${event.step} gets ${event.key} from ${event.adapter}"
        // ANCHOR_END: supplied
        else -> null
    }

/** What the log says about a step given up: which policy gave it up, and why, when one did. */
fun givenUp(event: StepGivenUp): String =
    when (val cause = event.cause) {
        is GiveUpCause.FailWorkflow ->
            "${event.step}: FailWorkflow from ${cause.decidedBy.policy}, ${cause.reason.code}"
        else -> "${event.step}: ${cause.cause}"
    }

/** The customer service desk: of all the events of a journey, it keeps only a failed journey's. */
class Desk : Reporter {
    override fun report(event: Event) {
        if (event is JourneyFailed)
            println("desk: order ${event.journeyId} failed at ${event.step}")
    }
}

/**
 * The shop's workflow: it works out what the order costs, then charges it, repairing a charge that
 * fails and auditing one that succeeds, then ships, and shipping may be tried once more. Once a
 * journey has ended, the customer is told, through the channel the shop configures.
 */
@Workflow(name = "orders", reporters = [Log::class, Desk::class], policies = [Notify::class])
@WithStep(Price::class, name = "price")
@WithStep(
    Charge::class,
    name = "charge",
    rescue = RenewCard::class,
    policies = [Audit::class, Outage::class],
)
@WithStep(Ship::class, name = "ship", retryBudget = 1, policies = [HeavyParcels::class])
class Orders(private val channel: String) : ItineraWorkflow, Messages {
    override fun tell(customer: String, message: String) {
        println("$channel to $customer: $message")
    }

    /** The bank, as the payment client of the journey. */
    @Provider(key = "payments") fun bank(): Provided<Payments> = provide(Bank(thisYear = 2026))

    /** A carrier for every attempt of a step: the main carrier first, the backup carrier after. */
    @Provider(perAttempt = true, key = "carrier")
    fun carriers(attempt: Attempt): Provided<Carrier> =
        if (attempt.isFirst) provide(MainCarrier()) else provide(BackupCarrier())

    // ANCHOR: adapter
    /**
     * Gives the shipping the weight of the order's parcel, as the warehouse describes it. Every
     * other input of the shipping is read from the data bag.
     */
    @InputAdapter(steps = ["ship"])
    fun parcels(@Key key: String, @FromWorkflow parcel: Parcel): Supplied =
        if (key == "weight") supply(parcel.weight) else fromDataBag()

    // ANCHOR_END: adapter

    /** Names each journey after its order. */
    @IdGenerator
    fun orderNumber(data: DataBag): String =
        data.read<String>("order").orNull() ?: error("an order without a number")
}

// ANCHOR: data
/** What the shop knows about an order when its journey starts. */
data class NewOrder(
    val order: String,
    val customer: String,
    val price: Long,
    val discount: Long? = null,
    val parcel: Parcel,
    val card: String,
    val giftCard: Boolean = false,
)

// ANCHOR_END: data

// ANCHOR: parcel
/**
 * How the warehouse describes an order's parcel: its weight in kg, and its length, width and height
 * in cm.
 */
data class Parcel(val weight: Long, val size: List<Long>)

// ANCHOR_END: parcel

/** What the shop reads back once the journey has ended. */
data class Order(val receipt: String?, val tracking: String?, val audit: String?)

fun main() {
    // Each order: its number, its customer, its price, its discount if it has one, its parcel, as
    // its weight in kg and its size in cm, the card, and whether it is a gift card.
    val orders =
        listOf(
            NewOrder(
                "A-1",
                "Ann",
                price = 40,
                discount = 10,
                parcel = Parcel(weight = 2, size = listOf(30, 20, 10)),
                card = "ann-2028",
            ),
            NewOrder(
                "A-7",
                "Gil",
                price = 45,
                parcel = Parcel(weight = 60, size = listOf(120, 80, 60)),
                card = "gil-2027",
            ),
            NewOrder(
                "A-8",
                "Jo",
                price = 30,
                parcel = Parcel(weight = 1, size = listOf(20, 15, 5)),
                card = "jo-2024",
            ),
            NewOrder(
                "A-9",
                "Kim",
                price = 25,
                parcel = Parcel(weight = 0, size = listOf(0, 0, 0)),
                card = "kim-2027",
                giftCard = true,
            ),
            NewOrder(
                "A-10",
                "Lee",
                price = 35,
                parcel = Parcel(weight = 3, size = listOf(40, 30, 20)),
                card = "lee-2019",
            ),
        )
    for (order in orders) {
        println("${order.customer}'s order:")
        val instance = Orders(channel = "email").instance().initialData(order).create()
        val result = LocalExecutor().run(instance)
        val data = result.output<Order>()
        val ended = if (result.isSucceeded) "succeeded" else "failed"
        val receipt = data.receipt ?: "none"
        val tracking = data.tracking ?: "none"
        println("the journey $ended, with receipt $receipt and tracking $tracking")
        println("audit: ${data.audit ?: "none"}")
    }
}
