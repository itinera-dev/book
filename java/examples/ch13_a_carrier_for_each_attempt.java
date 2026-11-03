// A carrier for each attempt: the workflow's provider makes a carrier for every attempt of the
// shipping, the main carrier first and the backup carrier after.

package ch13_a_carrier_for_each_attempt;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.itineradev.itinera.event.AttemptStarted;
import org.itineradev.itinera.event.ContributionCommitted;
import org.itineradev.itinera.event.Event;
import org.itineradev.itinera.event.JourneyFailed;
import org.itineradev.itinera.event.JourneyStarted;
import org.itineradev.itinera.event.ServiceProvided;
import org.itineradev.itinera.event.StepAbnormalTermination;
import org.itineradev.itinera.event.StepFailed;
import org.itineradev.itinera.event.StepGivenUp;
import org.itineradev.itinera.event.StepInfo;
import org.itineradev.itinera.event.StepRetrying;
import org.itineradev.itinera.event.StepSucceeded;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.Contributor;
import org.itineradev.itinera.journey.DataBag;
import org.itineradev.itinera.report.Reporter;
import org.itineradev.itinera.service.Provided;
import org.itineradev.itinera.service.Provider;
import org.itineradev.itinera.service.Service;
import org.itineradev.itinera.step.Attempt;
import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.step.StepReporter;
import org.itineradev.itinera.value.TypeToken;
import org.itineradev.itinera.workflow.IdGenerator;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

/**
 * A payment client: it charges an order's amount to a card, and gives back a receipt, or nothing
 * when the card has expired.
 */
interface Payments {
  Optional<String> charge(String order, String card, long amount) throws NetworkDown;
}

/**
 * A carrier: it books a parcel of this weight in kg and gives back its tracking number, then says
 * whether it collects the parcel.
 */
interface Carrier {
  String book(String order, long weight) throws BookingFailed;

  boolean collects(long weight);
}

/** The error of the bank's network when it fails. */
final class NetworkDown extends Exception {
  private static final long serialVersionUID = 1L;

  NetworkDown() {
    super("the payment network is down");
  }
}

/** The error of the carrier's booking system when it fails. */
final class BookingFailed extends Exception {
  private static final long serialVersionUID = 1L;

  BookingFailed() {
    super("the booking system failed");
  }
}

/**
 * The shop's bank, faked: a card is its holder's name and the year it expires, and the bank
 * declines a card that expired before this year. Its network fails on any amount over 100. It
 * prints what it does.
 */
final class Bank implements Payments {
  private final int thisYear;

  Bank(int thisYear) {
    this.thisYear = thisYear;
  }

  @Override
  public Optional<String> charge(String order, String card, long amount) throws NetworkDown {
    if (amount > 100) {
      System.out.println("the bank's network fails while charging order " + order);
      throw new NetworkDown();
    }
    if (expiry(card) < thisYear) {
      System.out.println("the bank declines card " + card + ": it has expired");
      return Optional.empty();
    }
    System.out.println("the bank charges " + amount + " for order " + order);
    return Optional.of("R-" + order);
  }

  /** The year a card expires: the part of it after its last {@code -}. */
  private static int expiry(String card) {
    return Integer.parseInt(card.substring(card.lastIndexOf('-') + 1));
  }
}

// ANCHOR: carriers
/**
 * The shop's main carrier, faked: its booking system fails on a parcel over 100 kg, and it collects
 * a parcel only up to 20 kg.
 */
final class MainCarrier implements Carrier {
  @Override
  public String book(String order, long weight) throws BookingFailed {
    if (weight > 100) {
      System.out.println("the main carrier's booking system fails on order " + order);
      throw new BookingFailed();
    }
    System.out.println("the main carrier books order " + order);
    return "T-" + order;
  }

  @Override
  public boolean collects(long weight) {
    return weight <= 20;
  }
}

/**
 * The shop's backup carrier, faked: its booking system fails on a parcel over 100 kg, and it
 * collects a parcel only up to 80 kg.
 */
final class BackupCarrier implements Carrier {
  @Override
  public String book(String order, long weight) throws BookingFailed {
    if (weight > 100) {
      System.out.println("the backup carrier's booking system fails on order " + order);
      throw new BookingFailed();
    }
    System.out.println("the backup carrier books order " + order);
    return "B-" + order;
  }

  @Override
  public boolean collects(long weight) {
    return weight <= 80;
  }
}

// ANCHOR_END: carriers

/** Charges an order's price to its card through the payment client, and gives back its receipt. */
@Step
final class Charge implements ItineraStep {
  private final String order;
  private final long price;
  private final String card;

  Charge(@Input String order, @Input long price, @Input String card) {
    this.order = order;
    this.price = price;
    this.card = card;
  }

  public Outcome run(@Service Payments payments, Contributor contributor) throws NetworkDown {
    if (price <= 0) return fail("nothing-to-charge");
    Optional<String> receipt = payments.charge(order, card, price);
    if (receipt.isEmpty()) return fail("card-expired");
    contributor.contribute("receipt", receipt.orElseThrow());
    return succeed();
  }
}

