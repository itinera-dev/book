// A first journey: the shop runs the charge for two orders, and the result says how each journey
// ended.

package ch05_first_journey

import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.step.Input
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/** The shop's bank, faked: it prints what it charges. */
fun bankCharges(amount: Long) {
    println("the bank charges $amount")
}

/** Charges an order's price. */
@Step
class Charge(@Input private val price: Long) : ItineraStep {
    fun run(): Outcome {
        if (price <= 0) return fail("nothing-to-charge")
        bankCharges(price)
        return succeed()
    }
}

// ANCHOR: workflow
/** The shop's workflow: for now, it only charges. */
@Workflow(name = "orders") @WithStep(Charge::class, name = "charge") class Orders : ItineraWorkflow

// ANCHOR_END: workflow

// ANCHOR: run
fun main() {
    // The customer, and the price of their order.
    for ((customer, price) in listOf("Ann" to 40L, "Hal" to 0L)) {
        println("$customer's order:")
        val instance = Orders().instance().data("price", price).create()
        val result = LocalExecutor().run(instance)
        when {
            result.isSucceeded -> println("the journey succeeded")
            result.isFailed -> println("the journey failed")
        }
    }
}
// ANCHOR_END: run
