// A first journey: the shop runs the charge for two orders, and the result says how each journey
// ended.

package ch05_first_journey;

import java.util.List;
import java.util.Map;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

/** The shop's bank, faked: it prints what it charges. */
final class Bank {
  private Bank() {}

  static void charges(long amount) {
    System.out.println("the bank charges " + amount);
  }
}

/** Charges an order's price. */
@Step
final class Charge implements ItineraStep {
  private final long price;

  Charge(@Input long price) {
    this.price = price;
  }

  public Outcome run() {
    if (price <= 0) return fail("nothing-to-charge");
    Bank.charges(price);
    return succeed();
  }
}

// ANCHOR: workflow
/** The shop's workflow: for now, it only charges. */
@Workflow(name = "orders")
@WithStep(step = Charge.class, name = "charge")
final class Orders implements ItineraWorkflow {}

// ANCHOR_END: workflow

final class Main {
  private Main() {}

  // ANCHOR: run
  public static void main(String[] args) throws Refusal {
    // The customer, and the price of their order.
    for (var order : List.of(Map.entry("Ann", 40L), Map.entry("Hal", 0L))) {
      System.out.println(order.getKey() + "'s order:");
      var instance = new Orders().instance().data("price", order.getValue()).create();
      var result = new LocalExecutor().run(instance);
      if (result.isSucceeded()) {
        System.out.println("the journey succeeded");
      } else if (result.isFailed()) {
        System.out.println("the journey failed");
      }
    }
  }
  // ANCHOR_END: run
}
