// An asynchronous workflow's instance given to the executor of synchronous ones.

package ch37_asynchronous_instance_run_locally

import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow
import org.itineradev.itinera.workflow.async.ItineraAsyncWorkflow

/** Charges an order, waiting for the bank. */
@Step
class Charge : ItineraStep {
    suspend fun run(): Outcome = succeed()
}

/** The shop's workflow, which runs asynchronously. */
@Workflow(name = "orders")
@WithStep(Charge::class, name = "charge")
class Orders : ItineraAsyncWorkflow

fun main() {
    val result = LocalExecutor().run(Orders().instance().create())
    println("the journey ${result.status}")
}
