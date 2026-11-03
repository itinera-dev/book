package ch27_testing_a_step_alone;

// ANCHOR: tests
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.Optional;
import org.itineradev.itinera.journey.testing.Contributions;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Reason;
import org.itineradev.itinera.step.RescueOutcome;
import org.itineradev.itinera.step.testing.StepEventReporter;
import org.itineradev.itinera.value.TypeToken;
import org.junit.jupiter.api.DisplayNameGeneration;
import org.junit.jupiter.api.DisplayNameGenerator;
import org.junit.jupiter.api.Test;

/** The shop's steps, each built from its inputs and run alone. */
@DisplayNameGeneration(DisplayNameGenerator.ReplaceUnderscores.class)
final class StepsTest {
  // ANCHOR_END: tests

  // ANCHOR: fakes
  /**
   * A bank for tests: it answers every charge as the test chose, and has replaced every card with
   * one named after its holder that expires in 2029.
   */
  enum TestBank implements Payments {
    CHARGES,
    DECLINES,
    IS_DOWN;

    @Override
    public Optional<String> charge(String order, String card, long amount) throws NetworkDown {
      return switch (this) {
        case CHARGES -> Optional.of("R-" + order);
        case DECLINES -> Optional.empty();
        case IS_DOWN -> throw new NetworkDown();
      };
    }

    @Override
    public Optional<String> replacement(String card) {
      return Optional.of(card.substring(0, card.lastIndexOf('-')) + "-2029");
    }
  }

  /** A carrier for tests: it books and collects every parcel, or its booking system is down. */
  enum TestCarrier implements Carrier {
    WORKS,
    IS_DOWN;

    @Override
    public String book(String order, long weight) throws BookingFailed {
      return switch (this) {
        case WORKS -> "T-" + order;
        case IS_DOWN -> throw new BookingFailed();
      };
    }

    @Override
    public boolean collects(long weight) {
      return true;
    }
  }

  // ANCHOR_END: fakes

  // ANCHOR: charge
  /** Ann's charge of 40 to her card. */
  private static Charge annsCharge() {
    return new Charge("A-1", 40, "ann-2028", false);
  }

  @Test
  void a_charge_the_bank_takes_gives_back_the_bank_receipt() throws NetworkDown {
    var contributions = new Contributions();
    var outcome = annsCharge().run(TestBank.CHARGES, contributions.getContributor());
    assertEquals(Outcome.success(), outcome);
    assertEquals("R-A-1", contributions.read("receipt", TypeToken.of(String.class)).orNull());
  }

  @Test
  void a_charge_the_bank_declines_fails_with_card_expired_and_no_receipt() throws NetworkDown {
    var contributions = new Contributions();
    var outcome = annsCharge().run(TestBank.DECLINES, contributions.getContributor());
    assertEquals(Outcome.failure("card-expired"), outcome);
    assertNull(contributions.read("receipt", TypeToken.of(String.class)).orNull());
  }

  @Test
  void a_charge_lets_the_error_of_a_bank_that_is_down_escape() {
    var contributions = new Contributions();
    assertThrows(
        NetworkDown.class,
        () -> annsCharge().run(TestBank.IS_DOWN, contributions.getContributor()));
  }

  // ANCHOR_END: charge

  // ANCHOR: rescue
  @Test
  void renewing_an_expired_card_gives_back_the_card_that_replaced_it() {
    var contributions = new Contributions();
    var renew = new RenewCard("jo-2024", new Reason("card-expired"));
    var outcome = renew.run(TestBank.CHARGES, contributions.getContributor());
    assertEquals(RescueOutcome.success(), outcome);
    assertEquals("jo-2029", contributions.read("card", TypeToken.of(String.class)).orNull());
  }

  @Test
  void a_failure_that_is_not_an_expired_card_is_not_repairable() {
    var contributions = new Contributions();
    var renew = new RenewCard("max-2027", new Reason("nothing-to-charge"));
    var outcome = renew.run(TestBank.CHARGES, contributions.getContributor());
    assertEquals(RescueOutcome.failure("not-repairable"), outcome);
  }

  // ANCHOR_END: rescue

  // ANCHOR: ship
  /** An order's shipping, for a parcel of this weight. */
  private static Ship shipping(long weight) {
    return new Ship("A-1", weight, "R-A-1", false);
  }

  @Test
  void a_parcel_of_2_kg_leaves_with_its_receipt() throws BookingFailed {
    var contributions = new Contributions();
    var events = new StepEventReporter();
    var outcome =
        shipping(2).run(TestCarrier.WORKS, contributions.getContributor(), events.getReporter());
    assertEquals(Outcome.success(), outcome);
    assertEquals("the parcel leaves with receipt R-A-1", events.getEmitted().get(0).message());
  }

  @Test
  void a_parcel_of_90_kg_fails_with_over_parcel_limit_before_the_carrier_is_called()
      throws BookingFailed {
    var contributions = new Contributions();
    var events = new StepEventReporter();
    var outcome =
        shipping(90).run(TestCarrier.IS_DOWN, contributions.getContributor(), events.getReporter());
    assertEquals(Outcome.failure("over-parcel-limit"), outcome);
  }
  // ANCHOR_END: ship
}
