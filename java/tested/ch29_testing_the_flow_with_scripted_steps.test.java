package ch29_testing_the_flow_with_scripted_steps;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.OutputStream;
import java.io.PrintStream;
import java.util.List;
import java.util.Optional;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.executor.testing.Scripted;
import org.itineradev.itinera.executor.testing.ScriptedRun;
import org.itineradev.itinera.instance.WorkflowInstance;
import org.itineradev.itinera.journey.Failure;
import org.itineradev.itinera.journey.FailureCause;
import org.itineradev.itinera.journey.JourneyResult;
import org.itineradev.itinera.testing.Mistake;
import org.itineradev.itinera.testing.TestMistake;
import org.itineradev.itinera.value.TypeToken;
import org.itineradev.itinera.workflow.Sync;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayNameGeneration;
import org.junit.jupiter.api.DisplayNameGenerator;
import org.junit.jupiter.api.Test;

/** The shop's flow, run with scripts saying how its steps' attempts end. */
@DisplayNameGeneration(DisplayNameGenerator.ReplaceUnderscores.class)
final class FlowTest {
  // ANCHOR: quiet
  private final PrintStream console = System.out;

  @BeforeEach
  void quiet() {
    System.setOut(new PrintStream(OutputStream.nullOutputStream()));
  }

  @AfterEach
  void speak() {
    System.setOut(console);
  }

  // ANCHOR_END: quiet

  // ANCHOR: order
  /** An instance of the shop's workflow for an order of 30, with this card and this parcel. */
  private static WorkflowInstance<Sync> order(String order, String card, long weight) {
    var parcel = new Parcel(weight, List.of(20L, 15L, 5L));
    return new Orders("email")
        .instance()
        .initialData(new NewOrder(order, "Ann", 30, parcel, card))
        .create();
  }

  // ANCHOR_END: order

  // ANCHOR: rescued
  @Test
  void a_declined_card_is_renewed_and_the_charge_tried_again_with_it() throws Refusal {
    var run =
        Scripted.scripted(order("A-1", "ann-2024", 1))
            .step("charge")
            .willFail("card-expired")
            .onAttempt(2)
            .willContribute("receipt", "R-A-1")
            .willSucceed()
            .andStep("charge_rescue")
            .willContribute("card", "ann-2029")
            .willSucceed()
            .andStep("ship")
            .willContribute("tracking", "T-A-1")
            .willSucceed()
            .run();
    assertTrue(run.getResult().isSucceeded());
    var retried = run.step("charge").attempt(2);
    assertEquals("ann-2029", retried.input("card", TypeToken.of(String.class)).orNull());
  }

  // ANCHOR_END: rescued

  // ANCHOR: given_up
  /** The reason a policy failed the journey with, if one did. */
  private static Optional<String> failedByAPolicy(JourneyResult result) {
    Failure failure = result.failure();
    if (failure == null || failure.cause() != FailureCause.FAIL_WORKFLOW) return Optional.empty();
    return Optional.of(failure.reason().code());
  }

  /**
   * Runs an order of 60 kg whose shipping fails on attempt 1 with this reason, and would succeed on
   * attempt 2.
   */
  private static ScriptedRun shippingThatFailsWith(String reason) throws Refusal {
    return Scripted.scripted(order("A-2", "ann-2028", 60))
        .step("charge")
        .willContribute("receipt", "R-A-2")
        .willSucceed()
        .andStep("ship")
        .willFail(reason)
        .onAttempt(2)
        .willContribute("tracking", "B-A-2")
        .willSucceed()
        .run();
  }

  @Test
  void a_shipping_that_fails_with_too_heavy_is_tried_again() throws Refusal {
    var run = shippingThatFailsWith("too-heavy");
    assertEquals(2, run.step("ship").getAttempts().size());
    assertTrue(run.getResult().isSucceeded());
  }

  @Test
  void a_shipping_that_fails_with_over_parcel_limit_is_given_up_after_attempt_1() throws Refusal {
    var run = shippingThatFailsWith("over-parcel-limit");
    assertEquals(1, run.step("ship").getAttempts().size());
    assertEquals(Optional.of("no-carrier-can-take-it"), failedByAPolicy(run.getResult()));
  }

  @Test
  void a_scripted_shipping_is_still_built_with_the_weight_the_adapter_supplies() throws Refusal {
    var run = shippingThatFailsWith("too-heavy");
    var first = run.step("ship").attempt(1);
    assertEquals(60L, first.input("weight", TypeToken.of(Long.class)).orNull());
  }

  // ANCHOR_END: given_up

  // ANCHOR: outage
  @Test
  void a_charge_that_ends_in_an_error_is_given_up_and_never_rescued() throws Refusal {
    var run =
        Scripted.scripted(order("A-3", "ann-2028", 1))
            .step("charge")
            .willTerminateAbnormally("the payment network is down")
            .onAttempt(2)
            .willContribute("receipt", "R-A-3")
            .willSucceed()
            .andStep("charge_rescue")
            .willSucceed()
            .run();
    assertTrue(run.step("charge_rescue").getAttempts().isEmpty());
    assertEquals(Optional.of("payment-network-down"), failedByAPolicy(run.getResult()));
  }

  // ANCHOR_END: outage

  // ANCHOR: mistake
  @Test
  void a_script_that_leaves_out_the_shipping_attempt_2_is_refused() {
    var script = Scripted.scripted(order("A-4", "ann-2028", 1)).step("ship").willFail("too-heavy");
    var mistake = assertThrows(TestMistake.class, script::run);
    assertEquals(List.of(new Mistake.AttemptLeftOut("ship", 2)), mistake.getProblems());
  }
  // ANCHOR_END: mistake
}
