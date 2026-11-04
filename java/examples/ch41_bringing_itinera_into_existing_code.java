// Chapter 1's checkout with its payment moved into a workflow: the charge is a step, renewing an
// expired card is its rescue step and the audit is a policy, while the bank's client, the audit
// trail, the log and the rest of the checkout stay as they were.

package ch41_bringing_itinera_into_existing_code;

import java.util.List;
import java.util.Optional;
import org.itineradev.itinera.event.Event;
import org.itineradev.itinera.event.StepFailed;
import org.itineradev.itinera.event.StepRescued;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.Contributor;
import org.itineradev.itinera.journey.DataBag;
import org.itineradev.itinera.journey.JourneyResult;
import org.itineradev.itinera.journey.JourneyStatus;
import org.itineradev.itinera.policy.FromStep;
import org.itineradev.itinera.policy.FromWorkflow;
import org.itineradev.itinera.policy.ItineraStepPolicy;
import org.itineradev.itinera.policy.OnStepSuccess;
import org.itineradev.itinera.policy.OnSuccess;
import org.itineradev.itinera.policy.StepPolicy;
import org.itineradev.itinera.report.Reporter;
import org.itineradev.itinera.service.Provided;
import org.itineradev.itinera.service.Provider;
import org.itineradev.itinera.service.Service;
import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraRescueStep;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Reason;
import org.itineradev.itinera.step.RescueOutcome;
import org.itineradev.itinera.step.RescueStep;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.value.TypeToken;
import org.itineradev.itinera.workflow.IdGenerator;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

// The outside world the shop talks to, faked so that every run is the same.

/** Why the bank refused a charge. */
enum Why {
  /** The card has expired; the bank may know the card that replaced it. */
  EXPIRED_CARD("card expired"),
  /** The bank refused the charge. */
  REFUSED("payment declined");

  final String message;

  Why(String message) {
    this.message = message;
  }
}

/** The bank refused a charge. */
final class Declined extends Exception {
  private static final long serialVersionUID = 1L;

  final Why why;

  Declined(Why why) {
    super(why.message);
    this.why = why;
  }
}

/**
 * The customers' bank. A card ends with the year it expires, such as {@code ann-card-2028}; the
 * bank refuses a card that expired before this year, and any charge above its limit.
 */
final class Bank {
  private final int thisYear;
  private final long limit;

  Bank(int thisYear, long limit) {
    this.thisYear = thisYear;
    this.limit = limit;
  }

  /** Charges an amount to a card, and returns the receipt. */
  String charge(String card, long amount) throws Declined {
    if (Integer.parseInt(card.substring(card.lastIndexOf('-') + 1)) < thisYear) {
      throw new Declined(Why.EXPIRED_CARD);
    }
    if (amount > limit) throw new Declined(Why.REFUSED);
    return "R-" + card + "-" + amount;
  }

  /** The card that replaced an expired one: the same card, expiring three years from now. */
  String renewedCard(String card) {
    return card.substring(0, card.lastIndexOf('-')) + "-" + (thisYear + 3);
  }
}

/** The warehouse, which holds the stock. */
final class Warehouse {
  private final List<String> outOfStock;

  Warehouse(List<String> outOfStock) {
    this.outOfStock = outOfStock;
  }

  /** Sets the order's goods aside. */
  void reserve(String order) {
    if (outOfStock.contains(order)) throw new IllegalStateException("nothing left for " + order);
  }
}

/** A carrier, which takes parcels up to a weight. */
final class Carrier {
  final String name;
  private final String trackingPrefix;
  private final long maxWeight;

  Carrier(String name, String trackingPrefix, long maxWeight) {
    this.name = name;
    this.trackingPrefix = trackingPrefix;
    this.maxWeight = maxWeight;
  }

  /** Takes the order's parcel, and returns its tracking number. */
  String ship(String order, long weight) {
    if (weight > maxWeight) {
      throw new IllegalStateException(weight + " kg is over its " + maxWeight + " kg limit");
    }
    return trackingPrefix + "-" + order;
  }
}

/** Sends messages to customers. */
final class Messages {
  /** Tells a customer something. */
  void tell(String customer, String message) {
    System.out.println("to " + customer + ": " + message);
  }
}

// The shop's logging, used here and there.

final class Log {
  private Log() {}

