// Saying what only a step knows: the shipping reports that the parcel left, as an event of its own
// in
// the journey's stream.

package ch09_saying_what_only_a_step_knows

import org.itineradev.itinera.event.AttemptStarted
import org.itineradev.itinera.event.ContributionCommitted
import org.itineradev.itinera.event.Event
import org.itineradev.itinera.event.JourneyFailed
import org.itineradev.itinera.event.JourneyStarted
import org.itineradev.itinera.event.ServiceProvided
import org.itineradev.itinera.event.StepFailed
import org.itineradev.itinera.event.StepGivenUp
import org.itineradev.itinera.event.StepInfo
import org.itineradev.itinera.event.StepSucceeded
import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.journey.Contributor
import org.itineradev.itinera.journey.kotlin.output
import org.itineradev.itinera.report.Reporter
import org.itineradev.itinera.service.Provided
import org.itineradev.itinera.service.Provider
import org.itineradev.itinera.service.Service
import org.itineradev.itinera.step.Input
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.step.StepReporter
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/** A payment client: it charges an order's amount, and gives back a receipt. */
interface Payments {
    fun charge(order: String, amount: Long): String
}

/**
 * A carrier: it books a parcel and gives back its tracking number, then says whether it collects a
 * parcel of this weight in kg.
 */
interface Carrier {
    fun book(order: String): String

    fun collects(weight: Long): Boolean
}

/** The shop's bank, faked: it prints what it charges, and gives a receipt for it. */
class Bank : Payments {
    override fun charge(order: String, amount: Long): String {
        println("the bank charges $amount for order $order")
        return "R-$order"
    }
}

/** The shop's carrier, faked: it books any parcel, and collects one only up to its weight limit. */
class MainCarrier(private val maxWeight: Long) : Carrier {
    override fun book(order: String): String {
        println("the carrier books order $order")
        return "T-$order"
    }

    override fun collects(weight: Long): Boolean = weight <= maxWeight
}

/** Charges an order's price through the payment client, and gives back its receipt. */
@Step
class Charge(@Input private val order: String, @Input private val price: Long) : ItineraStep {
    fun run(@Service payments: Payments, contributor: Contributor): Outcome {
        if (price <= 0) return fail("nothing-to-charge")
        val receipt = payments.charge(order, price)
        contributor.contribute("receipt", receipt)
        return succeed()
    }
}

// ANCHOR: ship
/**
 * Ships an order through the carrier, with its receipt in the parcel, gives back its tracking
 * number, and says when the parcel has left.
 */
@Step
class Ship(
    @Input private val order: String,
    @Input private val weight: Long,
    @Input private val receipt: String,
) : ItineraStep {
    fun run(@Service carrier: Carrier, contributor: Contributor, reporter: StepReporter): Outcome {
        val tracking = carrier.book(order)
        contributor.contribute("tracking", tracking)
        if (!carrier.collects(weight)) return fail("too-heavy")
        reporter.info("the parcel leaves with receipt $receipt")
        return succeed()
    }
}

// ANCHOR_END: ship

/** The shop's log: it writes a line for every event of a journey. */
class Log : Reporter {
    override fun report(event: Event) {
        when (val detail = detail(event)) {
            null -> println("log #${event.sequence} ${event.kind}")
            else -> println("log #${event.sequence} ${event.kind} $detail")
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
        is AttemptStarted -> "${event.step}"
        is ServiceProvided ->
            "${event.step} gets ${event.key} from ${event.provider}, a ${event.implementation}"
        is StepSucceeded -> "${event.step}"
        is StepFailed -> "${event.step}: ${event.reason.code}"
        is ContributionCommitted -> event.key
        is StepGivenUp -> "${event.step}"
        is JourneyFailed -> "${event.step}"
        // ANCHOR: step_info
        is StepInfo -> "${event.step}: ${event.message}"
        // ANCHOR_END: step_info
        else -> null
    }

/** The shop's workflow: it charges, then ships, and every journey is logged. */
@Workflow(name = "orders", reporters = [Log::class])
@WithStep(Charge::class, name = "charge")
@WithStep(Ship::class, name = "ship")
class Orders : ItineraWorkflow {
    /** The bank, as the payment client of the journey. */
    @Provider(key = "payments") fun bank(): Provided<Payments> = provide(Bank())

    /** The main carrier, as the carrier of the journey. */
    @Provider(key = "carrier")
    fun mainCarrier(): Provided<Carrier> = provide(MainCarrier(maxWeight = 20))
}

/** What the shop knows about an order when its journey starts. */
data class NewOrder(val order: String, val price: Long, val weight: Long)

/** What the shop reads back once the journey has ended. */
data class Order(val receipt: String, val tracking: String?)

fun main() {
    // The customer, and their order: its number, its price and the parcel's weight in kg.
    val orders =
        listOf(
            "Ann" to NewOrder("A-1", price = 40, weight = 2),
            "Gil" to NewOrder("A-7", price = 45, weight = 60),
        )
    for ((customer, order) in orders) {
        println("$customer's order:")
        val instance = Orders().instance().initialData(order).create()
        val result = LocalExecutor().run(instance)
        val data = result.output<Order>()
        val ended = if (result.isSucceeded) "succeeded" else "failed"
        println(
            "the journey $ended, with receipt ${data.receipt} and tracking ${data.tracking ?: "none"}"
        )
    }
}
