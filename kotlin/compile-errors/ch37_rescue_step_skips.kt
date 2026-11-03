// A rescue step that tries to skip the charge rather than repair it.

package ch37_rescue_step_skips

import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.step.ItineraRescueStep
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.RescueOutcome
import org.itineradev.itinera.step.RescueStep
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/** Charges an order, and fails: the card has expired. */
@Step
class Charge : ItineraStep {
    fun run(): Outcome = fail("card-expired")
}

/** Gives up on the card, and tries to skip the charge. */
@RescueStep
class LetItGo : ItineraRescueStep {
    fun run(): RescueOutcome = skipBecause("card-expired")
}

/** The shop's workflow. */
@Workflow(name = "orders")
@WithStep(Charge::class, name = "charge", rescue = LetItGo::class)
class Orders : ItineraWorkflow

fun main() {
    val result = LocalExecutor().run(Orders().instance().create())
    println("the journey ${result.status}")
}
