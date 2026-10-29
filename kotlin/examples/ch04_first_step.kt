// A first step: the shop's charge, declared as a step. Nothing builds or runs it yet, so the
// program
// prints nothing.

package ch04_first_step

import org.itineradev.itinera.step.Input
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step

/** The shop's bank, faked: it prints what it charges. */
fun bankCharges(amount: Long) {
    println("the bank charges $amount")
}

// ANCHOR: step
/** Charges an order's price. */
@Step
class Charge(@Input private val price: Long) : ItineraStep {
    fun run(): Outcome {
        if (price <= 0) return fail("nothing-to-charge")
        bankCharges(price)
        return succeed()
    }
}

// ANCHOR_END: step

fun main() {}
