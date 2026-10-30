// Runs an executor refuses: an instance handed to it a second time, and an instance handed to it
// while it runs another journey.

package ch24_runs_refused

import java.util.concurrent.CountDownLatch
import kotlin.concurrent.thread
import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.executor.Refusal
import org.itineradev.itinera.journey.JourneyResult
import org.itineradev.itinera.step.Input
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/**
 * The shop's bank, faked: it prints what it charges. Once it is told to hold, it keeps the next
 * charge waiting until it is told to answer.
 */
object Bank {
    @Volatile private var holding = false
    private val charging = CountDownLatch(1)
    private val answered = CountDownLatch(1)

    fun charge(amount: Long) {
        if (holding) {
            charging.countDown()
            answered.await()
        }
        println("the bank charges $amount")
    }

    /** Keeps the next charge waiting until [answer]. */
    fun hold() {
        holding = true
    }

    /** Returns once the charge it holds has started. */
    fun awaitHeldCharge() {
        charging.await()
    }

    /** Lets the charge it holds go on. */
    fun answer() {
        holding = false
        answered.countDown()
    }
}

/** Charges an amount at the faked bank. */
fun bankCharges(amount: Long) {
    Bank.charge(amount)
}

/** Charges an order's price. */
@Step
class Charge(@Input private val price: Long) : ItineraStep {
    fun run(): Outcome {
        if (price <= 0) return fail("nothing-to-charge")
        bankCharges(price)
        return succeed()
    }
}

/** The shop's workflow of chapter 5: it only charges. */
@Workflow(name = "orders") @WithStep(Charge::class, name = "charge") class Orders : ItineraWorkflow

// ANCHOR: run
/** Says how the journey [run] runs ended, or why the run was refused. */
fun said(run: () -> JourneyResult): String =
    try {
        if (run().isSucceeded) "the journey succeeded" else "the journey failed"
    } catch (refusal: Refusal) {
        "refused (${refusal.refused}): ${refusal.message}"
    }

fun main() {
    val executor = LocalExecutor()

    val ann = Orders().instance().data("price", 40L).create()
    println("Ann's order: ${said { executor.run(ann) }}")
    println("Ann's order again: ${said { executor.run(ann) }}")

    Bank.hold()
    val ben = Orders().instance().data("price", 25L).create()
    val bens = thread { println("Ben's order: ${said { executor.run(ben) }}") }
    Bank.awaitHeldCharge()
    val cat = Orders().instance().data("price", 30L).create()
    println("Cat's order, while Ben's runs: ${said { executor.run(cat) }}")
    Bank.answer()
    bens.join()
}
// ANCHOR_END: run