  static void info(String message) {
    System.out.println("INFO " + message);
  }

  static void warn(String message) {
    System.out.println("WARN " + message);
  }
}

// The shop's audit trail.

final class Audit {
  private Audit() {}

  static void record(String entry) {
    System.out.println("AUDIT " + entry);
  }
}

// Taking the payment for an order: a workflow of its own.

// ANCHOR: payments
/** What the payment's steps ask of the bank. */
interface Payments {
  String charge(String card, long amount) throws Declined;

  String renewedCard(String card);
}

/** The bank's client, as it was, behind the steps' interface. */
final class BankPayments implements Payments {
  private final Bank bank;

  BankPayments(Bank bank) {
    this.bank = bank;
  }

  @Override
  public String charge(String card, long amount) throws Declined {
    return bank.charge(card, amount);
  }

  @Override
  public String renewedCard(String card) {
    return bank.renewedCard(card);
  }
}

// ANCHOR_END: payments

// ANCHOR: steps
/** Charges the amount to the card, and gives back the receipt. */
@Step
final class Charge implements ItineraStep {
  private final String card;
  private final long amount;

  Charge(@Input String card, @Input long amount) {
    this.card = card;
    this.amount = amount;
  }

  public Outcome run(@Service Payments payments, Contributor contributor) {
    try {
      contributor.contribute("receipt", payments.charge(card, amount));
      return succeed();
    } catch (Declined declined) {
      return switch (declined.why) {
        case EXPIRED_CARD -> fail("card-expired");
        case REFUSED -> fail("declined");
      };
    }
  }
}

/** Repairs a charge whose card had expired: it asks the bank for the card that replaced it. */
@RescueStep
final class RenewCard implements ItineraRescueStep {
  private final String card;
  private final Reason reason;

  RenewCard(@Input String card, Reason reason) {
    this.card = card;
    this.reason = reason;
  }

  public RescueOutcome run(@Service Payments payments, Contributor contributor) {
    if (!reason.code().equals("card-expired")) return fail("not-repairable");
    contributor.contribute("card", payments.renewedCard(card));
    return succeed();
  }
}

// ANCHOR_END: steps

// ANCHOR: audit
/** Writes every successful charge to the shop's audit trail. */
@StepPolicy(name = "audit")
final class AuditCharge implements ItineraStepPolicy {
  @OnStepSuccess
  public OnSuccess charged(@FromWorkflow long amount, @FromStep String receipt) {
    Audit.record("charged " + amount + ", receipt " + receipt);
    return keepDefault();
  }
}

// ANCHOR_END: audit

// ANCHOR: log
/** Writes to the shop's log when a step fails, and when a rescue step repairs it. */
final class PaymentLog implements Reporter {
  @Override
  public void report(Event event) {
    if (event instanceof StepFailed failed) {
      Log.info(
          "payment "
              + failed.journeyId()
              + ": "
              + failed.step()
              + " failed, "
              + failed.reason().code());
    } else if (event instanceof StepRescued rescued) {
      Log.info(
          "payment "
              + rescued.journeyId()
              + ": "
              + rescued.step()
              + " repaired by "
              + rescued.rescue());
    }
  }
}

// ANCHOR_END: log

// ANCHOR: workflow
/** Charges an order, renewing a card that has expired, and audits the charge. */
@Workflow(
    name = "payment",
    reporters = {PaymentLog.class})
@WithStep(
    step = Charge.class,
    name = "charge",
    rescue = RenewCard.class,
    policies = {AuditCharge.class})
final class Payment implements ItineraWorkflow {
  /** Names each journey after its order. */
  @IdGenerator
  public String orderNumber(DataBag data) {
    String order = data.read("order", TypeToken.of(String.class)).orNull();
    if (order == null) throw new IllegalStateException("a payment without an order");
    return order;
  }

  /** The bank, as it is configured today. */
  @Provider(key = "payments")
  public Provided<Payments> bank() {
    return provide(new BankPayments(new Bank(2026, 1000)));
  }
}

/** What a payment starts with. */
record Due(String order, String card, long amount) {}

/** What a payment gives back. */
record Paid(String receipt) {}

// ANCHOR_END: workflow

// Shipping an order.

final class Shipping {
  private Shipping() {}

