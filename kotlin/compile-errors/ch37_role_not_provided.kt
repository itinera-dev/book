// A policy that needs a role the workflow does not provide.

package ch37_role_not_provided

import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.policy.FromWorkflow
import org.itineradev.itinera.policy.ItineraWorkflowPolicy
import org.itineradev.itinera.policy.OnWorkflowSuccess
import org.itineradev.itinera.policy.Role
import org.itineradev.itinera.policy.WorkflowPolicy
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/** Charges an order. */
@Step
class Charge : ItineraStep {
    fun run(): Outcome = succeed()
}

/** The shop's way of telling a customer something. */
@Role
interface Messages {
    fun tell(customer: String, message: String)
}

/** Tells the customer once the order is complete. */
@WorkflowPolicy(name = "notify")
class Notify : ItineraWorkflowPolicy {
    @OnWorkflowSuccess
    fun completed(@FromWorkflow customer: String, @Role messages: Messages) {
        messages.tell(customer, "your order is complete")
    }
}

/** The shop's workflow. It can tell a customer something, but does not implement the role. */
@Workflow(name = "orders", policies = [Notify::class])
@WithStep(Charge::class, name = "charge")
class Orders : ItineraWorkflow {
    fun tell(customer: String, message: String) {
        println("to $customer: $message")
    }
}

/** What the shop knows about an order when its journey starts. */
data class NewOrder(val customer: String)

fun main() {
    val instance = Orders().instance().initialData(NewOrder(customer = "Ann")).create()
    println("the journey ${LocalExecutor().run(instance).status}")
}
