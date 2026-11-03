// Using a step in another workflow: orders collected at the shop's counter run the lookup, the
// pricing and the charge of the delivered orders, with the counter's terminal and no shipping.

package ch42_using_a_step_in_another_workflow;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;
import org.itineradev.itinera.event.AttemptStarted;
import org.itineradev.itinera.event.ContributionCommitted;
import org.itineradev.itinera.event.ContributionsDiscarded;
import org.itineradev.itinera.event.DataOverwritten;
import org.itineradev.itinera.event.DecidingHook;
import org.itineradev.itinera.event.Event;
import org.itineradev.itinera.event.GiveUpCause;
import org.itineradev.itinera.event.GlobalMetadataSet;
import org.itineradev.itinera.event.InputAdapterSupplied;
import org.itineradev.itinera.event.JourneyFailed;
import org.itineradev.itinera.event.JourneyStarted;
import org.itineradev.itinera.event.JourneySucceeded;
import org.itineradev.itinera.event.OptionalInputAbsent;
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
import org.itineradev.itinera.journey.Contributor;
import org.itineradev.itinera.journey.DataBag;
import org.itineradev.itinera.journey.FailureCause;
import org.itineradev.itinera.journey.JourneyId;
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
import org.itineradev.itinera.workflow.ListedService;
import org.itineradev.itinera.workflow.ListedStep;
import org.itineradev.itinera.workflow.Supplied;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;
import org.itineradev.itinera.workflow.WorkflowDescriptor;

/**
 * A payment client: it charges an order's amount to a card, and gives back a receipt, or nothing
 * when the card has expired.
 */
interface Payments {
  Optional<String> charge(String order, String card, long amount) throws NetworkDown;
}

/** A card registry: it says which card replaced one, or nothing when none did. */
interface Cards {
  Optional<String> replacement(String card);
}

// ANCHOR: terminal
/**
 * The shop counter's card terminal, faked: it declines a card that expired before this year, and
 * prints what it does. Its receipts start with {@code C}.
 */
final class Terminal implements Payments {
  private final int thisYear;

  Terminal(int thisYear) {
    this.thisYear = thisYear;
  }

  @Override
  public Optional<String> charge(String order, String card, long amount) {
    if (Bank.expiry(card) < thisYear) {
      System.out.println("the counter's terminal declines card " + card + ": it has expired");
      return Optional.empty();
    }
    System.out.println("the counter's terminal charges " + amount + " for order " + order);
    return Optional.of("C-" + order);
  }
}

// ANCHOR_END: terminal

/**
 * The bank's own library: it connects to the bank's card registry for a region, and hands back its
 * client.
 */
final class BankLibrary {
  private BankLibrary() {}

  static Cards cardRegistry(String region) {
    return new Registry(region, 2026);
  }
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

  /** The year a card expires: the part of it after its last {@code -}. */
  static int expiry(String card) {
    return Integer.parseInt(card.substring(card.lastIndexOf('-') + 1));
  }
}

/** The bank's card registry for a region, faked: it prints what it does. */
final class Registry implements Cards {
  private final String region;
  private final int thisYear;

  Registry(String region, int thisYear) {
    this.region = region;
    this.thisYear = thisYear;
  }

  /**
   * A card that expired in the last three years was replaced by its holder's card that expires in
   * three years. An older one was not.
   */
  @Override
  public Optional<String> replacement(String card) {
    if (Bank.expiry(card) + 3 < thisYear) {
      System.out.println("the card registry in " + region + " says nothing replaced card " + card);
      return Optional.empty();
    }
    String replacement = card.substring(0, card.lastIndexOf('-')) + "-" + (thisYear + 3);
    System.out.println(
        "the card registry in "
            + region
            + " says card "
            + card
            + " was replaced by "
            + replacement);
    return Optional.of(replacement);
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
 * the gift card's code when the order is a gift card. It gives every reporter the receipt as soon
 * as the bank gives it.
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

  public Outcome run(@Service Payments payments, Contributor contributor, StepReporter reporter)
      throws NetworkDown {
    if (amount <= 0) return fail("nothing-to-charge");
    Optional<String> receipt = payments.charge(order, card, amount);
    if (receipt.isEmpty()) return fail("card-expired");
    reporter.setGlobalMetadata("receipt", receipt.orElseThrow());
    contributor.contribute("receipt", receipt.orElseThrow());
    if (giftCard) contributor.contribute("code", "G-" + order);
    return succeed();
  }
}

/**
 * Repairs a charge that failed because its card had expired: it asks the card registry for the card
 * that replaced it, and gives it back as the order's card.
 */
@RescueStep
final class RenewCard implements ItineraRescueStep {
  private final String expired;
  private final Reason reason;

