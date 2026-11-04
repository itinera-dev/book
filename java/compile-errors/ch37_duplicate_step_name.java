// The shipping listed twice.

package ch37_duplicate_step_name;

import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.Contributor;
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
    contributor.contribute("receipt", "R-1");
    return succeed();
  }
}

/** Ships an order. */
@Step
final class Ship implements ItineraStep {
  public Outcome run() {
    return succeed();
  }
}

/** The shop's workflow, with the shipping listed twice under one name. */
@Workflow(name = "orders")
@WithStep(step = Charge.class, name = "charge")
@WithStep(step = Ship.class, name = "ship")
@WithStep(step = Ship.class, name = "ship")
final class Orders implements ItineraWorkflow {}

final class Main {
  private Main() {}

  public static void main(String[] args) throws Refusal {
    var result = new LocalExecutor().run(new Orders().instance().create());
    System.out.println("the journey " + result.status());
  }
}
