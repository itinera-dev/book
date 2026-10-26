// A shipping step that counts its attempts in a static field of its class.

import { LocalExecutor } from "itinera/executor";
import { type Outcome, Step, step } from "itinera/step";
import { withStep, Workflow, workflow } from "itinera/workflow";

/** Ships an order, and fails its first attempt, counting attempts on its class. */
@step()
class Ship extends Step {
  static attempts = 0;

  override run(): Outcome {
    Ship.attempts += 1;
    return Ship.attempts === 1 ? this.fail("carrier-refused") : this.succeed();
  }
}

/** The shop's workflow. */
@workflow({ name: "orders" })
@withStep(Ship, { name: "shipping", retryBudget: 1 })
class Orders extends Workflow {}

const result = await new LocalExecutor().run(Orders.build().instance());
console.log(`the journey ${result.status.kind}`);
