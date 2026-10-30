// Testing a hook or an adapter alone: the shop of chapter 26, whose tests, beside it, call its
// policies' hooks and its workflow's input adapter as methods.

package ch28_testing_a_hook_or_an_adapter_alone_async

import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.yield
import org.itineradev.itinera.event.AttemptStarted
import org.itineradev.itinera.event.ContributionCommitted
import org.itineradev.itinera.event.ContributionsDiscarded
import org.itineradev.itinera.event.DataOverwritten
import org.itineradev.itinera.event.DecidedBy
import org.itineradev.itinera.event.DecidingHook
import org.itineradev.itinera.event.Event
import org.itineradev.itinera.event.GiveUpCause
import org.itineradev.itinera.event.InputAdapterSupplied
import org.itineradev.itinera.event.JourneyFailed
import org.itineradev.itinera.event.JourneyStarted
import org.itineradev.itinera.event.JourneySucceeded
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
import org.itineradev.itinera.executor.async.AsyncLocalExecutor
import org.itineradev.itinera.journey.Contributor
import org.itineradev.itinera.journey.DataBag
import org.itineradev.itinera.journey.FailureCause
import org.itineradev.itinera.journey.JourneyResult
import org.itineradev.itinera.journey.kotlin.read
import org.itineradev.itinera.policy.FromStep
import org.itineradev.itinera.policy.FromWorkflow
import org.itineradev.itinera.policy.ItineraStepPolicy
import org.itineradev.itinera.policy.ItineraWorkflowPolicy
import org.itineradev.itinera.policy.OnFailure
import org.itineradev.itinera.policy.OnStepAbnormalTermination
import org.itineradev.itinera.policy.OnStepFailure
import org.itineradev.itinera.policy.OnStepRetry
import org.itineradev.itinera.policy.OnStepSuccess
import org.itineradev.itinera.policy.OnSuccess
import org.itineradev.itinera.policy.OnWorkflowFailure
import org.itineradev.itinera.policy.OnWorkflowSuccess
import org.itineradev.itinera.policy.Role
import org.itineradev.itinera.policy.StepPolicy
import org.itineradev.itinera.policy.WorkflowPolicy
import org.itineradev.itinera.report.async.AsyncReporter
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
import org.itineradev.itinera.step.async.AsyncStepReporter
import org.itineradev.itinera.workflow.IdGenerator
import org.itineradev.itinera.workflow.InputAdapter
import org.itineradev.itinera.workflow.Key
import org.itineradev.itinera.workflow.Supplied
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow
import org.itineradev.itinera.workflow.async.ItineraAsyncWorkflow

/**
 * A payment client: it charges an order's amount to a card, and gives back a receipt, or `null`
 * when the card has expired. It throws [NetworkDown] when its network fails. And it says which card
 * replaced one, or `null` when none did. Each answer comes over the network.
 */
interface Payments {
    suspend fun charge(order: String, card: String, amount: Long): String?

    suspend fun replacement(card: String): String?
}

/**
 * A carrier: it books a parcel of this weight in kg and gives back its tracking number, or throws
 * [BookingFailed] when its booking system fails, then says whether it collects the parcel. Each
 * answer comes over the network.
 */
interface Carrier {
    suspend fun book(order: String, weight: Long): String

    suspend fun collects(weight: Long): Boolean
}

/** The shop's order book: it says whether an order was already completed. */
interface OrderBook {
    fun completed(order: String): Boolean
}

/** The error of the bank's network when it fails. */
class NetworkDown : Exception("the payment network is down")

/** The error of the carrier's booking system when it fails. */
class BookingFailed : Exception("the booking system failed")

/**
 * The network the fakes answer over: it hands the thread back once before the answer comes, as a
 * real call does while it waits.
 */
suspend fun network() {
    yield()
}

/**
 * The shop's bank, faked: a card is its holder's name and the year it expires, and the bank
 * declines a card that expired before this year. Its network fails on any amount over 100. It
 * prints what it does once it has answered.
 */
