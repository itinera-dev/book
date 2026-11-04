// A carrier for each attempt: the workflow's provider makes a carrier for every attempt of the
// shipping, the main carrier first and the backup carrier after.

package ch13_a_carrier_for_each_attempt

import org.itineradev.itinera.event.AttemptStarted
import org.itineradev.itinera.event.ContributionCommitted
import org.itineradev.itinera.event.Event
import org.itineradev.itinera.event.JourneyFailed
import org.itineradev.itinera.event.JourneyStarted
import org.itineradev.itinera.event.ServiceProvided
import org.itineradev.itinera.event.StepAbnormalTermination
import org.itineradev.itinera.event.StepFailed
import org.itineradev.itinera.event.StepGivenUp
import org.itineradev.itinera.event.StepInfo
import org.itineradev.itinera.event.StepRetrying
import org.itineradev.itinera.event.StepSucceeded
import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.journey.Contributor
import org.itineradev.itinera.journey.DataBag
import org.itineradev.itinera.journey.kotlin.output
import org.itineradev.itinera.journey.kotlin.read
import org.itineradev.itinera.report.Reporter
import org.itineradev.itinera.service.Provided
import org.itineradev.itinera.service.Provider
import org.itineradev.itinera.service.Service
import org.itineradev.itinera.step.Attempt
import org.itineradev.itinera.step.Input
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.step.StepReporter
import org.itineradev.itinera.workflow.IdGenerator
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/**
 * A payment client: it charges an order's amount to a card, and gives back a receipt, or `null`
 * when the card has expired. It throws [NetworkDown] when its network fails.
 */
interface Payments {
    fun charge(order: String, card: String, amount: Long): String?
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
}

/** The year a card expires: the part of it after its last `-`. */
fun expiry(card: String): Int = card.substringAfterLast('-').toInt()

// ANCHOR: carriers
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

// ANCHOR_END: carriers

/** Charges an order's price to its card through the payment client, and gives back its receipt. */
@Step
class Charge(
    @Input private val order: String,
    @Input private val price: Long,
    @Input private val card: String,
) : ItineraStep {
    fun run(@Service payments: Payments, contributor: Contributor): Outcome {
        if (price <= 0) return fail("nothing-to-charge")
        val receipt = payments.charge(order, card, price) ?: return fail("card-expired")
        contributor.contribute("receipt", receipt)
        return succeed()
    }
}

/**
 * Ships an order through the carrier, with its receipt in the parcel, gives back its tracking
 * number, and says when the parcel has left. A parcel the carrier won't collect may be taken on
 * another attempt.
 */
@Step
class Ship(
    @Input private val order: String,
    @Input private val weight: Long,
    @Input private val receipt: String,
) : ItineraStep {
    fun run(@Service carrier: Carrier, contributor: Contributor, reporter: StepReporter): Outcome {
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
        is StepGivenUp -> "${event.step}: ${event.cause.cause}"
        is ContributionCommitted -> event.key
        is JourneyFailed -> "${event.step}"
        is StepInfo -> "${event.step}: ${event.message}"
        else -> null
    }

/** The customer service desk: of all the events of a journey, it keeps only a failed journey's. */
class Desk : Reporter {
    override fun report(event: Event) {
        if (event is JourneyFailed)
            println("desk: order ${event.journeyId} failed at ${event.step}")
    }
}

/**
 * The shop's workflow: it charges, then ships, and each may be tried once more, except a charge
 * that ended in an error.
 */
@Workflow(name = "orders", reporters = [Log::class, Desk::class])
@WithStep(Charge::class, name = "charge", retryBudget = 1, noRetryOnAbnormalTermination = true)
@WithStep(Ship::class, name = "ship", retryBudget = 1)
class Orders : ItineraWorkflow {
    /** The bank, as the payment client of the journey. */
    @Provider(key = "payments") fun bank(): Provided<Payments> = provide(Bank(thisYear = 2026))

    // ANCHOR: provider
    /** A carrier for every attempt of a step: the main carrier first, the backup carrier after. */
    @Provider(perAttempt = true, key = "carrier")
    fun carriers(attempt: Attempt): Provided<Carrier> =
        if (attempt.isFirst) provide(MainCarrier()) else provide(BackupCarrier())

    // ANCHOR_END: provider

    /** Names each journey after its order. */
    @IdGenerator
    fun orderNumber(data: DataBag): String =
        data.read<String>("order").orNull() ?: error("an order without a number")
}

/** What the shop knows about an order when its journey starts. */
data class NewOrder(val order: String, val price: Long, val weight: Long, val card: String)

/** What the shop reads back once the journey has ended. */
data class Order(val receipt: String?, val tracking: String?)

fun main() {
    // The customer, and their order: its number, its price, the parcel's weight in kg and the card.
    val orders =
        listOf(
            "Ann" to NewOrder("A-1", price = 40, weight = 2, card = "ann-2028"),
            "Gil" to NewOrder("A-7", price = 45, weight = 60, card = "gil-2027"),
            "Jo" to NewOrder("A-8", price = 30, weight = 1, card = "jo-2024"),
        )
    for ((customer, order) in orders) {
        println("$customer's order:")
        val instance = Orders().instance().initialData(order).create()
        val result = LocalExecutor().run(instance)
        val data = result.output<Order>()
        val ended = if (result.isSucceeded) "succeeded" else "failed"
        val receipt = data.receipt ?: "none"
        val tracking = data.tracking ?: "none"
        println("the journey $ended, with receipt $receipt and tracking $tracking")
    }
}