/**
 * Ships an order through the carrier, with its receipt in the parcel, gives back its tracking
 * number, and says when the parcel has left. A parcel the carrier won't collect may be taken on
 * another attempt.
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

  public Outcome run(@Service Carrier carrier, Contributor contributor, StepReporter reporter)
      throws BookingFailed {
    String tracking = carrier.book(order, weight);
    contributor.contribute("tracking", tracking);
    if (!carrier.collects(weight)) return fail("too-heavy");
    reporter.info("the parcel leaves with receipt " + receipt);
    return succeed();
  }
}

/**
 * The shop's log: it writes a line for every event of a journey, starting with the workflow's name
 * and the journey's ID.
 */
final class Log implements Reporter {
  @Override
  public void report(Event event) {
    String line =
        "log "
            + event.getWorkflow()
            + " "
            + event.getJourneyId()
            + " #"
            + event.getSequence()
            + " "
            + event.getKind();
    System.out.println(line + detail(event).map(text -> " " + text).orElse(""));
  }

  /**
   * What the log says after an event's kind, for the kinds it says more about: the step the event
   * is about, and what it names.
   */
  static Optional<String> detail(Event event) {
    if (event instanceof JourneyStarted started) {
      return Optional.of(String.join(", ", started.initialKeys()));
    }
    if (event instanceof AttemptStarted started) {
      return Optional.of(started.step() + ", attempt " + started.attempt());
    }
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
    if (event instanceof StepAbnormalTermination ended) {
      return Optional.of(ended.step() + ": " + ended.errorMessage());
    }
    if (event instanceof StepRetrying retrying) return Optional.of(retrying.step().toString());
    if (event instanceof StepGivenUp givenUp) {
      return Optional.of(givenUp.step() + ": " + givenUp.cause().getCause());
    }
    if (event instanceof ContributionCommitted committed) return Optional.of(committed.key());
    if (event instanceof JourneyFailed failed) return Optional.of(failed.step().toString());
    if (event instanceof StepInfo info) return Optional.of(info.step() + ": " + info.message());
    return Optional.empty();
  }
}

/** The customer service desk: of all the events of a journey, it keeps only a failed journey's. */
final class Desk implements Reporter {
  @Override
  public void report(Event event) {
    if (event instanceof JourneyFailed failed) {
      System.out.println("desk: order " + failed.journeyId() + " failed at " + failed.step());
    }
  }
}

/**
 * The shop's workflow: it charges, then ships, and each may be tried once more, except a charge
 * that ended in an error.
 */
@Workflow(
    name = "orders",
    reporters = {Log.class, Desk.class})
@WithStep(
    step = Charge.class,
    name = "charge",
    retryBudget = 1,
    noRetryOnAbnormalTermination = true)
@WithStep(step = Ship.class, name = "ship", retryBudget = 1)
final class Orders implements ItineraWorkflow {
  /** The bank, as the payment client of the journey. */
  @Provider(key = "payments")
  public Provided<Payments> bank() {
    return provide(new Bank(2026));
  }

  // ANCHOR: provider
  /** A carrier for every attempt of a step: the main carrier first, the backup carrier after. */
  @Provider(perAttempt = true, key = "carrier")
  public Provided<Carrier> carriers(Attempt attempt) {
    return attempt.isFirst() ? provide(new MainCarrier()) : provide(new BackupCarrier());
  }

  // ANCHOR_END: provider

  /** Names each journey after its order. */
  @IdGenerator
  public String orderNumber(DataBag data) {
    String order = data.read("order", TypeToken.of(String.class)).orNull();
    if (order == null) throw new IllegalStateException("an order without a number");
    return order;
  }
}

/** What the shop knows about an order when its journey starts. */
record NewOrder(String order, long price, long weight, String card) {}

/** What the shop reads back once the journey has ended. */
record Order(Optional<String> receipt, Optional<String> tracking) {}

final class Main {
  private Main() {}

  public static void main(String[] args) throws Refusal {
    // The customer, and their order: its number, its price, the parcel's weight in kg and the card.
    var orders =
        List.of(
            Map.entry("Ann", new NewOrder("A-1", 40, 2, "ann-2028")),
            Map.entry("Gil", new NewOrder("A-7", 45, 60, "gil-2027")),
            Map.entry("Jo", new NewOrder("A-8", 30, 1, "jo-2024")));
    for (var entry : orders) {
      System.out.println(entry.getKey() + "'s order:");
      var instance = new Orders().instance().initialData(entry.getValue()).create();
      var result = new LocalExecutor().run(instance);
      var data = result.output(TypeToken.of(Order.class));
      var ended = result.isSucceeded() ? "succeeded" : "failed";
      var receipt = data.receipt().orElse("none");
      var tracking = data.tracking().orElse("none");
      System.out.println(
          "the journey " + ended + ", with receipt " + receipt + " and tracking " + tracking);
    }
  }
}
