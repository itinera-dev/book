// The shop's workflow, whole: three steps, a rescue step for the charge, a retry for shipping with
// the backup carrier on the second attempt, the audit and notify policies, and the services the
// steps use.

package ch02_big_picture;

import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;
import org.itineradev.itinera.event.Event;
import org.itineradev.itinera.event.JourneyInfo;
import org.itineradev.itinera.event.StepFailed;
import org.itineradev.itinera.event.StepGivenUp;
import org.itineradev.itinera.event.StepHookSource;
import org.itineradev.itinera.event.StepRescuing;
import org.itineradev.itinera.event.StepRetrying;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.Contributor;
import org.itineradev.itinera.journey.Failure;
import org.itineradev.itinera.journey.JourneyStatus;
import org.itineradev.itinera.policy.FromStep;
import org.itineradev.itinera.policy.FromWorkflow;
import org.itineradev.itinera.policy.HookReporter;
import org.itineradev.itinera.policy.ItineraStepPolicy;
import org.itineradev.itinera.policy.ItineraWorkflowPolicy;
import org.itineradev.itinera.policy.OnStepSuccess;
import org.itineradev.itinera.policy.OnSuccess;
import org.itineradev.itinera.policy.OnWorkflowFailure;
import org.itineradev.itinera.policy.OnWorkflowSuccess;
import org.itineradev.itinera.policy.Role;
import org.itineradev.itinera.policy.StepPolicy;
import org.itineradev.itinera.policy.WorkflowPolicy;
import org.itineradev.itinera.report.Reporter;
import org.itineradev.itinera.service.Provided;
import org.itineradev.itinera.service.Provider;
import org.itineradev.itinera.service.Service;
import org.itineradev.itinera.step.Attempt;
import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraRescueStep;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Reason;
import org.itineradev.itinera.step.RescueOutcome;
import org.itineradev.itinera.step.RescueStep;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.ListedService;
import org.itineradev.itinera.workflow.ListedStep;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;
import org.itineradev.itinera.workflow.WorkflowDescriptor;

// ANCHOR: workflow
/**
 * The shop's workflow: reserve, charge and ship, auditing the charge, rescuing it when the card has
 * expired, and telling the customer how the order went.
 */
@Workflow(
    name = "orders",
    policies = {Notify.class},
    reporters = {Log.class})
// The steps run in this order, and nothing reorders them.
@WithStep(step = Reserve.class, name = "reserve")
// The charge runs at most twice: once, and once more after its rescue step succeeds.
// Every successful charge is audited.
@WithStep(
    step = Charge.class,
    name = "charge",
    rescue = RenewCard.class,
    policies = {Audit.class})
// Shipping may be tried once more after a failure, which, for a parcel the carrier refused, is
// worth trying again.
@WithStep(step = Ship.class, name = "ship", retryBudget = 1)
final class Orders implements ItineraWorkflow, Messages {
  /** One payment client for the journey, used by the charge and its rescue step. */
  @Provider(key = "payments")
  public Provided<Payments> bank() {
    return provide(Fakes.BANK);
  }

  /** One stock store for the journey, shared by reserve and ship. */
  @Provider(key = "stock")
  public Provided<StockStore> warehouse() {
    return provide(Fakes.WAREHOUSE);
  }

  /** A carrier for every attempt: the main carrier first, the backup carrier after. */
  @Provider(perAttempt = true)
  public Provided<Carrier> carrier(Attempt attempt) {
    return provide(attempt.isFirst() ? Fakes.MAIN_CARRIER : Fakes.BACKUP_CARRIER);
  }

  /** How the shop tells a customer something, for the notify policy: here, by printing it. */
  @Override
  public void tell(String customer, String message) {
    System.out.println("  to " + customer + ": " + message);
  }
}

// ANCHOR_END: workflow

// ANCHOR: services
/** What the bank answers to a charge: the receipt, or why it refused. */
sealed interface Answer {
  record Paid(String receipt) implements Answer {}

  record ExpiredCard() implements Answer {}

  record Refused() implements Answer {}
}

/** A payment client: it charges a card, and knows the card that replaced an expired one. */
interface Payments {
  Answer charge(String card, long amount);

  String renewedCard(String card);
}

/** A stock store: it reserves an order's goods, then hands them over for shipping. */
interface StockStore {
  boolean reserve(String order);

  void handOver(String order);
}

/** A carrier: it takes a parcel and gives back its tracking number, or nothing when it cannot. */
interface Carrier {
  Optional<String> send(String order, long weight);
}

// ANCHOR_END: services

/**
 * The customers' bank, faked. A card ends with the year it expires, such as {@code ann-card-2028};
 * the bank refuses a card that expired before this year, and any charge above its limit.
 */
