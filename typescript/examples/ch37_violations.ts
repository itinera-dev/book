// Mistakes caught before a journey: a shop whose workflow is put together
// with two mistakes, which the decorators find as the class is defined.

import { type Contributor, contributor } from "itinera/journey";
import {
  fromStep,
  onStepSuccess,
  type OnSuccess,
  StepPolicy,
  stepPolicy,
} from "itinera/policy";
import { type Outcome, Step, step } from "itinera/step";
import { integer } from "itinera/value";
import { Violations, withStep, Workflow, workflow } from "itinera/workflow";

// ANCHOR: shop
/** Charges an order, and gives back the amount it charged. */
@step()
class Charge extends Step {
  override run(@contributor() contributor: Contributor): Outcome {
    contributor.contribute("charged", 40);
    return this.succeed();
  }
}

/** Ships an order. */
@step()
class Ship extends Step {
  override run(): Outcome {
    return this.succeed();
  }
}

/** Audits every successful charge. */
@stepPolicy({ name: "audit" })
class Audit extends StepPolicy {
  @onStepSuccess()
  audited(@fromStep("charged", integer) charged: number): OnSuccess {
    console.log(`audit: charged ${String(charged)}`);
    return this.keepDefault();
  }
}

/** Writes a receipt for every successful charge. */
@stepPolicy({ name: "receipts" })
class Receipts extends StepPolicy {
  @onStepSuccess()
  written(@fromStep("charged", integer) charged: number): OnSuccess {
    console.log(`receipt: R-${String(charged)}`);
    return this.keepDefault();
  }
}
// ANCHOR_END: shop

// ANCHOR: wrong
/**
 * Defines the shop's workflow, with both policies attached to the charge and
 * the shipping listed twice.
 */
function defineOrders(): void {
  @workflow({ name: "orders" })
  @withStep(Charge, { name: "charge", policies: [Audit, Receipts] })
  @withStep(Ship, { name: "ship" })
  @withStep(Ship, { name: "ship", retryBudget: 1 })
  class Orders extends Workflow {}
}
// ANCHOR_END: wrong

// ANCHOR: refused
try {
  defineOrders();
} catch (error) {
  if (!(error instanceof Violations)) throw error;
  console.log("the workflow was refused:");
  console.log(error.message);
}
// ANCHOR_END: refused
