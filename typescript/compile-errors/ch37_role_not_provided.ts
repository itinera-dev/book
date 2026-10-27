// A policy that needs a role the workflow does not provide.

import { LocalExecutor } from "itinera/executor";
import {
  onWorkflowSuccess,
  role,
  WorkflowPolicy,
  workflowPolicy,
} from "itinera/policy";
import { type Outcome, Step, step } from "itinera/step";
import { string } from "itinera/value";
import { fromWorkflow, withStep, Workflow, workflow } from "itinera/workflow";

/** Charges an order. */
@step()
class Charge extends Step {
  override run(): Outcome {
    return this.succeed();
  }
}

/** The shop's way of telling a customer something. */
interface Messages {
  tell(customer: string, message: string): void;
}

/** Tells the customer once the order is complete. */
@workflowPolicy({ name: "notify" })
class Notify extends WorkflowPolicy<Messages> {
  @onWorkflowSuccess()
  completed(
    @fromWorkflow("customer", string) customer: string,
    @role<Messages>() messages: Messages,
  ): void {
    messages.tell(customer, "your order is complete");
  }
}

/** The shop's workflow, which has no way of telling a customer anything. */
@workflow({ name: "orders", policies: [Notify] })
@withStep(Charge, { name: "charge" })
class Orders extends Workflow {}

const result = await new LocalExecutor().run(
  Orders.build().instance({ customer: "Ann" }),
);
console.log(`the journey ${result.status.kind}`);
