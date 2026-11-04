// A rescue step that tries to skip the charge rather than repair it.

package ch37_rescue_step_skips;

import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.step.ItineraRescueStep;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.RescueOutcome;
import org.itineradev.itinera.step.RescueStep;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

/** Charges an order, and fails: the card has expired. */
@Step
final class Charge implements ItineraStep {
  public Outcome run() {
    return fail("card-expired");
  }
}

/** Gives up on the card, and tries to skip the charge. */
@RescueStep
final class LetItGo implements ItineraRescueStep {
  public RescueOutcome run() {
    return skipBecause("card-expired");
  }
}

/** The shop's workflow. */
@Workflow(name = "orders")
@WithStep(step = Charge.class, name = "charge", rescue = LetItGo.class)
final class Orders implements ItineraWorkflow {}

final class Main {
  private Main() {}

  public static void main(String[] args) throws Refusal {
    var result = new LocalExecutor().run(new Orders().instance().create());
    System.out.println("the journey " + result.status());
  }
}
