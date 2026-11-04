package ch28_testing_a_hook_or_an_adapter_alone_async

import kotlinx.coroutines.runBlocking
import org.itineradev.itinera.journey.FailureCause
import org.itineradev.itinera.journey.testing.Contributions
import org.itineradev.itinera.policy.OnFailure
import org.itineradev.itinera.step.Reason
import org.itineradev.itinera.step.StepName
import org.itineradev.itinera.value.kotlin.typeToken
import org.itineradev.itinera.workflow.Supplied
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

/** The shop's hooks and its input adapter, each called alone. */
class HooksTest {
    // ANCHOR: fakes
    /** Messages for tests: it keeps each message it is asked to send, with the customer's name. */
    class TestMessages : Messages {
        val told = mutableListOf<String>()

        override suspend fun tell(customer: String, message: String) {
            told += "$customer: $message"
        }
    }

    /** Refunds for tests: it keeps each refund it is asked to give, with its receipt. */
    class TestRefunds : Refunds {
        val given = mutableListOf<String>()

        override suspend fun refund(receipt: String, amount: Long) {
            given += "$amount back on $receipt"
        }
    }

    // ANCHOR_END: fakes

    // ANCHOR: failed_shipping
    /**
     * What `beforeRetrying` decides for Ned's shipping, which failed with this reason after he paid
     * 50, and the refunds it asked for.
     */
    private suspend fun beforeRetrying(reason: Reason?): Pair<OnFailure, List<String>> {
        val refunds = TestRefunds()
        val contributions = Contributions()
        val decision =
            FailedShipping()
                .beforeRetrying(reason, "R-A-12", 50, refunds, contributions.contributor)
        return decision to refunds.given
    }

    @Test
    fun `failedShipping refunds and gives up a shipping that failed with over-parcel-limit`() =
        runBlocking<Unit> {
            val (decision, refunds) = beforeRetrying(Reason("over-parcel-limit"))
            assertEquals("no-carrier-can-take-it", decision.failsWorkflow?.code)
            assertEquals(listOf("50 back on R-A-12"), refunds)
        }

    @Test
    fun `failedShipping lets a shipping that failed with too-heavy be retried`() =
        runBlocking<Unit> {
            val (decision, refunds) = beforeRetrying(Reason("too-heavy"))
            assertTrue(decision.keepsDefault)
            assertEquals(emptyList<String>(), refunds)
        }

    @Test
    fun `failedShipping lets a shipping that ended in an error be retried`() =
        runBlocking<Unit> {
            val (decision, refunds) = beforeRetrying(null)
            assertTrue(decision.keepsDefault)
            assertEquals(emptyList<String>(), refunds)
        }

    @Test
    fun `failedShipping refunds a shipping the executor gave up`() =
        runBlocking<Unit> {
            val refunds = TestRefunds()
            val contributions = Contributions()
            val decision =
                FailedShipping()
                    .givenUp(
                        FailureCause.RETRIES_EXHAUSTED,
                        "R-A-14",
                        20,
                        refunds,
                        contributions.contributor,
                    )
            assertTrue(decision.keepsDefault)
            assertEquals(listOf("20 back on R-A-14"), refunds.given)
            assertEquals(
                "20 back on R-A-14: retries exhausted",
                contributions.read("refund", typeToken<String>()).orNull(),
            )
        }

    // ANCHOR_END: failed_shipping

    // ANCHOR: audit
    @Test
    fun `audit writes the step, the amount and the receipt`() =
        runBlocking<Unit> {
            val contributions = Contributions()
            Audit().charged(StepName.of("charge"), 30, "R-A-1", contributions.contributor)
            assertEquals(
                "charge took 30, receipt R-A-1",
                contributions.read("audit", typeToken<String>()).orNull(),
            )
        }

    // ANCHOR_END: audit

    // ANCHOR: notify
    @Test
    fun `notify gives a gift card's code and says sorry for a failed order`() =
        runBlocking<Unit> {
            val messages = TestMessages()
            Notify().succeeded("Kim", "A-9", tracking = null, code = "G-A-9", messages)
            Notify().failed("Lee", "A-10", messages)
            assertEquals(
                listOf(
                    "Kim: Order A-9 is paid: your gift card's code is G-A-9.",
                    "Lee: Sorry, order A-10 could not be completed.",
                ),
                messages.told,
            )
        }

    // ANCHOR_END: notify

    // ANCHOR: adapter
    /** Gil's parcel, as the warehouse describes it. */
    private fun gilsParcel(): Parcel = Parcel(weight = 60, size = listOf(120, 80, 60))

    @Test
    fun `parcels gives the shipping the parcel's weight`() =
        runBlocking<Unit> {
            val weight = Orders(channel = "email").parcels("weight", gilsParcel())
            assertEquals(Supplied.Value(60L), weight)
        }

    @Test
    fun `parcels leaves every other input to the data bag`() =
        runBlocking<Unit> {
            val receipt = Orders(channel = "email").parcels("receipt", gilsParcel())
            assertEquals(Supplied.FromDataBag, receipt)
        }
    // ANCHOR_END: adapter
}