  /**
   * Ships through the main carrier, or through the backup carrier when the main one cannot take the
   * parcel. Returns the tracking number, or nothing when neither can take it.
   */
  static Optional<String> ship(Carrier main, Carrier backup, String order, long weight) {
    try {
      return Optional.of(main.ship(order, weight));
    } catch (IllegalStateException refused) {
      Log.warn(
          main.name
              + " cannot take "
              + order
              + ": "
              + refused.getMessage()
              + "; trying "
              + backup.name);
      try {
        return Optional.of(backup.ship(order, weight));
      } catch (IllegalStateException alsoRefused) {
        return Optional.empty();
      }
    }
  }
}

// The checkout: what happens to an order once the customer has paid.

// ANCHOR: checkout
/** An order, as the shop's website hands it over. */
record Order(
    String order, String customer, String card, long price, Optional<Long> discount, long weight) {}

/** The payment workflow could not say how the payment went. */
final class PaymentNotRun extends Exception {
  private static final long serialVersionUID = 1L;

  PaymentNotRun(String why) {
    super("the payment did not run: " + why);
  }
}

/** The shop, with everything it talks to. */
final class Shop {
  private final Warehouse warehouse;
  private final Carrier main;
  private final Carrier backup;
  private final Messages messages;

  Shop(Warehouse warehouse, Carrier main, Carrier backup, Messages messages) {
    this.warehouse = warehouse;
    this.main = main;
    this.backup = backup;
    this.messages = messages;
  }

  /** Reserves, charges and ships an order, and tells the customer how it went. */
  String checkout(Order order) throws Declined, PaymentNotRun {
    Log.info("checkout " + order.order());
    try {
      String tracking = reserveChargeAndShip(order);
      messages.tell(order.customer(), "order " + order.order() + " is on its way: " + tracking);
      return tracking;
    } catch (Exception error) {
      Log.warn("checkout " + order.order() + " failed: " + error.getMessage());
      messages.tell(order.customer(), "order " + order.order() + " could not be completed");
      throw error;
    }
  }

  private String reserveChargeAndShip(Order order) throws Declined, PaymentNotRun {
    warehouse.reserve(order.order());
    long amount = order.price() - order.discount().orElse(0L);
    pay(order.order(), order.card(), amount);
    return Shipping.ship(main, backup, order.order(), order.weight())
        .orElseThrow(() -> new IllegalStateException("no carrier could take it"));
  }

  // ANCHOR_END: checkout

  // ANCHOR: pay
  /** Charges an order's card through the payment workflow, and gives back the receipt. */
  private String pay(String order, String card, long amount) throws Declined, PaymentNotRun {
    var instance = new Payment().instance().initialData(new Due(order, card, amount)).create();
    JourneyResult result;
    try {
      result = new LocalExecutor().run(instance);
    } catch (Refusal refusal) {
      throw new PaymentNotRun(refusal.getMessage());
    }
    JourneyStatus status = result.status();
    if (status instanceof JourneyStatus.Aborted aborted) {
      throw new PaymentNotRun(aborted.abort().reason().toString());
    }
    if (status instanceof JourneyStatus.Failed failed) {
      Reason reason = failed.failure().reason();
      if (reason != null && reason.code().equals("card-expired")) {
        throw new Declined(Why.EXPIRED_CARD);
      }
      throw new Declined(Why.REFUSED);
    }
    return result.output(TypeToken.of(Paid.class)).receipt();
  }

  // ANCHOR_END: pay
}

final class Main {
  private Main() {}

  public static void main(String[] args) {
    var shop =
        new Shop(
            new Warehouse(List.of()),
            new Carrier("Rapid", "T", 20),
            new Carrier("Steady", "B", 50),
            new Messages());
    var orders =
        List.of(
            new Order("A-1", "Ann", "ann-card-2028", 40, Optional.of(5L), 2),
            new Order("A-2", "Ben", "ben-card-2024", 25, Optional.empty(), 1),
            new Order("A-3", "Cat", "cat-card-2027", 30, Optional.empty(), 30),
            new Order("A-4", "Dan", "dan-card-2025", 1200, Optional.empty(), 3));
    for (var order : orders) {
      try {
        System.out.println(order.order() + ": shipped, " + shop.checkout(order));
      } catch (Exception error) {
        System.out.println(order.order() + ": failed, " + error.getMessage());
      }
      System.out.println();
    }
  }
}
