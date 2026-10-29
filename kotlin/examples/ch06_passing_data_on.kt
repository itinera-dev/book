// Passing data on: the charge gives back a receipt that the shipping puts in the parcel, and a
// failed attempt's contributions never reach the data bag.

package ch06_passing_data_on

import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.journey.Contributor
import org.itineradev.itinera.journey.kotlin.output
import org.itineradev.itinera.step.Input
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/** The shop's bank, faked: it prints what it charges, and gives a receipt for it. */
fun bankCharges(order: String, amount: Long): String {
    println("the bank charges $amount for order $order")
    return "R-$order"
}

/** The shop's carrier, faked: it books any parcel, and gives its tracking number at once. */
fun carrierBooks(order: String): String {
    println("the carrier books order $order")
    return "T-$order"
}

/**
 * The carrier, faked: it weighs a parcel when it comes to collect it, and takes it only up to 20
 * kg.
 */
fun carrierCollects(weight: Long): Boolean = weight <= 20

// ANCHOR: charge
/** Charges an order's price, and gives back the bank's receipt. */
@Step
class Charge(@Input private val order: String, @Input private val price: Long) : ItineraStep {
    fun run(contributor: Contributor): Outcome {
        if (price <= 0) return fail("nothing-to-charge")
        val receipt = bankCharges(order, price)
        contributor.contribute("receipt", receipt)
        return succeed()
    }
}

// ANCHOR_END: charge

// ANCHOR: ship
/** Ships an order, with its receipt in the parcel, and gives back its tracking number. */
@Step
class Ship(
    @Input private val order: String,
    @Input private val weight: Long,
    @Input private val receipt: String,
) : ItineraStep {
    fun run(contributor: Contributor): Outcome {
        val tracking = carrierBooks(order)
        contributor.contribute("tracking", tracking)
        if (!carrierCollects(weight)) return fail("too-heavy")
        println("the parcel leaves with receipt $receipt")
        return succeed()
    }
}

// ANCHOR_END: ship

// ANCHOR: workflow
/** The shop's workflow: it charges, then ships. */
@Workflow(name = "orders")
@WithStep(Charge::class, name = "charge")
@WithStep(Ship::class, name = "ship")
class Orders : ItineraWorkflow

// ANCHOR_END: workflow

// ANCHOR: data
/** What the shop knows about an order when its journey starts. */
data class NewOrder(val order: String, val price: Long, val weight: Long)

// ANCHOR_END: data

// ANCHOR: output
/** What the shop reads back once the journey has ended. */
data class Order(val receipt: String, val tracking: String?)

// ANCHOR_END: output

// ANCHOR: run
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
// ANCHOR_END: run
