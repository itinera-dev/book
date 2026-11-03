// Testing a whole journey: the shop of chapter 25, whose tests, beside it, run whole journeys with
// fake providers in place of the workflow's, and compare each journey's path with an approved one.

package ch30_testing_a_whole_journey;

import java.util.List;
import java.util.Optional;
import org.itineradev.itinera.event.AttemptStarted;
import org.itineradev.itinera.event.ContributionCommitted;
import org.itineradev.itinera.event.ContributionsDiscarded;
import org.itineradev.itinera.event.DataOverwritten;
import org.itineradev.itinera.event.DecidingHook;
import org.itineradev.itinera.event.Event;
import org.itineradev.itinera.event.GiveUpCause;
import org.itineradev.itinera.event.InputAdapterSupplied;
import org.itineradev.itinera.event.JourneyFailed;
import org.itineradev.itinera.event.JourneyStarted;
import org.itineradev.itinera.event.JourneySucceeded;
import org.itineradev.itinera.event.MissingData;
import org.itineradev.itinera.event.OptionalInputAbsent;
import org.itineradev.itinera.event.RequiredDataMissing;
import org.itineradev.itinera.event.ServiceProvided;
import org.itineradev.itinera.event.StepAbnormalTermination;
import org.itineradev.itinera.event.StepFailed;
import org.itineradev.itinera.event.StepGivenUp;
import org.itineradev.itinera.event.StepHookCalled;
import org.itineradev.itinera.event.StepInfo;
import org.itineradev.itinera.event.StepRescued;
import org.itineradev.itinera.event.StepRescuing;
import org.itineradev.itinera.event.StepRetrying;
import org.itineradev.itinera.event.StepSkipped;
import org.itineradev.itinera.event.StepSucceeded;
import org.itineradev.itinera.event.WorkflowHookCalled;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.Abort;
import org.itineradev.itinera.journey.Contributor;
import org.itineradev.itinera.journey.DataBag;
import org.itineradev.itinera.journey.Failure;
import org.itineradev.itinera.journey.FailureCause;
import org.itineradev.itinera.journey.JourneyResult;
import org.itineradev.itinera.journey.JourneyStatus;
import org.itineradev.itinera.policy.FromStep;
import org.itineradev.itinera.policy.FromWorkflow;
import org.itineradev.itinera.policy.ItineraStepPolicy;
import org.itineradev.itinera.policy.ItineraWorkflowPolicy;
import org.itineradev.itinera.policy.OnFailure;
import org.itineradev.itinera.policy.OnStepAbnormalTermination;
import org.itineradev.itinera.policy.OnStepFailure;
import org.itineradev.itinera.policy.OnStepRetry;
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
import org.itineradev.itinera.step.StepName;
import org.itineradev.itinera.step.StepReporter;
import org.itineradev.itinera.value.TypeToken;
import org.itineradev.itinera.workflow.IdGenerator;
import org.itineradev.itinera.workflow.InputAdapter;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.Key;
import org.itineradev.itinera.workflow.Supplied;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

/**
 * A payment client: it charges an order's amount to a card, and gives back a receipt, or nothing
 * when the card has expired. And it says which card replaced one, or nothing when none did.
 */
interface Payments {
  Optional<String> charge(String order, String card, long amount) throws NetworkDown;

  Optional<String> replacement(String card);
}

/**
 * A carrier: it books a parcel of this weight in kg and gives back its tracking number, then says
 * whether it collects the parcel.
 */
interface Carrier {
  String book(String order, long weight) throws BookingFailed;

  boolean collects(long weight);
}

/** The shop's order book: it says whether an order was already completed. */
interface OrderBook {
  boolean completed(String order);
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

  /**
   * A card that expired in the last three years was replaced by its holder's card that expires in
   * three years. An older one was not.
   */
  @Override
  public Optional<String> replacement(String card) {
    if (expiry(card) + 3 < thisYear) {
      System.out.println("the bank says nothing replaced card " + card);
      return Optional.empty();
    }
    String replacement = card.substring(0, card.lastIndexOf('-')) + "-" + (thisYear + 3);
    System.out.println("the bank says card " + card + " was replaced by " + replacement);
    return Optional.of(replacement);
  }

