// A policy that needs a role the workflow does not provide.

package ch37_role_not_provided;

import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.policy.FromWorkflow;
import org.itineradev.itinera.policy.ItineraWorkflowPolicy;
import org.itineradev.itinera.policy.OnWorkflowSuccess;
import org.itineradev.itinera.policy.Role;
import org.itineradev.itinera.policy.WorkflowPolicy;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

/** Charges an order. */
@Step
final class Charge implements ItineraStep {
  public Outcome run() {
    return succeed();
  }
}

/** The shop's way of telling a customer something. */
@Role
interface Messages {
  void tell(String customer, String message);
}

/** Tells the customer once the order is complete. */
@WorkflowPolicy(name = "notify")
final class Notify implements ItineraWorkflowPolicy {
  @OnWorkflowSuccess
  public void completed(@FromWorkflow String customer, @Role Messages messages) {
    messages.tell(customer, "your order is complete");
  }
}

/** The shop's workflow. It can tell a customer something, but does not implement the role. */
@Workflow(
    name = "orders",
    policies = {Notify.class})
@WithStep(step = Charge.class, name = "charge")
final class Orders implements ItineraWorkflow {
  public void tell(String customer, String message) {
    System.out.println("to " + customer + ": " + message);
  }
}

/** What the shop knows about an order when its journey starts. */
record NewOrder(String customer) {}

final class Main {
  private Main() {}

  public static void main(String[] args) throws Refusal {
    var instance = new Orders().instance().initialData(new NewOrder("Ann")).create();
    System.out.println("the journey " + new LocalExecutor().run(instance).status());
  }
}