class Bank(private val thisYear: Int) : Payments {
    override suspend fun charge(order: String, card: String, amount: Long): String? {
        network()
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
    override suspend fun replacement(card: String): String? {
        network()
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
 * The shop's main carrier, faked: its booking system fails on a parcel that weighs nothing, and it
 * collects a parcel only up to 20 kg.
 */
class MainCarrier : Carrier {
    override suspend fun book(order: String, weight: Long): String {
        network()
        if (weight == 0L) {
            println("the main carrier's booking system fails on order $order")
            throw BookingFailed()
        }
        println("the main carrier books order $order")
        return "T-$order"
    }

    override suspend fun collects(weight: Long): Boolean {
        network()
        return weight <= 20
    }
}

/**
 * The shop's backup carrier, faked: its booking system fails on a parcel that weighs nothing, and
 * it collects a parcel only up to 80 kg.
 */
class BackupCarrier : Carrier {
    override suspend fun book(order: String, weight: Long): String {
        network()
        if (weight == 0L) {
            println("the backup carrier's booking system fails on order $order")
            throw BookingFailed()
        }
        println("the backup carrier books order $order")
        return "B-$order"
    }

    override suspend fun collects(weight: Long): Boolean {
        network()
        return weight <= 80
    }
}

/**
 * The shop's order book, faked: it holds the orders completed before today, and prints what it
 * finds.
 */
class Ledger(private val done: List<String>) : OrderBook {
    override fun completed(order: String): Boolean {
        if (order in done) {
            println("the order book has order $order completed already")
            return true
        }
        println("the order book has no order $order yet")
        return false
    }
}

/** Looks the order up in the order book, and gives back whether it was already completed. */
@Step
class Lookup(@Input private val order: String) : ItineraStep {
    suspend fun run(@Service orderBook: OrderBook, contributor: Contributor): Outcome {
        contributor.contribute("completed", orderBook.completed(order))
        return succeed()
    }
}

/**
 * Finishes the journey of an order that was already completed: there is nothing left to do for it.
 */
@StepPolicy(name = "alreadyCompleted")
class AlreadyCompleted : ItineraStepPolicy {
    @OnStepSuccess
    suspend fun lookedUp(@FromStep completed: Boolean): OnSuccess =
        if (completed) finishWorkflow() else keepDefault()
}

/**
 * Works out what an order costs: its price, less its discount if it has one, given back as the
 * amount to charge.
 */
@Step
class Price(@Input private val price: Long, @Input private val discount: Long?) : ItineraStep {
    suspend fun run(contributor: Contributor): Outcome {
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
    suspend fun run(@Service payments: Payments, contributor: Contributor): Outcome {
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
    suspend fun run(@Service payments: Payments, contributor: Contributor): RescueOutcome {
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
    suspend fun charged(
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
    suspend fun tell(customer: String, message: String)
}

/** What the shop does for its policies: give an order's money back. */
@Role
interface Refunds {
    suspend fun refund(receipt: String, amount: Long)
}

/** Tells the customer how their order went, once its journey has ended. */
@WorkflowPolicy(name = "notify")
class Notify : ItineraWorkflowPolicy {
    @OnWorkflowSuccess
    suspend fun succeeded(
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
    suspend fun failed(
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
    @OnStepAbnormalTermination
    suspend fun endedInError(): OnFailure = failWorkflow("payment-network-down")
}

/**
 * Deals with a shipping that fails for good. It gives up a parcel over the shop's limit rather than
 * try it again, since the limit is the same on every attempt, whichever carrier comes. And whatever
 * gave the shipping up, it refunds the order's charge.
 */
@StepPolicy(name = "failedShipping")
class FailedShipping : ItineraStepPolicy {
    @OnStepRetry
    suspend fun beforeRetrying(
        reason: Reason?,
        @FromWorkflow receipt: String,
        @FromWorkflow amount: Long,
        @Role refunds: Refunds,
        contributor: Contributor,
    ): OnFailure {
        if (reason?.code != "over-parcel-limit") return keepDefault()
        refund(refunds, contributor, receipt, amount, "over-parcel-limit")
        return failWorkflow("no-carrier-can-take-it")
    }

    @OnStepFailure
    suspend fun givenUp(
        cause: FailureCause,
        @FromWorkflow receipt: String,
        @FromWorkflow amount: Long,
        @Role refunds: Refunds,
        contributor: Contributor,
    ): OnFailure {
        refund(refunds, contributor, receipt, amount, "$cause")
        return keepDefault()
    }

    /** Gives the order's money back, and records why. */
    private suspend fun refund(
        refunds: Refunds,
        contributor: Contributor,
        receipt: String,
        amount: Long,
        why: String,
    ) {
        refunds.refund(receipt, amount)
        contributor.contribute("refund", "$amount back on $receipt: $why")
    }
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
    suspend fun run(
        @Service carrier: Carrier,
        contributor: Contributor,
        reporter: AsyncStepReporter,
    ): Outcome {
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
class Log : AsyncReporter {
    override suspend fun report(event: Event) {
        val line = "log ${event.workflow} ${event.journeyId} #${event.sequence} ${event.kind}"
        when (val detail = detail(event)) {
            null -> println(line)
            else -> println("$line $detail")
        }
    }
}

/** The customer service desk: of all the events of a journey, it keeps only a failed journey's. */
class Desk : AsyncReporter {
    override suspend fun report(event: Event) {
        if (event is JourneyFailed)
            println("desk: order ${event.journeyId} failed at ${event.step}")
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
        is InputAdapterSupplied -> "${event.step} gets ${event.key} from ${event.adapter}"
        is JourneySucceeded ->
            when (val decidedBy = event.decidedBy) {
                is DecidingHook -> "FinishWorkflow from ${decidedBy.policy}"
                DecidedBy.Default -> null
            }
        else -> null
    }

/** What the log says about a step given up: which policy gave it up, and why, when one did. */
fun givenUp(event: StepGivenUp): String =
    when (val cause = event.cause) {
        is GiveUpCause.FailWorkflow ->
            "${event.step}: FailWorkflow from ${cause.decidedBy.policy}, ${cause.reason.code}"
        else -> "${event.step}: ${cause.cause}"
    }

/**
 * The shop's workflow: it looks the order up, and finishes an order already completed. Otherwise it
 * works out what the order costs, then charges it, repairing a charge that fails and auditing one
 * that succeeds, then ships, and shipping may be tried once more. Once a journey has ended, the
 * customer is told, through the channel the shop configures.
 */
@Workflow(name = "orders", reporters = [Log::class, Desk::class], policies = [Notify::class])
@WithStep(Lookup::class, name = "lookup", policies = [AlreadyCompleted::class])
@WithStep(Price::class, name = "price")
@WithStep(
    Charge::class,
    name = "charge",
    rescue = RenewCard::class,
    policies = [Audit::class, Outage::class],
)
@WithStep(Ship::class, name = "ship", retryBudget = 1, policies = [FailedShipping::class])
class Orders(private val channel: String) : ItineraAsyncWorkflow, Messages, Refunds {
    override suspend fun tell(customer: String, message: String) {
        network()
        println("$channel to $customer: $message")
    }

    override suspend fun refund(receipt: String, amount: Long) {
        network()
        println("the bank gives $amount back on receipt $receipt")
    }

    /** The bank, as the payment client of the journey. */
    @Provider(key = "payments")
    suspend fun bank(): Provided<Payments> = provide(Bank(thisYear = 2026))

    /** A carrier for every attempt of a step: the main carrier first, the backup carrier after. */
    @Provider(perAttempt = true, key = "carrier")
    suspend fun carriers(attempt: Attempt): Provided<Carrier> =
        if (attempt.isFirst) provide(MainCarrier()) else provide(BackupCarrier())

    /**
     * Gives the shipping the weight of the order's parcel, as the warehouse describes it. Every
     * other input of the shipping is read from the data bag.
     */
    @InputAdapter(steps = ["ship"])
    suspend fun parcels(@Key key: String, @FromWorkflow parcel: Parcel): Supplied =
        if (key == "weight") supply(parcel.weight) else fromDataBag()

    /** Names each journey after its order. */
    @IdGenerator
    fun orderNumber(data: DataBag): String =
        data.read<String>("order").orNull() ?: error("an order without a number")

    /** The order book, which has the orders completed before today. */
    @Provider(key = "orderBook")
    suspend fun ledger(): Provided<OrderBook> = provide(Ledger(listOf("A-13")))
}

/**
 * What the shop knows about an order when its journey starts. An order with no card leaves it out.
 */
data class NewOrder(
    val order: String,
    val customer: String,
    val price: Long,
    val discount: Long? = null,
    val parcel: Parcel,
    val card: String? = null,
    val giftCard: Boolean = false,
)

/**
 * How the warehouse describes an order's parcel: its weight in kg, and its length, width and height
 * in cm.
 */
data class Parcel(val weight: Long, val size: List<Long>)

/** Runs an order's journey with an executor of its own. */
suspend fun journey(order: NewOrder): JourneyResult {
    val instance = Orders(channel = "email").instance().initialData(order).create()
    return AsyncLocalExecutor().run(instance)
}

fun main() {
    // Each order: its number, its customer, its price, its discount if it has one, its parcel, as
    // its weight in kg and its size in cm, and the card.
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
        )
    runBlocking {
        val results = orders.map { order -> async { journey(order) } }.awaitAll()
        for (result in results) {
            val ended = if (result.isSucceeded) "succeeded" else "failed"
            println("order ${result.journeyId} $ended")
        }
    }
}
