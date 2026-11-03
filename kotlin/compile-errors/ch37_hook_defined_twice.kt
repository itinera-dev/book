// Two policies of one step that both define `on step success`.

package ch37_hook_defined_twice

import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.journey.Contributor
import org.itineradev.itinera.policy.FromStep
import org.itineradev.itinera.policy.ItineraStepPolicy
import org.itineradev.itinera.policy.OnStepSuccess
import org.itineradev.itinera.policy.OnSuccess
import org.itineradev.itinera.policy.StepPolicy
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
        contributor.contribute("charged", 40L)
        return succeed()
    }
}

/** Audits every successful charge. */
@StepPolicy(name = "audit")
class Audit : ItineraStepPolicy {
    @OnStepSuccess
    fun audited(@FromStep charged: Long): OnSuccess {
        println("audit: charged $charged")
        return keepDefault()
    }
}

/** Writes a receipt for every successful charge. */
@StepPolicy(name = "receipts")
class Receipts : ItineraStepPolicy {
    @OnStepSuccess
    fun written(@FromStep charged: Long): OnSuccess {
        println("receipt: R-$charged")
        return keepDefault()
    }
}

/** The shop's workflow, with both policies attached to the charge. */
@Workflow(name = "orders")
@WithStep(Charge::class, name = "charge", policies = [Audit::class, Receipts::class])
class Orders : ItineraWorkflow

fun main() {
    val result = LocalExecutor().run(Orders().instance().create())
    println("the journey ${result.status}")
}
