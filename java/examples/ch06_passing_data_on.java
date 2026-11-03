// Passing data on: the charge gives back a receipt that the shipping puts in the parcel, and a
// failed attempt's contributions never reach the data bag.

package ch06_passing_data_on;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.Contributor;
import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.value.TypeToken;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

/** The shop's bank, faked: it prints what it charges, and gives a receipt for it. */
final class Bank {
  private Bank() {}

  static String charges(String order, long amount) {
    System.out.println("the bank charges " + amount + " for order " + order);
    return "R-" + order;
  }
}

/** The shop's carrier, faked. */
final class Carrier {
  private Carrier() {}

  /** Books any parcel, and gives its tracking number at once. */
  static String books(String order) {
    System.out.println("the carrier books order " + order);
    return "T-" + order;
  }

  /** Weighs a parcel when it comes to collect it, and takes it only up to 20 kg. */
  static boolean collects(long weight) {
    return weight <= 20;
  }
}

// ANCHOR: charge
/** Charges an order's price, and gives back the bank's receipt. */
@Step
final class Charge implements ItineraStep {
  private final String order;
  private final long price;

  Charge(@Input String order, @Input long price) {
    this.order = order;
    this.price = price;
  }

  public Outcome run(Contributor contributor) {
    if (price <= 0) return fail("nothing-to-charge");
    String receipt = Bank.charges(order, price);
    contributor.contribute("receipt", receipt);
    return succeed();
  }
}

// ANCHOR_END: charge

// ANCHOR: ship
/** Ships an order, with its receipt in the parcel, and gives back its tracking number. */
@Step
final class Ship implements ItineraStep {
  private final String order;
  private final long weight;
  private final String receipt;

  Ship(@Input String order, @Input long weight, @Input String receipt) {
    this.order = order;
    this.weight = weight;
    this.receipt = receipt;
  }

  public Outcome run(Contributor contributor) {
    String tracking = Carrier.books(order);
    contributor.contribute("tracking", tracking);
    if (!Carrier.collects(weight)) return fail("too-heavy");
    System.out.println("the parcel leaves with receipt " + receipt);
    return succeed();
  }
}

// ANCHOR_END: ship

// ANCHOR: workflow
/** The shop's workflow: it charges, then ships. */
@Workflow(name = "orders")
@WithStep(step = Charge.class, name = "charge")
@WithStep(step = Ship.class, name = "ship")
final class Orders implements ItineraWorkflow {}

// ANCHOR_END: workflow

// ANCHOR: data
/** What the shop knows about an order when its journey starts. */
record NewOrder(String order, long price, long weight) {}

// ANCHOR_END: data

// ANCHOR: output
/** What the shop reads back once the journey has ended. */
record Order(String receipt, Optional<String> tracking) {}

// ANCHOR_END: output

final class Main {
  private Main() {}

  // ANCHOR: run
  public static void main(String[] args) throws Refusal {
    // The customer, and their order: its number, its price and the parcel's weight in kg.
    var orders =
        List.of(
            Map.entry("Ann", new NewOrder("A-1", 40, 2)),
            Map.entry("Gil", new NewOrder("A-7", 45, 60)));
    for (var entry : orders) {
      System.out.println(entry.getKey() + "'s order:");
      var instance = new Orders().instance().initialData(entry.getValue()).create();
      var result = new LocalExecutor().run(instance);
      var data = result.output(TypeToken.of(Order.class));
      var ended = result.isSucceeded() ? "succeeded" : "failed";
      System.out.println(
          "the journey "
              + ended
              + ", with receipt "
              + data.receipt()
              + " and tracking "
              + data.tracking().orElse("none"));
    }
  }
  // ANCHOR_END: run
}
