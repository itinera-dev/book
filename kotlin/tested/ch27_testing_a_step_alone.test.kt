package ch27_testing_a_step_alone

// ANCHOR: tests
import org.itineradev.itinera.journey.testing.Contributions
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Reason
import org.itineradev.itinera.step.RescueOutcome
import org.itineradev.itinera.step.testing.StepEventReporter
import org.itineradev.itinera.value.kotlin.typeToken
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

/** The shop's steps, each built from its inputs and run alone. */
class StepsTest {
    // ANCHOR_END: tests

    // ANCHOR: fakes
    /**
     * A bank for tests: it answers every charge as the test chose, and has replaced every card with
     * one named after its holder that expires in 2029.
     */
    enum class TestBank : Payments {
        CHARGES,
        DECLINES,
        IS_DOWN;

        override fun charge(order: String, card: String, amount: Long): String? =
            when (this) {
                CHARGES -> "R-$order"
                DECLINES -> null
                IS_DOWN -> throw NetworkDown()
            }

        override fun replacement(card: String): String? = "${card.substringBeforeLast('-')}-2029"
    }

    /** A carrier for tests: it books and collects every parcel, or its booking system is down. */
    enum class TestCarrier : Carrier {
        WORKS,
        IS_DOWN;

        override fun book(order: String, weight: Long): String =
            when (this) {
                WORKS -> "T-$order"
                IS_DOWN -> throw BookingFailed()
            }

        override fun collects(weight: Long): Boolean = true
    }

    // ANCHOR_END: fakes

    // ANCHOR: charge
    /** Ann's charge of 40 to her card. */
    private fun annsCharge(): Charge =
        Charge(order = "A-1", amount = 40, card = "ann-2028", giftCard = false)

    @Test
    fun `a charge the bank takes gives back the bank's receipt`() {
        val contributions = Contributions()
        val outcome = annsCharge().run(TestBank.CHARGES, contributions.contributor)
        assertEquals(Outcome.success(), outcome)
        assertEquals("R-A-1", contributions.read("receipt", typeToken<String>()).orNull())
    }

    @Test
    fun `a charge the bank declines fails with card-expired and no receipt`() {
        val contributions = Contributions()
        val outcome = annsCharge().run(TestBank.DECLINES, contributions.contributor)
        assertEquals(Outcome.failure("card-expired"), outcome)
        assertNull(contributions.read("receipt", typeToken<String>()).orNull())
    }

    @Test
    fun `a charge lets the error of a bank that is down escape`() {
        val contributions = Contributions()
        assertThrows<NetworkDown> { annsCharge().run(TestBank.IS_DOWN, contributions.contributor) }
    }

    // ANCHOR_END: charge

    // ANCHOR: rescue
    @Test
    fun `renewing an expired card gives back the card that replaced it`() {
        val contributions = Contributions()
        val renew = RenewCard(card = "jo-2024", reason = Reason("card-expired"))
        val outcome = renew.run(TestBank.CHARGES, contributions.contributor)
        assertEquals(RescueOutcome.success(), outcome)
        assertEquals("jo-2029", contributions.read("card", typeToken<String>()).orNull())
    }

    @Test
    fun `a failure that is not an expired card is not repairable`() {
        val contributions = Contributions()
        val renew = RenewCard(card = "max-2027", reason = Reason("nothing-to-charge"))
        val outcome = renew.run(TestBank.CHARGES, contributions.contributor)
        assertEquals(RescueOutcome.failure("not-repairable"), outcome)
    }

    // ANCHOR_END: rescue

    // ANCHOR: ship
    /** An order's shipping, for a parcel of this weight. */
    private fun shipping(weight: Long): Ship =
        Ship(order = "A-1", weight = weight, receipt = "R-A-1", giftCard = false)

    @Test
    fun `a parcel of 2 kg leaves with its receipt`() {
        val contributions = Contributions()
        val events = StepEventReporter()
        val outcome = shipping(2).run(TestCarrier.WORKS, contributions.contributor, events.reporter)
        assertEquals(Outcome.success(), outcome)
        assertEquals("the parcel leaves with receipt R-A-1", events.emitted.first().message)
    }

    @Test
    fun `a parcel of 90 kg fails with over-parcel-limit before the carrier is called`() {
        val contributions = Contributions()
        val events = StepEventReporter()
        val outcome =
            shipping(90).run(TestCarrier.IS_DOWN, contributions.contributor, events.reporter)
        assertEquals(Outcome.failure("over-parcel-limit"), outcome)
    }
    // ANCHOR_END: ship
}
