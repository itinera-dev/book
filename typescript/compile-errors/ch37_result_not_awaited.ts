// A journey run without awaiting its result.

import { LocalExecutor } from "itinera/executor";
import { type Outcome, Step, step } from "itinera/step";
import { withStep, Workflow, workflow } from "itinera/workflow";

/** Charges an order. */
@step()
class Charge extends Step {
  override run(): Outcome {
    return this.succeed();
  }
}

/** The shop's workflow. */
@workflow({ name: "orders" })
@withStep(Charge, { name: "charge" })
class Orders extends Workflow {}

const result = new LocalExecutor().run(Orders.build().instance());
if (result.isSucceeded()) console.log("the journey succeeded");
