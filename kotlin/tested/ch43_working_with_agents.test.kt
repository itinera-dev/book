package ch43_working_with_agents

import java.io.OutputStream
import java.io.PrintStream
import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.instance.testing.TestInstanceBuilder
import org.itineradev.itinera.journey.DataBag
import org.itineradev.itinera.journey.kotlin.read
import org.itineradev.itinera.policy.FromWorkflow
import org.itineradev.itinera.report.testing.RecordingReporter
import org.itineradev.itinera.service.Provided
import org.itineradev.itinera.service.Provider
import org.itineradev.itinera.workflow.IdGenerator
import org.itineradev.itinera.workflow.InputAdapter
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.Key
import org.itineradev.itinera.workflow.Supplied
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow
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

    /** A bank for tests: it charges every card, or its network is down. */
    enum class TestBank : Payments {
        CHARGES,
        IS_DOWN;

        override fun charge(order: String, card: String, amount: Long): String? =
            when (this) {
                CHARGES -> "R-$order"
                IS_DOWN -> throw NetworkDown()
            }

        override fun replacement(card: String): String? = null
    }

    /** A carrier for tests: it books every parcel, and collects every parcel or none. */
    enum class TestCarrier : Carrier {
        COLLECTS_ALL,
        COLLECTS_NONE;

        override fun book(order: String, weight: Long): String = "T-$order"

        override fun collects(weight: Long): Boolean = this == COLLECTS_ALL
    }

    // ANCHOR: paths
    /** What the shop knows about an order of 30 with a parcel of this weight. */
    private fun orderOf(order: String, weight: Long): NewOrder =
        NewOrder(
            order,
            "Ann",
            price = 30,
            parcel = Parcel(weight, size = listOf(20, 15, 5)),
            card = "ann-2028",
        )

    /**
     * Runs an order of 30 with a parcel of this weight, through the shop's real steps and policies,
     * with the test's bank and carriers in place of the shop's, and gives back its journey's path.
     */
    private fun pathOf(
        order: String,
        weight: Long,
        bank: TestBank,
        carrier: TestCarrier,
    ): List<String> {
        val recorder = RecordingReporter()
        val instance =
            TestInstanceBuilder.of(Orders(channel = "email"))
                .initialData(orderOf(order, weight))
                .reporter(recorder)
                .fakeProvider("bank", Payments::class.java) { Provided.Instance(bank) }
                .fakeProvider("carriers", Carrier::class.java) { Provided.Instance(carrier) }
                .create()
        LocalExecutor().run(instance)
        return recorder.path
    }

    /** Runs the same order as `pathOf`, through the proposed workflow. */
    private fun proposedPathOf(
        order: String,
        weight: Long,
        bank: TestBank,
        carrier: TestCarrier,
    ): List<String> {
        val recorder = RecordingReporter()
        val instance =
            TestInstanceBuilder.of(Proposed())
                .initialData(orderOf(order, weight))
                .reporter(recorder)
                .fakeProvider("bank", Payments::class.java) { Provided.Instance(bank) }
                .fakeProvider("carriers", Carrier::class.java) { Provided.Instance(carrier) }
                .create()
        LocalExecutor().run(instance)
        return recorder.path
    }

    // ANCHOR_END: paths

    @Test
    fun `an order the bank charges and the carrier takes is shipped`() {
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
    fun `a parcel over the limit is given up after its first attempt`() {
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
    fun `a charge whose bank is down is given up and never rescued`() {
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
    fun `a parcel no carrier collects is tried once more, then given up`() {
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

    // ANCHOR: compared
    @Test
    fun `the proposed flow keeps two approved paths and changes two`() {
        assertEquals(
            pathOf("A-1", 2, TestBank.CHARGES, TestCarrier.COLLECTS_ALL),
            proposedPathOf("A-1", 2, TestBank.CHARGES, TestCarrier.COLLECTS_ALL),
        )
        assertEquals(
            pathOf("A-3", 2, TestBank.IS_DOWN, TestCarrier.COLLECTS_ALL),
            proposedPathOf("A-3", 2, TestBank.IS_DOWN, TestCarrier.COLLECTS_ALL),
        )
        assertEquals(
            listOf(
                "lookup attempt 1 succeeded",
                "price attempt 1 succeeded",
                "charge attempt 1 succeeded",
                "ship attempt 1 failed: over-parcel-limit",
                "ship given up: failure",
                "journey failed at ship: over-parcel-limit",
            ),
            proposedPathOf("A-2", 90, TestBank.CHARGES, TestCarrier.COLLECTS_ALL),
        )
        assertEquals(
            listOf(
                "lookup attempt 1 succeeded",
                "price attempt 1 succeeded",
                "charge attempt 1 succeeded",
                "ship attempt 1 failed: too-heavy",
                "ship given up: failure",
                "journey failed at ship: too-heavy",
            ),
            proposedPathOf("A-4", 60, TestBank.CHARGES, TestCarrier.COLLECTS_NONE),
        )
    }
    // ANCHOR_END: compared
}

// ANCHOR: proposed
/**
 * The shop's workflow as an agent proposes to change it: a parcel the carrier won't collect is
 * given up at once, without trying the backup carrier.
 */
@Workflow(name = "orders", reporters = [Log::class, Desk::class], policies = [Notify::class])
@WithStep(Lookup::class, name = "lookup", policies = [AlreadyCompleted::class])
@WithStep(Price::class, name = "price")
@WithStep(
    Charge::class,
    name = "charge",
    rescue = RenewCard::class,
    policies = [Audit::class, Outage::class],
)
@WithStep(Ship::class, name = "ship", policies = [FailedShipping::class])
class Proposed : ItineraWorkflow, Messages, Refunds {
    override fun tell(customer: String, message: String) {
        println("email to $customer: $message")
    }

    override fun refund(receipt: String, amount: Long) {
        println("the bank gives $amount back on receipt $receipt")
    }

    /** The bank, as the payment client of the journey. */
    @Provider(key = "payments") fun bank(): Provided<Payments> = provide(Bank(thisYear = 2026))

    /** The main carrier, for every attempt of a step. */
    @Provider(perAttempt = true, key = "carrier")
    fun carriers(): Provided<Carrier> = provide(MainCarrier())

    /**
     * Gives the shipping the weight of the order's parcel, as the warehouse describes it. Every
     * other input of the shipping is read from the data bag.
     */
    @InputAdapter(steps = ["ship"])
    fun parcels(@Key key: String, @FromWorkflow parcel: Parcel): Supplied =
        if (key == "weight") supply(parcel.weight) else fromDataBag()

    /** Names each journey after its order. */
    @IdGenerator
    fun orderNumber(data: DataBag): String =
        data.read<String>("order").orNull() ?: error("an order without a number")

    /** The order book, which has the orders completed before today. */
    @Provider(key = "orderBook") fun ledger(): Provided<OrderBook> = provide(Ledger(listOf("A-13")))
}

// ANCHOR_END: proposed