  /** The year a card expires: the part of it after its last {@code -}. */
  private static int expiry(String card) {
    return Integer.parseInt(card.substring(card.lastIndexOf('-') + 1));
  }
}

/**
 * The shop's main carrier, faked: its booking system fails on a parcel that weighs nothing, and it
 * collects a parcel only up to 20 kg.
 */
final class MainCarrier implements Carrier {
  @Override
  public String book(String order, long weight) throws BookingFailed {
    if (weight == 0) {
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
 * The shop's backup carrier, faked: its booking system fails on a parcel that weighs nothing, and
 * it collects a parcel only up to 80 kg.
 */
final class BackupCarrier implements Carrier {
  @Override
  public String book(String order, long weight) throws BookingFailed {
    if (weight == 0) {
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

/**
 * The shop's order book, faked: it holds the orders completed before today, and prints what it
 * finds.
 */
final class Ledger implements OrderBook {
  private final List<String> done;

  Ledger(List<String> done) {
    this.done = done;
  }

  @Override
  public boolean completed(String order) {
    if (done.contains(order)) {
      System.out.println("the order book has order " + order + " completed already");
      return true;
    }
    System.out.println("the order book has no order " + order + " yet");
    return false;
  }
}

/** Looks the order up in the order book, and gives back whether it was already completed. */
@Step
final class Lookup implements ItineraStep {
  private final String order;

  Lookup(@Input String order) {
    this.order = order;
  }

  public Outcome run(@Service OrderBook orderBook, Contributor contributor) {
    contributor.contribute("completed", orderBook.completed(order));
    return succeed();
  }
}

/**
 * Finishes the journey of an order that was already completed: there is nothing left to do for it.
 */
@StepPolicy(name = "alreadyCompleted")
final class AlreadyCompleted implements ItineraStepPolicy {
  @OnStepSuccess
  public OnSuccess lookedUp(@FromStep boolean completed) {
    return completed ? finishWorkflow() : keepDefault();
  }
}

/**
 * Works out what an order costs: its price, less its discount if it has one, given back as the
 * amount to charge.
 */
@Step
final class Price implements ItineraStep {
  private final long price;
  private final Optional<Long> discount;

  Price(@Input long price, @Input Optional<Long> discount) {
    this.price = price;
    this.discount = discount;
  }

  public Outcome run(Contributor contributor) {
    contributor.contribute("amount", price - discount.orElse(0L));
    return succeed();
  }
}

/**
 * Charges an order's amount to its card through the payment client, and gives back its receipt, and
 * the gift card's code when the order is a gift card.
 */
@Step
final class Charge implements ItineraStep {
  private final String order;
  private final long amount;
  private final String card;
  private final boolean giftCard;

  Charge(@Input String order, @Input long amount, @Input String card, @Input boolean giftCard) {
    this.order = order;
    this.amount = amount;
    this.card = card;
    this.giftCard = giftCard;
  }

  public Outcome run(@Service Payments payments, Contributor contributor) throws NetworkDown {
    if (amount <= 0) return fail("nothing-to-charge");
    Optional<String> receipt = payments.charge(order, card, amount);
    if (receipt.isEmpty()) return fail("card-expired");
    contributor.contribute("receipt", receipt.orElseThrow());
    if (giftCard) contributor.contribute("code", "G-" + order);
    return succeed();
  }
}

/**
 * Repairs a charge that failed because its card had expired: it asks the bank for the card that
 * replaced it, and gives it back as the order's card.
 */
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
    Optional<String> replacement = payments.replacement(card);
    if (replacement.isEmpty()) return fail("not-replaced");
    contributor.contribute("card", replacement.orElseThrow());
    return succeed();
  }
}

/**
 * Writes an audit line for every step it is attached to that succeeds: the amount the order was
 * charged, and the receipt the step gave back.
 */
@StepPolicy(name = "audit")
final class Audit implements ItineraStepPolicy {
  @OnStepSuccess
  public OnSuccess charged(
      StepName step, @FromWorkflow long amount, @FromStep String receipt, Contributor contributor) {
    contributor.contribute("audit", step + " took " + amount + ", receipt " + receipt);
    return keepDefault();
  }
}

/** What the shop does for its policies: tell a customer something. */
@Role
interface Messages {
  void tell(String customer, String message);
}

/** Tells the customer how their order went, once its journey has ended. */
@WorkflowPolicy(name = "notify")
final class Notify implements ItineraWorkflowPolicy {
  @OnWorkflowSuccess
  public void succeeded(
      @FromWorkflow String customer,
      @FromWorkflow String order,
      @FromWorkflow Optional<String> tracking,
      @FromWorkflow Optional<String> code,
      @Role Messages messages) {
    String message;
    if (tracking.isPresent()) {
      message = "Order " + order + " is on its way, tracking " + tracking.orElseThrow() + ".";
    } else if (code.isPresent()) {
      message = "Order " + order + " is paid: your gift card's code is " + code.orElseThrow() + ".";
    } else {
      message = "Order " + order + " is complete.";
    }
    messages.tell(customer, message);
  }

  @OnWorkflowFailure
  public void failed(
      @FromWorkflow String customer, @FromWorkflow String order, @Role Messages messages) {
    messages.tell(customer, "Sorry, order " + order + " could not be completed.");
  }
}

/**
 * Gives up a step that ends in an error at once: the bank's network is down, and neither a retry
 * nor a rescue can mend that.
 */
@StepPolicy(name = "outage")
final class Outage implements ItineraStepPolicy {
  @OnStepAbnormalTermination
  public OnFailure endedInError() {
    return failWorkflow("payment-network-down");
  }
}

/** What the shop does for its policies: give an order's money back. */
@Role
interface Refunds {
  void refund(String receipt, long amount);
}

/**
 * Deals with a shipping that fails for good. It gives up a parcel over the shop's limit rather than
 * try it again, since the limit is the same on every attempt, whichever carrier comes. And whatever
 * gave the shipping up, it refunds the order's charge.
 */
@StepPolicy(name = "failedShipping")
final class FailedShipping implements ItineraStepPolicy {
  @OnStepRetry
  public OnFailure beforeRetrying(
      Optional<Reason> reason,
      @FromWorkflow String receipt,
      @FromWorkflow long amount,
      @Role Refunds refunds,
      Contributor contributor) {
    boolean overLimit = reason.map(Reason::code).filter("over-parcel-limit"::equals).isPresent();
    if (!overLimit) return keepDefault();
    refund(refunds, contributor, receipt, amount, "over-parcel-limit");
    return failWorkflow("no-carrier-can-take-it");
  }

  @OnStepFailure
  public OnFailure givenUp(
      FailureCause cause,
      @FromWorkflow String receipt,
      @FromWorkflow long amount,
      @Role Refunds refunds,
      Contributor contributor) {
    refund(refunds, contributor, receipt, amount, cause.toString());
    return keepDefault();
  }

  /** Gives the order's money back, and records why. */
  private static void refund(
      Refunds refunds, Contributor contributor, String receipt, long amount, String why) {
    refunds.refund(receipt, amount);
    contributor.contribute("refund", amount + " back on " + receipt + ": " + why);
  }
}

/**
 * Ships an order through the carrier, with its receipt in the parcel, gives back its tracking
 * number, and says when the parcel has left. A parcel the carrier won't collect may be taken on
 * another attempt. A gift card has nothing to ship, and the shop ships no parcel over 80 kg.
 */
@Step
final class Ship implements ItineraStep {
  private final String order;
  private final long weight;
  private final String receipt;
  private final boolean giftCard;

  Ship(@Input String order, @Input long weight, @Input String receipt, @Input boolean giftCard) {
    this.order = order;
    this.weight = weight;
    this.receipt = receipt;
    this.giftCard = giftCard;
  }

  public Outcome run(@Service Carrier carrier, Contributor contributor, StepReporter reporter)
      throws BookingFailed {
    if (giftCard) return skipBecause("nothing-to-ship");
    if (weight > 80) return fail("over-parcel-limit");
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
    if (event instanceof StepGivenUp givenUp) return Optional.of(givenUp(givenUp));
    if (event instanceof ContributionCommitted committed) return Optional.of(committed.key());
    if (event instanceof JourneyFailed failed) return Optional.of(failed.step().toString());
    if (event instanceof StepInfo info) return Optional.of(info.step() + ": " + info.message());
    if (event instanceof StepRescuing rescuing) {
      return Optional.of(
          rescuing.step() + ": " + rescuing.cause() + ", rescued by " + rescuing.rescue());
    }
    if (event instanceof StepRescued rescued) {
      return Optional.of(rescued.step() + " by " + rescued.rescue());
    }
    if (event instanceof DataOverwritten overwritten) return Optional.of(overwritten.key());
    if (event instanceof StepSkipped skipped) {
      Reason reason = skipped.reason();
      return Optional.of(skipped.step() + ": " + (reason == null ? "no reason" : reason.code()));
    }
    if (event instanceof ContributionsDiscarded discarded) {
      return Optional.of(discarded.step().toString());
    }
    if (event instanceof StepHookCalled called) {
      return Optional.of(
          called.source().policy()
              + ": "
              + called.source().hook()
              + " of "
              + called.source().step());
    }
    if (event instanceof WorkflowHookCalled called) {
      return Optional.of(called.source().policy() + ": " + called.source().hook());
    }
    if (event instanceof OptionalInputAbsent absent) return Optional.of(absent.key());
    if (event instanceof InputAdapterSupplied supplied) {
      return Optional.of(
          supplied.step() + " gets " + supplied.key() + " from " + supplied.adapter());
    }
    if (event instanceof JourneySucceeded succeeded) {
      if (succeeded.decidedBy() instanceof DecidingHook hook) {
        return Optional.of("FinishWorkflow from " + hook.policy());
      }
      return Optional.empty();
    }
    return Optional.empty();
  }

  /** What the log says about a step given up: which policy gave it up, and why, when one did. */
  static String givenUp(StepGivenUp event) {
    if (event.cause() instanceof GiveUpCause.FailWorkflow failed) {
      return event.step()
          + ": FailWorkflow from "
          + failed.decidedBy().policy()
          + ", "
          + failed.reason().code();
    }
    return event.step() + ": " + event.cause().getCause();
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
 * The shop's workflow: it looks the order up, and finishes an order already completed. Otherwise it
 * works out what the order costs, then charges it, repairing a charge that fails and auditing one
 * that succeeds, then ships, and shipping may be tried once more. Once a journey has ended, the
 * customer is told, through the channel the shop configures.
 */
@Workflow(
    name = "orders",
    reporters = {Log.class, Desk.class},
    policies = {Notify.class})
@WithStep(
    step = Lookup.class,
    name = "lookup",
    policies = {AlreadyCompleted.class})
@WithStep(step = Price.class, name = "price")
@WithStep(
    step = Charge.class,
    name = "charge",
    rescue = RenewCard.class,
    policies = {Audit.class, Outage.class})
@WithStep(
    step = Ship.class,
    name = "ship",
    retryBudget = 1,
    policies = {FailedShipping.class})
final class Orders implements ItineraWorkflow, Messages, Refunds {
  private final String channel;

  Orders(String channel) {
    this.channel = channel;
  }

  @Override
  public void tell(String customer, String message) {
    System.out.println(channel + " to " + customer + ": " + message);
  }

  @Override
  public void refund(String receipt, long amount) {
    System.out.println("the bank gives " + amount + " back on receipt " + receipt);
  }

  /** The bank, as the payment client of the journey. */
  @Provider(key = "payments")
  public Provided<Payments> bank() {
    return provide(new Bank(2026));
  }

  /** A carrier for every attempt of a step: the main carrier first, the backup carrier after. */
  @Provider(perAttempt = true, key = "carrier")
  public Provided<Carrier> carriers(Attempt attempt) {
    return attempt.isFirst() ? provide(new MainCarrier()) : provide(new BackupCarrier());
  }

  /**
   * Gives the shipping the weight of the order's parcel, as the warehouse describes it. Every other
   * input of the shipping is read from the data bag.
   */
  @InputAdapter(steps = {"ship"})
  public Supplied parcels(@Key String key, @FromWorkflow Parcel parcel) {
    return key.equals("weight") ? supply(parcel.weight()) : fromDataBag();
  }

  /** Names each journey after its order. */
  @IdGenerator
  public String orderNumber(DataBag data) {
    String order = data.read("order", TypeToken.of(String.class)).orNull();
    if (order == null) throw new IllegalStateException("an order without a number");
    return order;
  }

  /** The order book, which has the orders completed before today. */
  @Provider(key = "orderBook")
  public Provided<OrderBook> ledger() {
    return provide(new Ledger(List.of("A-13")));
  }
}

/**
 * What the shop knows about an order when its journey starts. An order with no card leaves it out.
 */
record NewOrder(
    String order,
    String customer,
    long price,
    Optional<Long> discount,
    Parcel parcel,
    Optional<String> card,
    boolean giftCard) {
  /** An order without a discount. */
  NewOrder(
      String order, String customer, long price, Parcel parcel, String card, boolean giftCard) {
    this(order, customer, price, Optional.empty(), parcel, Optional.of(card), giftCard);
  }

  /** An order without a discount that isn't a gift card. */
  NewOrder(String order, String customer, long price, Parcel parcel, String card) {
    this(order, customer, price, parcel, card, false);
  }

  /** An order without a discount or a card, that isn't a gift card. */
  NewOrder(String order, String customer, long price, Parcel parcel) {
    this(order, customer, price, Optional.empty(), parcel, Optional.empty(), false);
  }
}

/** What the shop reads back once the journey has ended. */
record Order(Optional<String> receipt, Optional<String> tracking, Optional<String> audit) {}

/**
 * How the warehouse describes an order's parcel: its weight in kg, and its length, width and height
 * in cm.
 */
record Parcel(long weight, List<Long> size) {}

final class Main {
  private Main() {}

  /** Says how an order's journey ended, and what explains it. */
  static void followUp(JourneyResult result) {
    JourneyStatus status = result.status();
    if (status instanceof JourneyStatus.Succeeded) {
      var data = result.output(TypeToken.of(Order.class));
      var receipt = data.receipt().orElse("none");
      var tracking = data.tracking().orElse("none");
      System.out.println(
          "the journey succeeded, with receipt " + receipt + " and tracking " + tracking);
      System.out.println("audit: " + data.audit().orElse("none"));
    } else if (status instanceof JourneyStatus.Failed failed) {
      var data = result.output(TypeToken.of(Order.class));
      var receipt = data.receipt().orElse("none");
      System.out.println(
          "the journey failed (" + why(failed.failure()) + "), with receipt " + receipt);
    } else if (status instanceof JourneyStatus.Aborted aborted) {
      System.out.println("the journey was aborted (" + whatWentWrong(aborted.abort()) + ")");
    }
  }

  /** The cause of a failure, with the reason a step gave for it when it gave one. */
  static String why(Failure failure) {
    if (failure.reason() == null) return failure.cause().toString();
    return failure.cause() + ": " + failure.reason().code();
  }

  /** The reason of an abort, with the key that was missing when that is what happened. */
  static String whatWentWrong(Abort abort) {
    if (abort.details() instanceof RequiredDataMissing missingData
        && missingData.missing() instanceof MissingData.Key missing) {
      return abort.reason() + ": " + missing.key();
    }
    return abort.reason().toString();
  }

  public static void main(String[] args) throws Refusal {
    var max = new NewOrder("A-11", "Max", 20, new Parcel(1, List.of(20L, 10L, 5L)));
    // Each order: its number, its customer, its price, its discount if it has one, its parcel, as
    // its weight in kg and its size in cm, the card if it has one, and whether it is a gift card.
    var orders =
        List.of(
            new NewOrder(
                "A-1",
                "Ann",
                40,
                Optional.of(10L),
                new Parcel(2, List.of(30L, 20L, 10L)),
                Optional.of("ann-2028"),
                false),
            new NewOrder("A-7", "Gil", 45, new Parcel(60, List.of(120L, 80L, 60L)), "gil-2027"),
            new NewOrder("A-8", "Jo", 30, new Parcel(1, List.of(20L, 15L, 5L)), "jo-2024"),
            new NewOrder("A-9", "Kim", 25, new Parcel(0, List.of(0L, 0L, 0L)), "kim-2027", true),
            new NewOrder("A-10", "Lee", 35, new Parcel(3, List.of(40L, 30L, 20L)), "lee-2019"),
            max);
    var executor = new LocalExecutor();
    for (var order : orders) {
      System.out.println(order.customer() + "'s order:");
      var instance = new Orders("email").instance().initialData(order).create();
      followUp(executor.run(instance));
    }
  }
}
