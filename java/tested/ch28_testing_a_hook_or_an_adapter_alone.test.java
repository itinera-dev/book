package ch28_testing_a_hook_or_an_adapter_alone;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.itineradev.itinera.journey.FailureCause;
import org.itineradev.itinera.journey.testing.Contributions;
import org.itineradev.itinera.policy.OnFailure;
import org.itineradev.itinera.step.Reason;
import org.itineradev.itinera.step.StepName;
import org.itineradev.itinera.value.TypeToken;
import org.itineradev.itinera.workflow.Supplied;
import org.junit.jupiter.api.DisplayNameGeneration;
import org.junit.jupiter.api.DisplayNameGenerator;
import org.junit.jupiter.api.Test;

/** The shop's hooks and its input adapter, each called alone. */
@DisplayNameGeneration(DisplayNameGenerator.ReplaceUnderscores.class)
final class HooksTest {
  // ANCHOR: fakes
  /** Messages for tests: it keeps each message it is asked to send, with the customer's name. */
  static final class TestMessages implements Messages {
    final List<String> told = new ArrayList<>();

    @Override
    public void tell(String customer, String message) {
      told.add(customer + ": " + message);
    }
  }

  /** Refunds for tests: it keeps each refund it is asked to give, with its receipt. */
  static final class TestRefunds implements Refunds {
    final List<String> given = new ArrayList<>();

    @Override
    public void refund(String receipt, long amount) {
      given.add(amount + " back on " + receipt);
    }
  }

  // ANCHOR_END: fakes

  // ANCHOR: failed_shipping
  /** What {@code beforeRetrying} decided, and the refunds it asked for. */
  record Retrying(OnFailure decision, List<String> refunds) {}

  /**
   * What {@code beforeRetrying} decides for Ned's shipping, which failed this way after he paid 50.
   */
  private static Retrying beforeRetrying(Optional<Reason> reason) {
    var refunds = new TestRefunds();
    var contributions = new Contributions();
    var decision =
        new FailedShipping()
            .beforeRetrying(reason, "R-A-12", 50, refunds, contributions.getContributor());
    return new Retrying(decision, refunds.given);
  }

  @Test
  void failedShipping_refunds_and_gives_up_a_shipping_that_failed_with_over_parcel_limit() {
    var retrying = beforeRetrying(Optional.of(new Reason("over-parcel-limit")));
    assertEquals("no-carrier-can-take-it", retrying.decision().getFailsWorkflow().code());
    assertEquals(List.of("50 back on R-A-12"), retrying.refunds());
  }

  @Test
  void failedShipping_lets_a_shipping_that_failed_with_too_heavy_be_retried() {
    var retrying = beforeRetrying(Optional.of(new Reason("too-heavy")));
    assertTrue(retrying.decision().getKeepsDefault());
    assertEquals(List.of(), retrying.refunds());
  }

  @Test
  void failedShipping_lets_a_shipping_that_ended_in_an_error_be_retried() {
    var retrying = beforeRetrying(Optional.empty());
    assertTrue(retrying.decision().getKeepsDefault());
    assertEquals(List.of(), retrying.refunds());
  }

  @Test
  void failedShipping_refunds_a_shipping_the_executor_gave_up() {
    var refunds = new TestRefunds();
    var contributions = new Contributions();
    var decision =
        new FailedShipping()
            .givenUp(
                FailureCause.RETRIES_EXHAUSTED,
                "R-A-14",
                20,
                refunds,
                contributions.getContributor());
    assertTrue(decision.getKeepsDefault());
    assertEquals(List.of("20 back on R-A-14"), refunds.given);
    assertEquals(
        "20 back on R-A-14: retries exhausted",
        contributions.read("refund", TypeToken.of(String.class)).orNull());
  }

  // ANCHOR_END: failed_shipping

  // ANCHOR: audit
  @Test
  void audit_writes_the_step_the_amount_and_the_receipt() {
    var contributions = new Contributions();
    new Audit().charged(StepName.of("charge"), 30, "R-A-1", contributions.getContributor());
    assertEquals(
        "charge took 30, receipt R-A-1",
        contributions.read("audit", TypeToken.of(String.class)).orNull());
  }

  // ANCHOR_END: audit

  // ANCHOR: notify
  @Test
  void notify_gives_a_gift_card_code_and_says_sorry_for_a_failed_order() {
    var messages = new TestMessages();
    new Notify().succeeded("Kim", "A-9", Optional.empty(), Optional.of("G-A-9"), messages);
    new Notify().failed("Lee", "A-10", messages);
    assertEquals(
        List.of(
            "Kim: Order A-9 is paid: your gift card's code is G-A-9.",
            "Lee: Sorry, order A-10 could not be completed."),
        messages.told);
  }

  // ANCHOR_END: notify

  // ANCHOR: adapter
  /** Gil's parcel, as the warehouse describes it. */
  private static Parcel gilsParcel() {
    return new Parcel(60, List.of(120L, 80L, 60L));
  }

  @Test
  void parcels_gives_the_shipping_the_parcel_weight() {
    var weight = new Orders("email").parcels("weight", gilsParcel());
    assertEquals(new Supplied.Value(60L), weight);
  }

  @Test
  void parcels_leaves_every_other_input_to_the_data_bag() {
    var receipt = new Orders("email").parcels("receipt", gilsParcel());
    assertEquals(Supplied.FromDataBag.INSTANCE, receipt);
  }
  // ANCHOR_END: adapter
}
