package ch43_working_with_agents;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.io.OutputStream;
import java.io.PrintStream;
import java.util.List;
import java.util.Optional;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.instance.testing.TestInstanceBuilder;
import org.itineradev.itinera.journey.DataBag;
import org.itineradev.itinera.policy.FromWorkflow;
import org.itineradev.itinera.report.testing.RecordingReporter;
import org.itineradev.itinera.service.Provided;
import org.itineradev.itinera.service.Provider;
import org.itineradev.itinera.value.TypeToken;
import org.itineradev.itinera.workflow.IdGenerator;
import org.itineradev.itinera.workflow.InputAdapter;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.Key;
import org.itineradev.itinera.workflow.Supplied;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayNameGeneration;
import org.junit.jupiter.api.DisplayNameGenerator;
import org.junit.jupiter.api.Test;

/** Whole journeys of the shop, with a fake bank and carrier, compared with the approved paths. */
@DisplayNameGeneration(DisplayNameGenerator.ReplaceUnderscores.class)
final class JourneysTest {
  private final PrintStream console = System.out;

  @BeforeEach
  void quiet() {
    System.setOut(new PrintStream(OutputStream.nullOutputStream()));
  }

  @AfterEach
  void speak() {
    System.setOut(console);
  }

  /** A bank for tests: it charges every card, or its network is down. */
  enum TestBank implements Payments {
    CHARGES,
    IS_DOWN;

    @Override
    public Optional<String> charge(String order, String card, long amount) throws NetworkDown {
      return switch (this) {
        case CHARGES -> Optional.of("R-" + order);
        case IS_DOWN -> throw new NetworkDown();
      };
    }

    @Override
    public Optional<String> replacement(String card) {
      return Optional.empty();
    }
  }

  /** A carrier for tests: it books every parcel, and collects every parcel or none. */
  enum TestCarrier implements Carrier {
    COLLECTS_ALL,
    COLLECTS_NONE;

    @Override
    public String book(String order, long weight) {
      return "T-" + order;
    }

    @Override
    public boolean collects(long weight) {
      return this == COLLECTS_ALL;
    }
  }

  // ANCHOR: paths
  /** What the shop knows about an order of 30 with a parcel of this weight. */
  private static NewOrder orderOf(String order, long weight) {
    return new NewOrder(order, "Ann", 30, new Parcel(weight, List.of(20L, 15L, 5L)), "ann-2028");
  }

  /**
   * Runs an order of 30 with a parcel of this weight, through the shop's real steps and policies,
   * with the test's bank and carriers in place of the shop's, and gives back its journey's path.
   */
  private static List<String> pathOf(String order, long weight, TestBank bank, TestCarrier carrier)
      throws Refusal {
    var recorder = new RecordingReporter();
    var instance =
        TestInstanceBuilder.of(new Orders("email"))
            .initialData(orderOf(order, weight))
            .reporter(recorder)
            .fakeProvider("bank", Payments.class, () -> new Provided.Instance<>(bank))
            .fakeProvider("carriers", Carrier.class, () -> new Provided.Instance<>(carrier))
            .create();
    new LocalExecutor().run(instance);
    return recorder.getPath();
  }

  /** Runs the same order as {@code pathOf}, through the proposed workflow. */
  private static List<String> proposedPathOf(
      String order, long weight, TestBank bank, TestCarrier carrier) throws Refusal {
    var recorder = new RecordingReporter();
    var instance =
        TestInstanceBuilder.of(new Proposed())
            .initialData(orderOf(order, weight))
            .reporter(recorder)
            .fakeProvider("bank", Payments.class, () -> new Provided.Instance<>(bank))
            .fakeProvider("carriers", Carrier.class, () -> new Provided.Instance<>(carrier))
            .create();
    new LocalExecutor().run(instance);
    return recorder.getPath();
  }

  // ANCHOR_END: paths

