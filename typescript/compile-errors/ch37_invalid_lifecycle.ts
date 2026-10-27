// A hook that returns a lifecycle it may not return: finishing the journey
// after a failure.

import { LocalExecutor } from "itinera/executor";
import {
  onStepFailure,
  type OnFailure,
  StepPolicy,
  stepPolicy,
} from "itinera/policy";
import { type Outcome, Step, step } from "itinera/step";
import { withStep, Workflow, workflow } from "itinera/workflow";

/** Charges an order, and fails when there is nothing to charge. */
@step()
class Charge extends Step {
  override run(): Outcome {
    return this.fail("nothing-to-charge");
  }
}

/** Tries to end the journey as succeeded when the charge fails. */
@stepPolicy({ name: "forgive" })
class Forgive extends StepPolicy {
  @onStepFailure()
  forgiven(): OnFailure {
    return this.finishWorkflow();
  }
}

/** The shop's workflow. */
@workflow({ name: "orders" })
@withStep(Charge, { name: "charge", policies: [Forgive] })
class Orders extends Workflow {}

const result = await new LocalExecutor().run(Orders.build().instance());
console.log(`the journey ${result.status.kind}`);
