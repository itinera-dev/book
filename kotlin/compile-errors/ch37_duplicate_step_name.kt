// The shipping listed twice.

package ch37_duplicate_step_name

import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.journey.Contributor
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/** Charges an order. */
@Step
class Charge : ItineraStep {
    fun run(contributor: Contributor): Outcome {
        contributor.contribute("receipt", "R-1")
        return succeed()
    }
}

/** Ships an order. */
@Step
class Ship : ItineraStep {
    fun run(): Outcome = succeed()
}

/** The shop's workflow, with the shipping listed twice under one name. */
@Workflow(name = "orders")
@WithStep(Charge::class, name = "charge")
@WithStep(Ship::class, name = "ship")
@WithStep(Ship::class, name = "ship")
class Orders : ItineraWorkflow

fun main() {
    val result = LocalExecutor().run(Orders().instance().create())
    println("the journey ${result.status}")
}
