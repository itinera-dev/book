package ch30_testing_a_whole_journey;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.io.OutputStream;
import java.io.PrintStream;
import java.util.List;
import java.util.Optional;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.instance.testing.TestInstanceBuilder;
import org.itineradev.itinera.report.testing.RecordingReporter;
import org.itineradev.itinera.service.Provided;
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

  // ANCHOR: fakes
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

  // ANCHOR_END: fakes

  // ANCHOR: path
  /**
   * Runs an order of 30 with a parcel of this weight, through the shop's real steps and policies,
   * with the test's bank and carriers in place of the shop's, and gives back its journey's path.
   */
  private static List<String> pathOf(String order, long weight, TestBank bank, TestCarrier carrier)
      throws Refusal {
    var recorder = new RecordingReporter();
    var parcel = new Parcel(weight, List.of(20L, 15L, 5L));
    var instance =
        TestInstanceBuilder.of(new Orders("email"))
            .initialData(new NewOrder(order, "Ann", 30, parcel, "ann-2028"))
            .reporter(recorder)
            .fakeProvider("bank", Payments.class, () -> new Provided.Instance<>(bank))
            .fakeProvider("carriers", Carrier.class, () -> new Provided.Instance<>(carrier))
            .create();
    new LocalExecutor().run(instance);
    return recorder.getPath();
  }

  // ANCHOR_END: path

  // ANCHOR: approved
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
  // ANCHOR_END: approved
}