record Bank(int thisYear, long limit) implements Payments {
  @Override
  public Answer charge(String card, long amount) {
    if (Integer.parseInt(card.substring(card.lastIndexOf('-') + 1)) < thisYear) {
      return new Answer.ExpiredCard();
    }
    if (amount > limit) return new Answer.Refused();
    return new Answer.Paid("R-" + card + "-" + amount);
  }

  @Override
  public String renewedCard(String card) {
    return card.substring(0, card.lastIndexOf('-')) + "-" + (thisYear + 3);
  }
}

/** The warehouse, faked: it lists the orders it has no stock for. */
record Warehouse(List<String> outOfStock) implements StockStore {
  @Override
  public boolean reserve(String order) {
    return !outOfStock.contains(order);
  }

  @Override
  public void handOver(String order) {}
}

/** A carrier, faked: it takes parcels up to a weight. */
record Van(String trackingPrefix, long maxWeight) implements Carrier {
  @Override
  public Optional<String> send(String order, long weight) {
    if (weight > maxWeight) return Optional.empty();
    return Optional.of(trackingPrefix + "-" + order);
  }
}

/** The fakes every journey uses. */
final class Fakes {
  static final Bank BANK = new Bank(2026, 1000);
  static final Warehouse WAREHOUSE = new Warehouse(List.of("A-5"));
  static final Van MAIN_CARRIER = new Van("T", 20);
  static final Van BACKUP_CARRIER = new Van("B", 50);

  private Fakes() {}
}

// ANCHOR: steps
/** Reserves the order's goods in the stock store. */
@Step
final class Reserve implements ItineraStep {
  private final String order;

  Reserve(@Input String order) {
    this.order = order;
  }

  public Outcome run(@Service StockStore stock) {
    if (!stock.reserve(order)) return fail("out-of-stock");
    return succeed();
  }
}

/**
 * Charges the card the price, less any discount. It only says how it went: what happens after a
 * declined card is not its business.
 */
@Step
final class Charge implements ItineraStep {
  private final String card;
  private final long price;
  private final Optional<Long> discount;

  Charge(@Input String card, @Input long price, @Input Optional<Long> discount) {
    this.card = card;
    this.price = price;
    this.discount = discount;
  }

  public Outcome run(@Service Payments payments, Contributor contributor) {
    long amount = price - discount.orElse(0L);
    Answer answer = payments.charge(card, amount);
    if (answer instanceof Answer.Paid paid) {
      contributor.contribute("charged", amount);
      contributor.contribute("receipt", paid.receipt());
      return succeed();
    }
    if (answer instanceof Answer.ExpiredCard) return fail("expired-card");
    return fail("declined");
  }
}

/**
 * The charge's rescue step: asks the customer's bank for the card that replaced an expired one. A
 * charge that failed for any other reason, or ended in an abnormal termination, which has no
 * reason, cannot be repaired this way.
 */
@RescueStep
final class RenewCard implements ItineraRescueStep {
  private final String card;
  private final Optional<Reason> reason;

  RenewCard(@Input String card, Optional<Reason> reason) {
    this.card = card;
    this.reason = reason;
  }

  public RescueOutcome run(@Service Payments payments, Contributor contributor) {
    if (!reason.map(Reason::code).equals(Optional.of("expired-card"))) {
      return fail("nothing-to-renew");
    }
    contributor.contribute("card", payments.renewedCard(card));
    return succeed();
  }
}

/**
 * Ships the order through the carrier it is given, without knowing which one it is. A carrier that
 * cannot take the parcel might be replaced on another try, so that failure is worth trying again.
 */
@Step
final class Ship implements ItineraStep {
  private final String order;
  private final long weight;

  Ship(@Input String order, @Input long weight) {
    this.order = order;
    this.weight = weight;
  }

  public Outcome run(@Service Carrier carrier, @Service StockStore stock, Contributor contributor) {
    Optional<String> tracking = carrier.send(order, weight);
    if (tracking.isEmpty()) return fail("parcel-refused");
    stock.handOver(order);
    contributor.contribute("tracking", tracking.get());
    return succeed();
  }
}

// ANCHOR_END: steps

// ANCHOR: policies
/** Audits every successful charge, in the journey's events, including one after a rescue. */
@StepPolicy(name = "audit")
final class Audit implements ItineraStepPolicy {
  @OnStepSuccess
  public OnSuccess audited(
      @FromStep long charged, @FromStep String receipt, HookReporter reporter) {
    reporter.info("charged " + charged + ", receipt " + receipt);
    return keepDefault();
  }
}

/** A role the shop provides: telling a customer something. */
@Role
interface Messages {
  void tell(String customer, String message);
}

