// A rescue step that tries to skip the step it rescues.

import { LocalExecutor } from "itinera/executor";
import {
  type Outcome,
  RescueStep,
  rescueStep,
  type RescueOutcome,
  Step,
  step,
} from "itinera/step";
import { withStep, Workflow, workflow } from "itinera/workflow";

/** Charges an order, and fails when its card has expired. */
@step()
class Charge extends Step {
  override run(): Outcome {
    return this.fail("card-expired");
  }
}

/** Tries to skip the charge rather than repair it. */
@rescueStep()
class RenewCard extends RescueStep {
  override run(): RescueOutcome {
    return this.skipBecause("no-card-to-renew");
  }
}

/** The shop's workflow. */
@workflow({ name: "orders" })
@withStep(Charge, { name: "charge", rescue: RenewCard })
class Orders extends Workflow {}

const result = await new LocalExecutor().run(Orders.build().instance());
console.log(`the journey ${result.status.kind}`);
