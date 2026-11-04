package ch29_testing_the_flow_with_scripted_steps_async

import java.io.OutputStream
import java.io.PrintStream
import kotlinx.coroutines.runBlocking
import org.itineradev.itinera.executor.testing.ScriptedRun
import org.itineradev.itinera.executor.testing.scripted
import org.itineradev.itinera.instance.WorkflowInstance
import org.itineradev.itinera.journey.FailureCause
import org.itineradev.itinera.journey.JourneyResult
import org.itineradev.itinera.testing.Mistake
import org.itineradev.itinera.testing.TestMistake
import org.itineradev.itinera.value.kotlin.typeToken
import org.itineradev.itinera.workflow.Async
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

/** The shop's flow, run with scripts saying how its steps' attempts end. */
class FlowTest {
    // ANCHOR: quiet
    private val console = System.out

    @BeforeEach
    fun quiet() {
        System.setOut(PrintStream(OutputStream.nullOutputStream()))
    }

    @AfterEach
    fun speak() {
        System.setOut(console)
    }

    // ANCHOR_END: quiet

    // ANCHOR: order
    /** An instance of the shop's workflow for an order of 30, with this card and this parcel. */
    private fun order(order: String, card: String, weight: Long): WorkflowInstance<Async> =
        Orders(channel = "email")
            .instance()
            .initialData(
                NewOrder(
                    order,
                    "Ann",
                    price = 30,
                    parcel = Parcel(weight, size = listOf(20, 15, 5)),
                    card = card,
                )
            )
            .create()

    // ANCHOR_END: order

    // ANCHOR: rescued
    @Test
    fun `a declined card is renewed and the charge tried again with it`() =
        runBlocking<Unit> {
            val run =
                scripted(order("A-1", "ann-2024", 1))
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
                    .run()
            assertTrue(run.result.isSucceeded)
            val retried = run.step("charge").attempt(2)
            assertEquals("ann-2029", retried?.input("card", typeToken<String>())?.orNull())
        }

    // ANCHOR_END: rescued

    // ANCHOR: given_up
    /** The reason a policy failed the journey with, if one did. */
    private fun failedByAPolicy(result: JourneyResult): String? =
        result.failure?.takeIf { it.cause == FailureCause.FAIL_WORKFLOW }?.reason?.code

    /**
     * Runs an order of 60 kg whose shipping fails on attempt 1 with this reason, and would succeed
     * on attempt 2.
     */
    private suspend fun shippingThatFailsWith(reason: String): ScriptedRun =
        scripted(order("A-2", "ann-2028", 60))
            .step("charge")
            .willContribute("receipt", "R-A-2")
            .willSucceed()
            .andStep("ship")
            .willFail(reason)
            .onAttempt(2)
            .willContribute("tracking", "B-A-2")
            .willSucceed()
            .run()

    @Test
    fun `a shipping that fails with too-heavy is tried again`() =
        runBlocking<Unit> {
            val run = shippingThatFailsWith("too-heavy")
            assertEquals(2, run.step("ship").attempts.size)
            assertTrue(run.result.isSucceeded)
        }

    @Test
    fun `a shipping that fails with over-parcel-limit is given up after attempt 1`() =
        runBlocking<Unit> {
            val run = shippingThatFailsWith("over-parcel-limit")
            assertEquals(1, run.step("ship").attempts.size)
            assertEquals("no-carrier-can-take-it", failedByAPolicy(run.result))
        }

    @Test
    fun `a scripted shipping is still built with the weight the adapter supplies`() =
        runBlocking<Unit> {
            val run = shippingThatFailsWith("too-heavy")
            val first = run.step("ship").attempt(1)
            assertEquals(60L, first?.input("weight", typeToken<Long>())?.orNull())
        }

    // ANCHOR_END: given_up

    // ANCHOR: outage
    @Test
    fun `a charge that ends in an error is given up and never rescued`() =
        runBlocking<Unit> {
            val run =
                scripted(order("A-3", "ann-2028", 1))
                    .step("charge")
                    .willTerminateAbnormally("the payment network is down")
                    .onAttempt(2)
                    .willContribute("receipt", "R-A-3")
                    .willSucceed()
                    .andStep("charge_rescue")
                    .willSucceed()
                    .run()
            assertTrue(run.step("charge_rescue").attempts.isEmpty())
            assertEquals("payment-network-down", failedByAPolicy(run.result))
        }

    // ANCHOR_END: outage

    // ANCHOR: mistake
    @Test
    fun `a script that leaves out the shipping's attempt 2 is refused`() =
        runBlocking<Unit> {
            val script = scripted(order("A-4", "ann-2028", 1)).step("ship").willFail("too-heavy")
            val mistake = assertThrows<TestMistake> { script.run() }
            assertEquals(listOf(Mistake.AttemptLeftOut("ship", 2)), mistake.problems)
        }
    // ANCHOR_END: mistake
}
