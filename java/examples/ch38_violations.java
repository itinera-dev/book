// A workflow put together wrongly by hand: the program compiles, and building the workflow throws
// every violation at once. Fixed, the workflow is built and listed.

package ch38_violations;

import java.util.List;
import org.itineradev.itinera.policy.ItineraStepPolicy;
import org.itineradev.itinera.policy.OnStepSuccess;
import org.itineradev.itinera.policy.OnSuccess;
import org.itineradev.itinera.policy.StepPolicy;
import org.itineradev.itinera.policy.StepPolicyDescriptor;
import org.itineradev.itinera.request.Request;
import org.itineradev.itinera.request.Requests;
import org.itineradev.itinera.request.WorkflowDataKind;
import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.value.TypeToken;
import org.itineradev.itinera.workflow.InputAdapterDescriptor;
import org.itineradev.itinera.workflow.StepDescriptor;
import org.itineradev.itinera.workflow.Supplied;
import org.itineradev.itinera.workflow.Sync;
import org.itineradev.itinera.workflow.Violation;
import org.itineradev.itinera.workflow.Violations;
import org.itineradev.itinera.workflow.WorkflowBuilder;
import org.itineradev.itinera.workflow.WorkflowDescriptor;

// ANCHOR: shop
/** Charges an order. */
@Step
final class Charge implements ItineraStep {
  public Outcome run() {
    return succeed();
  }
}

/** Ships an order of the weight it is given. */
@Step
final class Ship implements ItineraStep {
  Ship(@Input long weight) {}

  public Outcome run() {
    return succeed();
  }
}

/** Audits a charge that succeeds. */
@StepPolicy(name = "audit")
final class Audit implements ItineraStepPolicy {
  @OnStepSuccess
  public OnSuccess charged() {
    return keepDefault();
  }
}

/** Keeps the receipt of a charge that succeeds. */
@StepPolicy(name = "receipts")
final class Receipts implements ItineraStepPolicy {
  @OnStepSuccess
  public OnSuccess charged() {
    return keepDefault();
  }
}

/** The shop whose orders the workflow takes. */
final class Shop {}

// ANCHOR_END: shop

final class Main {
  private Main() {}

  // ANCHOR: parts
  static final Request<WorkflowDataKind, Long> PARCEL =
      Requests.fromWorkflow("parcel", TypeToken.of(Long.class));

  /** The parcels adapter, which gives the step it is attached to the parcel's weight. */
  static InputAdapterDescriptor<Shop, Sync> parcelsFor(String step) {
    return InputAdapterDescriptor.<Shop>builder(
            "parcels", step, (shop, got) -> new Supplied.Value(got.get(PARCEL)))
        .request(PARCEL)
        .build();
  }

  /** The charge, with the policies given. */
  static StepDescriptor<Sync> charge(List<StepPolicyDescriptor<?, Sync>> policies) {
    var charge = StepDescriptor.builder("charge", Charge.class);
    for (var policy : policies) charge.policy(policy);
    return charge.build();
  }

  /** The shipping. */
  static StepDescriptor<Sync> ship() {
    return StepDescriptor.builder("ship", Ship.class).build();
  }

  /** Starts the shop's workflow. */
  static WorkflowBuilder<Shop, Sync> shop() {
    return WorkflowDescriptor.builder(Shop.class, "shop");
  }

  /** Says what one violation is, by the name the specification gives it, and what it concerns. */
  static String detail(Violation violation) {
    if (violation instanceof Violation.DuplicateStepName duplicate) {
      return violation.getNamed() + ": " + duplicate.step();
    }
    if (violation instanceof Violation.StepHookDefinedTwice twice) {
      return violation.getNamed() + ": " + twice.hook() + " on step " + twice.step();
    }
    if (violation instanceof Violation.InputAdapterForUnknownStep unknown) {
      return violation.getNamed() + ": " + unknown.adapter() + " on " + unknown.step();
    }
    return violation.getNamed();
  }

  // ANCHOR_END: parts

  public static void main(String[] args) {
    try {
      // ANCHOR: wrong
      var audit = StepPolicyDescriptor.of(Audit.class);
      var receipts = StepPolicyDescriptor.of(Receipts.class);
      shop()
          .step(charge(List.of(audit, receipts)))
          .step(ship())
          .step(ship())
          .inputAdapter(parcelsFor("shipping"))
          .build();
      // ANCHOR_END: wrong
      // ANCHOR: report
    } catch (Violations refused) {
      System.out.println("the workflow was refused:");
      for (var violation : refused.getViolations()) {
        System.out.println("  " + detail(violation));
      }
    }
    // ANCHOR_END: report
    // ANCHOR: fixed
    var fixed =
        shop()
            .step(charge(List.of(StepPolicyDescriptor.of(Audit.class))))
            .step(ship())
            .inputAdapter(parcelsFor("ship"))
            .build();
    System.out.println("the fixed workflow was built:");
    for (var step : fixed.listing()) {
      System.out.println("  " + step.position() + ". " + step.step());
    }
    // ANCHOR_END: fixed
  }
}
