// An `on step failure` hook declared to return what only `on step success` returns.

package ch37_hook_of_the_wrong_kind;

import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.policy.ItineraStepPolicy;
import org.itineradev.itinera.policy.OnStepFailure;
import org.itineradev.itinera.policy.OnSuccess;
import org.itineradev.itinera.policy.StepPolicy;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

/** Charges an order, and fails: the card was declined. */
@Step
final class Charge implements ItineraStep {
  public Outcome run() {
    return fail("card-declined");
  }
}

/** Tries to finish the journey as succeeded when the charge fails. */
@StepPolicy(name = "forgiving")
final class Forgiving implements ItineraStepPolicy {
  @OnStepFailure
  public OnSuccess declined() {
    return finishWorkflow();
  }
}

/** The shop's workflow. */
@Workflow(name = "orders")
@WithStep(
    step = Charge.class,
    name = "charge",
    policies = {Forgiving.class})
final class Orders implements ItineraWorkflow {}

final class Main {
  private Main() {}

  public static void main(String[] args) throws Refusal {
    var result = new LocalExecutor().run(new Orders().instance().create());
    System.out.println("the journey " + result.status());
  }
}
