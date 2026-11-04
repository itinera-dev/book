// Giving a step the tools it needs: the workflow provides the bank and the carrier, and the steps
// use them without knowing which ones they got.

package ch07_services

import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.journey.Contributor
import org.itineradev.itinera.journey.kotlin.output
import org.itineradev.itinera.service.Provided
import org.itineradev.itinera.service.Provider
import org.itineradev.itinera.service.Service
import org.itineradev.itinera.step.Input
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

// ANCHOR: services
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

// ANCHOR_END: services

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

// ANCHOR: steps
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

/**
 * Ships an order through the carrier, with its receipt in the parcel, and gives back its tracking
 * number.
 */
@Step
class Ship(
    @Input private val order: String,
    @Input private val weight: Long,
    @Input private val receipt: String,
) : ItineraStep {
    fun run(@Service carrier: Carrier, contributor: Contributor): Outcome {
        val tracking = carrier.book(order)
        contributor.contribute("tracking", tracking)
        if (!carrier.collects(weight)) return fail("too-heavy")
        println("the parcel leaves with receipt $receipt")
        return succeed()
    }
}

// ANCHOR_END: steps

// ANCHOR: workflow
/** The shop's workflow: it charges, then ships, with the services each step needs. */
@Workflow(name = "orders")
@WithStep(Charge::class, name = "charge")
@WithStep(Ship::class, name = "ship")
class Orders : ItineraWorkflow {
    /** The bank, as the payment client of the journey. */
    @Provider(key = "payments")
    fun bank(): Provided<Payments> {
        println("the shop connects to the bank")
        return provide(Bank())
    }

    /** The main carrier, for the journey. */
    @Provider
    fun carrier(): Provided<Carrier> {
        println("the shop calls the carrier")
        return provide(MainCarrier(maxWeight = 20))
    }
}

// ANCHOR_END: workflow

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
