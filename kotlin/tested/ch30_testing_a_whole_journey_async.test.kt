package ch30_testing_a_whole_journey_async

import java.io.OutputStream
import java.io.PrintStream
import kotlinx.coroutines.runBlocking
import org.itineradev.itinera.executor.async.AsyncLocalExecutor
import org.itineradev.itinera.instance.testing.AsyncTestInstanceBuilder
import org.itineradev.itinera.report.testing.AsyncRecordingReporter
import org.itineradev.itinera.service.Provided
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

/** Whole journeys of the shop, with a fake bank and carrier, compared with the approved paths. */
class JourneysTest {
    private val console = System.out

    @BeforeEach
    fun quiet() {
        System.setOut(PrintStream(OutputStream.nullOutputStream()))
    }

    @AfterEach
    fun speak() {
        System.setOut(console)
    }

    // ANCHOR: fakes
    /** A bank for tests: it charges every card, or its network is down. */
    enum class TestBank : Payments {
        CHARGES,
        IS_DOWN;

        override suspend fun charge(order: String, card: String, amount: Long): String? =
            when (this) {
                CHARGES -> "R-$order"
                IS_DOWN -> throw NetworkDown()
            }

        override suspend fun replacement(card: String): String? = null
    }

    /** A carrier for tests: it books every parcel, and collects every parcel or none. */
    enum class TestCarrier : Carrier {
        COLLECTS_ALL,
        COLLECTS_NONE;

        override suspend fun book(order: String, weight: Long): String = "T-$order"

        override suspend fun collects(weight: Long): Boolean = this == COLLECTS_ALL
    }

    // ANCHOR_END: fakes

    // ANCHOR: path
    /**
     * Runs an order of 30 with a parcel of this weight, through the shop's real steps and policies,
     * with the test's bank and carriers in place of the shop's, and gives back its journey's path.
     */
    private suspend fun pathOf(
        order: String,
        weight: Long,
        bank: TestBank,
        carrier: TestCarrier,
    ): List<String> {
        val recorder = AsyncRecordingReporter()
        val instance =
            AsyncTestInstanceBuilder.of(Orders(channel = "email"))
                .initialData(
                    NewOrder(
                        order,
                        "Ann",
                        price = 30,
                        parcel = Parcel(weight, size = listOf(20, 15, 5)),
                        card = "ann-2028",
                    )
                )
                .reporter(recorder)
                .fakeProvider("bank", Payments::class.java) { Provided.Instance(bank) }
                .fakeProvider("carriers", Carrier::class.java) { Provided.Instance(carrier) }
                .create()
        AsyncLocalExecutor().run(instance)
        return recorder.path
    }

    // ANCHOR_END: path

    // ANCHOR: approved
    @Test
    fun `an order the bank charges and the carrier takes is shipped`() =
        runBlocking<Unit> {
            assertEquals(
                listOf(
                    "lookup attempt 1 succeeded",
                    "price attempt 1 succeeded",
                    "charge attempt 1 succeeded",
                    "ship attempt 1 succeeded",
                    "journey succeeded",
                ),
                pathOf("A-1", 2, TestBank.CHARGES, TestCarrier.COLLECTS_ALL),
            )
        }

    @Test
    fun `a parcel over the limit is given up after its first attempt`() =
        runBlocking<Unit> {
            assertEquals(
                listOf(
                    "lookup attempt 1 succeeded",
                    "price attempt 1 succeeded",
                    "charge attempt 1 succeeded",
                    "ship attempt 1 failed: over-parcel-limit",
                    "ship given up: FailWorkflow from failedShipping",
                    "journey failed at ship: no-carrier-can-take-it",
                ),
                pathOf("A-2", 90, TestBank.CHARGES, TestCarrier.COLLECTS_ALL),
            )
        }

    @Test
    fun `a charge whose bank is down is given up and never rescued`() =
        runBlocking<Unit> {
            assertEquals(
                listOf(
                    "lookup attempt 1 succeeded",
                    "price attempt 1 succeeded",
                    "charge attempt 1 terminated abnormally: the payment network is down",
                    "charge given up: FailWorkflow from outage",
                    "journey failed at charge: payment-network-down",
                ),
                pathOf("A-3", 2, TestBank.IS_DOWN, TestCarrier.COLLECTS_ALL),
            )
        }

    @Test
    fun `a parcel no carrier collects is tried once more, then given up`() =
        runBlocking<Unit> {
            assertEquals(
                listOf(
                    "lookup attempt 1 succeeded",
                    "price attempt 1 succeeded",
                    "charge attempt 1 succeeded",
                    "ship attempt 1 failed: too-heavy",
                    "ship retrying",
                    "ship attempt 2 failed: too-heavy",
                    "ship given up: retries exhausted",
                    "journey failed at ship: too-heavy",
                ),
                pathOf("A-4", 60, TestBank.CHARGES, TestCarrier.COLLECTS_NONE),
            )
        }
    // ANCHOR_END: approved
}
