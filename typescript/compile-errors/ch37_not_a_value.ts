// A step that contributes something that is not a value: its connection to
// the bank.

import { LocalExecutor } from "itinera/executor";
import { type Contributor, contributor } from "itinera/journey";
import { type Outcome, Step, step } from "itinera/step";
import { withStep, Workflow, workflow } from "itinera/workflow";

/** A connection to the customer's bank, faked. */
class BankConnection {
  constructor(readonly bank: string) {}

  charge(amount: number): string {
    return `${this.bank} charged ${String(amount)}`;
  }
}

/** Charges an order, and hands its bank connection on. */
@step()
class Charge extends Step {
  override run(@contributor() contributor: Contributor): Outcome {
    const connection = new BankConnection("the customer's bank");
    contributor.contribute("bank", connection);
    return this.succeed();
  }
}

/** The shop's workflow. */
@workflow({ name: "orders" })
@withStep(Charge, { name: "charge" })
class Orders extends Workflow {}

const result = await new LocalExecutor().run(Orders.build().instance());
console.log(`the journey ${result.status.kind}`);
