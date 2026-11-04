// Two policies of one step that both define `on step success`.

package ch37_hook_defined_twice;

import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.Contributor;
import org.itineradev.itinera.policy.FromStep;
import org.itineradev.itinera.policy.ItineraStepPolicy;
import org.itineradev.itinera.policy.OnStepSuccess;
import org.itineradev.itinera.policy.OnSuccess;
import org.itineradev.itinera.policy.StepPolicy;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

/** Charges an order. */
@Step
final class Charge implements ItineraStep {
  public Outcome run(Contributor contributor) {
    contributor.contribute("charged", 40L);
    return succeed();
  }
}

/** Audits every successful charge. */
@StepPolicy(name = "audit")
final class Audit implements ItineraStepPolicy {
  @OnStepSuccess
  public OnSuccess audited(@FromStep long charged) {
    System.out.println("audit: charged " + charged);
    return keepDefault();
  }
}

/** Writes a receipt for every successful charge. */
@StepPolicy(name = "receipts")
final class Receipts implements ItineraStepPolicy {
  @OnStepSuccess
  public OnSuccess written(@FromStep long charged) {
    System.out.println("receipt: R-" + charged);
    return keepDefault();
  }
}

/** The shop's workflow, with both policies attached to the charge. */
@Workflow(name = "orders")
@WithStep(
    step = Charge.class,
    name = "charge",
    policies = {Audit.class, Receipts.class})
final class Orders implements ItineraWorkflow {}

final class Main {
  private Main() {}

  public static void main(String[] args) throws Refusal {
    var result = new LocalExecutor().run(new Orders().instance().create());
    System.out.println("the journey " + result.status());
  }
}
