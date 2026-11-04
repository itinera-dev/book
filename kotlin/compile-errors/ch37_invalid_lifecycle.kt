// An `on step failure` hook that tries to finish the journey as succeeded.

package ch37_invalid_lifecycle

import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.policy.ItineraStepPolicy
import org.itineradev.itinera.policy.OnFailure
import org.itineradev.itinera.policy.OnStepFailure
import org.itineradev.itinera.policy.StepPolicy
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/** Charges an order, and fails: the card was declined. */
@Step
class Charge : ItineraStep {
    fun run(): Outcome = fail("card-declined")
}

/** Tries to finish the journey as succeeded when the charge fails. */
@StepPolicy(name = "forgiving")
class Forgiving : ItineraStepPolicy {
    @OnStepFailure fun declined(): OnFailure = finishWorkflow()
}

/** The shop's workflow. */
@Workflow(name = "orders")
@WithStep(Charge::class, name = "charge", policies = [Forgiving::class])
class Orders : ItineraWorkflow

fun main() {
    val result = LocalExecutor().run(Orders().instance().create())
    println("the journey ${result.status}")
}
