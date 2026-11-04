// Saying what only a step knows: the shipping reports that the parcel left, as an event of its
// own in the journey's stream.

package ch09_saying_what_only_a_step_knows;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.itineradev.itinera.event.AttemptStarted;
import org.itineradev.itinera.event.ContributionCommitted;
import org.itineradev.itinera.event.Event;
import org.itineradev.itinera.event.JourneyFailed;
import org.itineradev.itinera.event.JourneyStarted;
import org.itineradev.itinera.event.ServiceProvided;
import org.itineradev.itinera.event.StepFailed;
import org.itineradev.itinera.event.StepGivenUp;
import org.itineradev.itinera.event.StepInfo;
import org.itineradev.itinera.event.StepSucceeded;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.Contributor;
import org.itineradev.itinera.report.Reporter;
import org.itineradev.itinera.service.Provided;
import org.itineradev.itinera.service.Provider;
import org.itineradev.itinera.service.Service;
import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.step.StepReporter;
import org.itineradev.itinera.value.TypeToken;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

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

// ANCHOR: ship
/**
 * Ships an order through the carrier, with its receipt in the parcel, gives back its tracking
 * number, and says when the parcel has left.
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

  public Outcome run(@Service Carrier carrier, Contributor contributor, StepReporter reporter) {
    String tracking = carrier.book(order);
    contributor.contribute("tracking", tracking);
    if (!carrier.collects(weight)) return fail("too-heavy");
    reporter.info("the parcel leaves with receipt " + receipt);
    return succeed();
  }
}

// ANCHOR_END: ship

/** The shop's log: it writes a line for every event of a journey. */
final class Log implements Reporter {
  @Override
  public void report(Event event) {
    String more = detail(event).map(text -> " " + text).orElse("");
    System.out.println("log #" + event.getSequence() + " " + event.getKind() + more);
  }

  /**
   * What the log says after an event's kind, for the kinds it says more about: the step the event
   * is about, and what it names.
   */
  static Optional<String> detail(Event event) {
    if (event instanceof JourneyStarted started) {
      return Optional.of(String.join(", ", started.initialKeys()));
    }
    if (event instanceof AttemptStarted started) return Optional.of(started.step().toString());
    if (event instanceof ServiceProvided provided) {
      return Optional.of(
          provided.step()
              + " gets "
              + provided.key()
              + " from "
              + provided.provider()
              + ", a "
              + provided.implementation());
    }
    if (event instanceof StepSucceeded succeeded) return Optional.of(succeeded.step().toString());
    if (event instanceof StepFailed failed) {
      return Optional.of(failed.step() + ": " + failed.reason().code());
    }
    if (event instanceof ContributionCommitted committed) return Optional.of(committed.key());
    if (event instanceof StepGivenUp givenUp) return Optional.of(givenUp.step().toString());
    if (event instanceof JourneyFailed failed) return Optional.of(failed.step().toString());
    // ANCHOR: step_info
    if (event instanceof StepInfo info) return Optional.of(info.step() + ": " + info.message());
    // ANCHOR_END: step_info
    return Optional.empty();
  }
}

/** The shop's workflow: it charges, then ships, and every journey is logged. */
@Workflow(
    name = "orders",
    reporters = {Log.class})
@WithStep(step = Charge.class, name = "charge")
@WithStep(step = Ship.class, name = "ship")
final class Orders implements ItineraWorkflow {
  /** The bank, as the payment client of the journey. */
  @Provider(key = "payments")
  public Provided<Payments> bank() {
    return provide(new Bank());
  }

  /** The main carrier, as the carrier of the journey. */
  @Provider(key = "carrier")
  public Provided<Carrier> mainCarrier() {
    return provide(new MainCarrier(20));
  }
}

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
