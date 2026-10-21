// A first journey: the shop runs the charge for two orders, and the result
// says how each journey ended.

import { LocalExecutor } from "itinera/executor";
import { input, type Outcome, Step, step } from "itinera/step";
import { integer } from "itinera/value";
import { withStep, Workflow, workflow } from "itinera/workflow";

/** The shop's bank, faked: it prints what it charges. */
function bankCharges(amount: number): void {
  console.log(`the bank charges ${String(amount)}`);
}

/** Charges an order's price. */
@step()
class Charge extends Step {
  constructor(@input("price", integer) private readonly price: number) {
    super();
  }

  override run(): Outcome {
    if (this.price <= 0) return this.fail("nothing-to-charge");
    bankCharges(this.price);
    return this.succeed();
  }
}

// ANCHOR: workflow
/** The shop's workflow: for now, it only charges. */
@workflow({ name: "orders" })
@withStep(Charge, { name: "charge" })
class Orders extends Workflow {}
// ANCHOR_END: workflow

// ANCHOR: run
const shop = Orders.build();
// The customer, and the price of their order.
const orders = [
  { customer: "Ann", price: 40 },
  { customer: "Hal", price: 0 },
];
for (const { customer, price } of orders) {
  console.log(`${customer}'s order:`);
  const instance = shop.instance({ price });
  const result = await new LocalExecutor().run(instance);
  console.log(`the journey ${result.status.kind}`);
}
// ANCHOR_END: run