  RenewCard(@Input("card") String expired, Reason reason) {
    this.expired = expired;
    this.reason = reason;
  }

  public RescueOutcome run(@Service("cards") Cards registry, Contributor contributor) {
    if (!reason.code().equals("card-expired")) return fail("not-repairable");
    Optional<String> replacement = registry.replacement(expired);
    if (replacement.isEmpty()) return fail("not-replaced");
    contributor.contribute("card", replacement.orElseThrow());
    return succeed();
  }
}

/**
 * Writes an audit line for every step it is attached to that succeeds: the journey, the attempt
 * that took the money, the amount the order was charged, and the receipt the step gave back.
 */
@StepPolicy(name = "audit")
final class Audit implements ItineraStepPolicy {
  @OnStepSuccess
  public OnSuccess charged(
      StepName step,
      JourneyId journey,
      @FromWorkflow long amount,
      Attempt attempt,
      @FromStep String receipt,
      Contributor contributor) {
    contributor.contribute(
        "audit",
        journey
            + ": "
            + step
            + " took "
            + amount
            + " on attempt "
            + attempt
            + ", receipt "
            + receipt);
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
 * nor a rescue can mend that. Since the bank may have taken the money before its network failed, it
 * writes an audit line with the error, for the accountants to check with the bank.
 */
@StepPolicy(name = "outage")
final class Outage implements ItineraStepPolicy {
  @OnStepAbnormalTermination
  public OnFailure endedInError(
      JourneyId journey,
      StepName step,
      Attempt attempt,
      Throwable error,
      @FromWorkflow long amount,
      Contributor contributor) {
    contributor.contribute(
        "audit",
        journey
            + ": "
            + step
            + " ended in an error on attempt "
            + attempt
            + " ("
            + error.getMessage()
            + "); check whether the bank took "
            + amount);
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

/** The logging service's error when there is no customer's name to file a journey's lines under. */
final class NoCustomer extends RuntimeException {
  private static final long serialVersionUID = 1L;

  NoCustomer() {
    super("the logging service files no line without a customer's name");
  }
}

/** The logging service's error: it can't take a line. */
final class LoggingDown extends RuntimeException {
  private static final long serialVersionUID = 1L;

  LoggingDown() {
    super("the logging service is down");
  }
}

/**
 * The shop's logging service, faked: it prints each line it takes. It is down when the line saying
 * that order A-15's charge succeeded reaches it.
 */
final class LoggingService {
  void fileUnder(String customer) {
    if (customer.isEmpty()) throw new NoCustomer();
  }

  void write(String line) {
    if (line.contains(" A-15 ") && line.endsWith(" step_succeeded charge")) {
      throw new LoggingDown();
    }
    System.out.println(line);
  }
}

/**
 * The shop's log: it writes a line for every event of a journey to the logging service, starting
 * with the workflow's name, the journey's ID, the customer's name and, once the bank has given it,
 * the receipt. It has the service file the journey's lines under the customer's name, and a name or
 * a line the service refuses fails it.
 */
final class Log implements Reporter {
  private final LoggingService service = new LoggingService();
  private final Map<String, Object> metadata = new HashMap<>();

  @Override
  public void setGlobalMetadata(String key, Object value) {
    if (key.equals("customer")) service.fileUnder(value instanceof String name ? name : "");
    metadata.put(key, value);
  }

  @Override
  public void report(Event event) {
    String line =
        "log "
            + event.getWorkflow()
            + " "
            + event.getJourneyId()
            + " "
            + whose()
            + " #"
            + event.getSequence()
            + " "
            + event.getKind();
    service.write(line + detail(event).map(text -> " " + text).orElse(""));
  }

  /** The customer the journey is for and, once the bank has given it, the charge's receipt. */
  private String whose() {
    String customer = metadata.get("customer") instanceof String name ? name : "someone";
    if (metadata.get("receipt") instanceof String receipt) {
      return "for " + customer + ", receipt " + receipt;
    }
    return "for " + customer;
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
              + provided.implementation()
              + supplied(provided.data()));
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
    if (event instanceof GlobalMetadataSet set) return Optional.of(set.key());
    return Optional.empty();
  }

  /** The data a provider supplied with a service, when it is text. */
  static String supplied(Object data) {
    return data instanceof String text ? ", with " + text : "";
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

  /** The bank's card registry in Europe, for the charge's rescue step only. */
  @Provider(steps = {"charge_rescue"})
  public Provided<Cards> cards() {
    return provide(BankLibrary.cardRegistry("eu"), "eu");
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

// ANCHOR: counter
/**
 * The counter's workflow: it looks the order up, and finishes an order already completed. Otherwise
 * it works out what the order costs, then charges it at the counter's terminal and audits the
 * charge. The customer takes the goods away, so nothing is shipped, and a card the terminal
 * declines is handed back rather than repaired.
 */
@Workflow(
    name = "counter",
    reporters = {Log.class})
@WithStep(
    step = Lookup.class,
    name = "lookup",
    policies = {AlreadyCompleted.class})
@WithStep(step = Price.class, name = "price")
@WithStep(
    step = Charge.class,
    name = "charge",
    policies = {Audit.class})
final class Counter implements ItineraWorkflow {
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

  /** The counter's terminal, as the payment client of the journey. */
  @Provider(key = "payments")
  public Provided<Payments> terminal() {
    return provide(new Terminal(2026));
  }
}

/** What the shop knows about an order collected at the counter when its journey starts. */
record CounterOrder(
    String order, long price, Optional<Long> discount, String card, boolean giftCard) {
  /** An order with a discount, that isn't a gift card. */
  CounterOrder(String order, long price, long discount, String card) {
    this(order, price, Optional.of(discount), card, false);
  }

  /** An order without a discount, that isn't a gift card. */
  CounterOrder(String order, long price, String card) {
    this(order, price, Optional.empty(), card, false);
  }
}

// ANCHOR_END: counter

/**
 * How the warehouse describes an order's parcel: its weight in kg, and its length, width and height
 * in cm.
 */
record Parcel(long weight, List<Long> size) {}

final class Main {
  private Main() {}

  /** A service in the listing: its key, and the provider that serves it. */
  static String service(ListedService listed) {
    return listed.key() + " from " + listed.provider();
  }

  /** The services of a listing's entry, separated by commas. */
  static String services(List<ListedService> listed) {
    return listed.stream().map(Main::service).collect(Collectors.joining(", "));
  }

  /**
   * One entry of the listing: a step's position and name, its policies, its adapter, its services
   * with the provider that serves each, and its rescue step with its services.
   */
  static String entry(ListedStep listed) {
    var entry = new StringBuilder(listed.position() + ". " + listed.step());
    if (!listed.policies().isEmpty()) {
      var policies = listed.policies().stream().map(String::valueOf);
      entry.append(", with ").append(policies.collect(Collectors.joining(", ")));
    }
    if (listed.adapter() != null) entry.append("; adapted by ").append(listed.adapter());
    if (!listed.services().isEmpty()) {
      entry.append("; services: ").append(services(listed.services()));
    }
    var rescue = listed.rescue();
    if (rescue != null) {
      entry.append("; rescued by ").append(rescue.step());
      entry.append(", services: ").append(services(rescue.services()));
    }
    return entry.toString();
  }

  // ANCHOR: follow_up
  /** What the counter reads back once the journey has ended. */
  record Collected(Optional<String> receipt, Optional<String> audit) {}

  /** Says how a journey at the counter ended. */
  static void followUp(JourneyResult result) {
    var data = result.output(TypeToken.of(Collected.class));
    JourneyStatus status = result.status();
    if (status instanceof JourneyStatus.Succeeded) {
      System.out.println("the journey succeeded, with receipt " + data.receipt().orElse("none"));
      System.out.println("audit: " + data.audit().orElse("none"));
    } else if (status instanceof JourneyStatus.Failed failed) {
      Reason reason = failed.failure().reason();
      String why = reason == null ? failed.failure().cause().toString() : reason.code();
      System.out.println("the journey failed: " + why + ", hand the card back");
    } else if (status instanceof JourneyStatus.Aborted aborted) {
      System.out.println("the journey was aborted (" + aborted.abort().reason() + ")");
    }
  }

  // ANCHOR_END: follow_up

  public static void main(String[] args) throws Refusal {
    // ANCHOR: listings
    System.out.println("orders:");
    for (var listed : WorkflowDescriptor.of(Orders.class).listing()) {
      System.out.println(entry(listed));
    }
    System.out.println("counter:");
    for (var listed : WorkflowDescriptor.of(Counter.class).listing()) {
      System.out.println(entry(listed));
    }
    // ANCHOR_END: listings
    // ANCHOR: run
    var executor = new LocalExecutor();
    // Each order collected at the counter: its customer, and its number, price, discount if it has
    // one, and card.
    var orders =
        List.of(
            Map.entry("Nia", new CounterOrder("A-17", 18, 3, "nia-2028")),
            Map.entry("Omar", new CounterOrder("A-18", 12, "omar-2023")));
    for (var entry : orders) {
      var customer = entry.getKey();
      var order = entry.getValue();
      System.out.println(customer + "'s order " + order.order() + ", at the counter:");
      var instance =
          new Counter()
              .instance()
              .reportersGlobalMetadata("customer", customer)
              .initialData(order)
              .create();
      followUp(executor.run(instance));
    }
    // ANCHOR_END: run
  }
}
