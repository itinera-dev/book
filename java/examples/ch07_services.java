// Giving a step the tools it needs: the workflow provides the bank and the carrier, and the steps
// use them without knowing which ones they got.

package ch07_services;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.Contributor;
import org.itineradev.itinera.service.Provided;
import org.itineradev.itinera.service.Provider;
import org.itineradev.itinera.service.Service;
import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.value.TypeToken;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

// ANCHOR: services
/** A payment client: it charges an order's amount, and gives back a receipt. */
interface Payments {
  String charge(String order, long amount);
}

/**
 * A carrier: it books a parcel and gives back its tracking number, then says whether it collects a
 * parcel of this weight in kg.
 */
interface Carrier {
  String book(String order);

  boolean collects(long weight);
}

// ANCHOR_END: services

/** The shop's bank, faked: it prints what it charges, and gives a receipt for it. */
final class Bank implements Payments {
  @Override
  public String charge(String order, long amount) {
    System.out.println("the bank charges " + amount + " for order " + order);
    return "R-" + order;
  }
}

/** The shop's carrier, faked: it books any parcel, and collects one only up to its weight limit. */
final class MainCarrier implements Carrier {
  private final long maxWeight;

  MainCarrier(long maxWeight) {
    this.maxWeight = maxWeight;
  }

  @Override
  public String book(String order) {
    System.out.println("the carrier books order " + order);
    return "T-" + order;
  }

  @Override
  public boolean collects(long weight) {
    return weight <= maxWeight;
  }
}

// ANCHOR: steps
/** Charges an order's price through the payment client, and gives back its receipt. */
@Step
final class Charge implements ItineraStep {
  private final String order;
  private final long price;

  Charge(@Input String order, @Input long price) {
    this.order = order;
    this.price = price;
  }

  public Outcome run(@Service Payments payments, Contributor contributor) {
    if (price <= 0) return fail("nothing-to-charge");
    String receipt = payments.charge(order, price);
    contributor.contribute("receipt", receipt);
    return succeed();
  }
}

/**
 * Ships an order through the carrier, with its receipt in the parcel, and gives back its tracking
 * number.
 */
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

  public Outcome run(@Service Carrier carrier, Contributor contributor) {
    String tracking = carrier.book(order);
    contributor.contribute("tracking", tracking);
    if (!carrier.collects(weight)) return fail("too-heavy");
    System.out.println("the parcel leaves with receipt " + receipt);
    return succeed();
  }
}

// ANCHOR_END: steps

// ANCHOR: workflow
/** The shop's workflow: it charges, then ships, with the services each step needs. */
@Workflow(name = "orders")
@WithStep(step = Charge.class, name = "charge")
@WithStep(step = Ship.class, name = "ship")
final class Orders implements ItineraWorkflow {
  /** The bank, as the payment client of the journey. */
  @Provider(key = "payments")
  public Provided<Payments> bank() {
    System.out.println("the shop connects to the bank");
    return provide(new Bank());
  }

  /** The main carrier, for the journey. */
  @Provider
  public Provided<Carrier> carrier() {
    System.out.println("the shop calls the carrier");
    return provide(new MainCarrier(20));
  }
}

// ANCHOR_END: workflow

/** What the shop knows about an order when its journey starts. */
record NewOrder(String order, long price, long weight) {}

/** What the shop reads back once the journey has ended. */
record Order(String receipt, Optional<String> tracking) {}

final class Main {
  private Main() {}

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
}