  @Test
  void an_order_the_bank_charges_and_the_carrier_takes_is_shipped() throws Refusal {
    assertEquals(
        List.of(
            "lookup attempt 1 succeeded",
            "price attempt 1 succeeded",
            "charge attempt 1 succeeded",
            "ship attempt 1 succeeded",
            "journey succeeded"),
        pathOf("A-1", 2, TestBank.CHARGES, TestCarrier.COLLECTS_ALL));
  }

  @Test
  void a_parcel_over_the_limit_is_given_up_after_its_first_attempt() throws Refusal {
    assertEquals(
        List.of(
            "lookup attempt 1 succeeded",
            "price attempt 1 succeeded",
            "charge attempt 1 succeeded",
            "ship attempt 1 failed: over-parcel-limit",
            "ship given up: FailWorkflow from failedShipping",
            "journey failed at ship: no-carrier-can-take-it"),
        pathOf("A-2", 90, TestBank.CHARGES, TestCarrier.COLLECTS_ALL));
  }

  @Test
  void a_charge_whose_bank_is_down_is_given_up_and_never_rescued() throws Refusal {
    assertEquals(
        List.of(
            "lookup attempt 1 succeeded",
            "price attempt 1 succeeded",
            "charge attempt 1 terminated abnormally: the payment network is down",
            "charge given up: FailWorkflow from outage",
            "journey failed at charge: payment-network-down"),
        pathOf("A-3", 2, TestBank.IS_DOWN, TestCarrier.COLLECTS_ALL));
  }

  @Test
  void a_parcel_no_carrier_collects_is_tried_once_more_then_given_up() throws Refusal {
    assertEquals(
        List.of(
            "lookup attempt 1 succeeded",
            "price attempt 1 succeeded",
            "charge attempt 1 succeeded",
            "ship attempt 1 failed: too-heavy",
            "ship retrying",
            "ship attempt 2 failed: too-heavy",
            "ship given up: retries exhausted",
            "journey failed at ship: too-heavy"),
        pathOf("A-4", 60, TestBank.CHARGES, TestCarrier.COLLECTS_NONE));
  }

  // ANCHOR: compared
  @Test
  void the_proposed_flow_keeps_two_approved_paths_and_changes_two() throws Refusal {
    assertEquals(
        pathOf("A-1", 2, TestBank.CHARGES, TestCarrier.COLLECTS_ALL),
        proposedPathOf("A-1", 2, TestBank.CHARGES, TestCarrier.COLLECTS_ALL));
    assertEquals(
        pathOf("A-3", 2, TestBank.IS_DOWN, TestCarrier.COLLECTS_ALL),
        proposedPathOf("A-3", 2, TestBank.IS_DOWN, TestCarrier.COLLECTS_ALL));
    assertEquals(
        List.of(
            "lookup attempt 1 succeeded",
            "price attempt 1 succeeded",
            "charge attempt 1 succeeded",
            "ship attempt 1 failed: over-parcel-limit",
            "ship given up: failure",
            "journey failed at ship: over-parcel-limit"),
        proposedPathOf("A-2", 90, TestBank.CHARGES, TestCarrier.COLLECTS_ALL));
    assertEquals(
        List.of(
            "lookup attempt 1 succeeded",
            "price attempt 1 succeeded",
            "charge attempt 1 succeeded",
            "ship attempt 1 failed: too-heavy",
            "ship given up: failure",
            "journey failed at ship: too-heavy"),
        proposedPathOf("A-4", 60, TestBank.CHARGES, TestCarrier.COLLECTS_NONE));
  }
  // ANCHOR_END: compared
}

// ANCHOR: proposed
/**
 * The shop's workflow as an agent proposes to change it: a parcel the carrier won't collect is
 * given up at once, without trying the backup carrier.
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
    policies = {FailedShipping.class})
final class Proposed implements ItineraWorkflow, Messages, Refunds {
  @Override
  public void tell(String customer, String message) {
    System.out.println("email to " + customer + ": " + message);
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

  /** The main carrier, for every attempt of a step. */
  @Provider(perAttempt = true, key = "carrier")
  public Provided<Carrier> carriers() {
    return provide(new MainCarrier());
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

// ANCHOR_END: proposed