/** Tells the customer how the order went, once, when the journey ends. */
@WorkflowPolicy(name = "notify")
final class Notify implements ItineraWorkflowPolicy {
  @OnWorkflowSuccess
  public void shipped(
      @FromWorkflow String customer,
      @FromWorkflow String order,
      @FromWorkflow String tracking,
      @Role Messages messages) {
    messages.tell(customer, "order " + order + " is on its way: " + tracking);
  }

  @OnWorkflowFailure
  public void failed(
      @FromWorkflow String customer, @FromWorkflow String order, @Role Messages messages) {
    messages.tell(customer, "order " + order + " could not be completed");
  }
}

// ANCHOR_END: policies

/**
 * The shop's log: every failure, every decision and what the hooks report, for every journey, in
 * one format.
 */
final class Log implements Reporter {
  @Override
  public void report(Event event) {
    if (event instanceof StepFailed failed) {
      System.out.println(
          "  WARN "
              + failed.step()
              + ", attempt "
              + failed.attempt()
              + ": "
              + failed.reason().code());
    } else if (event instanceof StepRetrying retrying) {
      System.out.println("  INFO " + retrying.step() + " tried again");
    } else if (event instanceof StepRescuing rescuing) {
      System.out.println("  INFO " + rescuing.step() + " to be rescued by " + rescuing.rescue());
    } else if (event instanceof StepGivenUp givenUp) {
      System.out.println("  WARN " + givenUp.step() + " given up: " + givenUp.cause().getCause());
    } else if (event instanceof JourneyInfo info && info.source() instanceof StepHookSource hook) {
      System.out.println("  INFO " + hook.policy() + ": " + info.message());
    }
  }
}

/** What the shop knows about an order when the journey starts. */
record NewOrder(
    String order, String customer, String card, long price, Optional<Long> discount, long weight) {}

final class Main {
  private Main() {}

  /** A service in the listing: its key, and the provider that serves it. */
  static String served(ListedService listed) {
    return listed.key() + " from " + listed.provider();
  }

  /** The services of a listing's entry, separated by commas. */
  static String services(List<ListedService> listed) {
    return listed.stream().map(Main::served).collect(Collectors.joining(", "));
  }

  /**
   * One entry of the listing: a step's position and name, its policies, its services with the
   * provider that serves each, and its rescue step with its services.
   */
  static String entry(ListedStep listed) {
    var entry = new StringBuilder(listed.position() + ". " + listed.step());
    if (!listed.policies().isEmpty()) {
      entry.append(", with ");
      entry.append(
          listed.policies().stream().map(Object::toString).collect(Collectors.joining(", ")));
    }
    entry.append("; services: ").append(services(listed.services()));
    var rescue = listed.rescue();
    if (rescue != null) {
      entry.append("; rescued by " + rescue.step() + ", services: " + services(rescue.services()));
    }
    return entry.toString();
  }

  /** Why a journey failed: its cause, and the reason a step gave when it gave one. */
  static String why(Failure failure) {
    var reason = failure.reason();
    if (reason == null) return failure.cause().toString();
    return failure.cause() + ", " + reason.code();
  }

  public static void main(String[] args) throws Refusal {
    // ANCHOR: list
    // The workflow lists its steps, their policies, services and rescue step, without running
    // anything.
    for (var listed : WorkflowDescriptor.of(Orders.class).listing()) {
      System.out.println(entry(listed));
    }
    // ANCHOR_END: list
    // ANCHOR: orders
    // The order, the customer, the card, the price, the discount when there is one, and the
    // parcel's weight in kg.
    var orders =
        List.of(
            new NewOrder("A-1", "Ann", "ann-card-2028", 40, Optional.of(5L), 2),
            new NewOrder("A-2", "Ben", "ben-card-2024", 25, Optional.empty(), 1),
            new NewOrder("A-3", "Cat", "cat-card-2027", 30, Optional.empty(), 30),
            new NewOrder("A-4", "Dan", "dan-card-2025", 1200, Optional.empty(), 3),
            new NewOrder("A-5", "Eve", "eve-card-2027", 20, Optional.empty(), 2),
            new NewOrder("A-6", "Fin", "fin-card-2028", 1500, Optional.empty(), 4),
            new NewOrder("A-7", "Gil", "gil-card-2029", 45, Optional.empty(), 60));
    // One executor runs every journey, one after another, each an instance of the workflow made
    // with the order's data.
    var executor = new LocalExecutor();
    for (var order : orders) {
      System.out.println(order.order() + ":");
      var result = executor.run(new Orders().instance().initialData(order).create());
      var status = result.status();
      if (status instanceof JourneyStatus.Succeeded) {
        System.out.println("succeeded");
      } else if (status instanceof JourneyStatus.Failed failed) {
        System.out.println("failed: " + why(failed.failure()));
      } else {
        System.out.println("aborted");
      }
    }
    // ANCHOR_END: orders
  }
}
