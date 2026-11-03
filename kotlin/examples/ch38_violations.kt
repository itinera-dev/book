// A workflow put together wrongly by hand: the program compiles, and building the workflow throws
// every violation at once. Fixed, the workflow is built and listed.
package ch38_violations

import org.itineradev.itinera.policy.ItineraStepPolicy
import org.itineradev.itinera.policy.OnStepSuccess
import org.itineradev.itinera.policy.OnSuccess
import org.itineradev.itinera.policy.StepPolicy
import org.itineradev.itinera.policy.StepPolicyDescriptor
import org.itineradev.itinera.request.Requests
import org.itineradev.itinera.step.Input
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.value.kotlin.typeToken
import org.itineradev.itinera.workflow.InputAdapterDescriptor
import org.itineradev.itinera.workflow.StepDescriptor
import org.itineradev.itinera.workflow.Supplied
import org.itineradev.itinera.workflow.Sync
import org.itineradev.itinera.workflow.Violation
import org.itineradev.itinera.workflow.Violations
import org.itineradev.itinera.workflow.WorkflowBuilder
import org.itineradev.itinera.workflow.WorkflowDescriptor

// ANCHOR: shop
/** Charges an order. */
@Step
class Charge : ItineraStep {
    fun run(): Outcome = succeed()
}

/** Ships an order of the weight it is given. */
@Step
class Ship(@Input private val weight: Long) : ItineraStep {
    fun run(): Outcome = succeed()
}

/** Audits a charge that succeeds. */
@StepPolicy(name = "audit")
class Audit : ItineraStepPolicy {
    @OnStepSuccess fun charged(): OnSuccess = keepDefault()
}

/** Keeps the receipt of a charge that succeeds. */
@StepPolicy(name = "receipts")
class Receipts : ItineraStepPolicy {
    @OnStepSuccess fun charged(): OnSuccess = keepDefault()
}

/** The shop whose orders the workflow takes. */
class Shop

val PARCEL = Requests.fromWorkflow("parcel", typeToken<Long>())

/** The parcels adapter, which gives the step it is attached to the parcel's weight. */
fun parcelsFor(step: String): InputAdapterDescriptor<Shop, Sync> =
    InputAdapterDescriptor.builder<Shop>("parcels", step) { _, got -> Supplied.Value(got[PARCEL]) }
        .request(PARCEL)
        .build()

/** The charge, with the policies given. */
fun charge(vararg policies: StepPolicyDescriptor<*, Sync>): StepDescriptor<Sync> =
    policies
        .fold(StepDescriptor.builder("charge", Charge::class.java)) { charge, policy ->
            charge.policy(policy)
        }
        .build()

/** The shipping. */
fun ship(): StepDescriptor<Sync> = StepDescriptor.builder("ship", Ship::class.java).build()

/** Starts the shop's workflow. */
fun shop(): WorkflowBuilder<Shop, Sync> = WorkflowDescriptor.builder(Shop::class.java, "shop")

/** Says what one violation is, by the name the specification gives it, and what it concerns. */
fun detail(violation: Violation): String =
    when (violation) {
        is Violation.DuplicateStepName -> "${violation.named}: ${violation.step}"
        is Violation.StepHookDefinedTwice ->
            "${violation.named}: ${violation.hook} on step ${violation.step}"
        is Violation.InputAdapterForUnknownStep ->
            "${violation.named}: ${violation.adapter} on ${violation.step}"
        else -> "${violation.named}"
    }

// ANCHOR_END: shop

fun main() {
    try {
        // ANCHOR: wrong
        val audit = StepPolicyDescriptor.of(Audit::class.java)
        val receipts = StepPolicyDescriptor.of(Receipts::class.java)
        shop()
            .step(charge(audit, receipts))
            .step(ship())
            .step(ship())
            .inputAdapter(parcelsFor("shipping"))
            .build()
        // ANCHOR_END: wrong
    } catch (refused: Violations) {
        // ANCHOR: report
        println("the workflow was refused:")
        for (violation in refused.violations) {
            println("  ${detail(violation)}")
        }
        // ANCHOR_END: report
    }
    // ANCHOR: fixed
    val fixed =
        shop()
            .step(charge(StepPolicyDescriptor.of(Audit::class.java)))
            .step(ship())
            .inputAdapter(parcelsFor("ship"))
            .build()
    println("the fixed workflow was built:")
    for (step in fixed.listing()) {
        println("  ${step.position}. ${step.step}")
    }
    // ANCHOR_END: fixed
}
