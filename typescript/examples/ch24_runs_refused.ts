// Runs an executor refuses: an instance handed to it a second time, and an
// instance handed to it while it runs another journey.

import { LocalExecutor, Refusal } from "itinera/executor";
import type { JourneyResult } from "itinera/journey";
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

/** The shop's workflow: for now, it only charges. */
@workflow({ name: "orders" })
@withStep(Charge, { name: "charge" })
class Orders extends Workflow {}

// ANCHOR: run
/** Says how a run went: how its journey ended, or why it was refused. */
async function said(run: Promise<JourneyResult>): Promise<string> {
  try {
    const result = await run;
    return `the journey ${result.status.kind}`;
  } catch (error) {
    if (!(error instanceof Refusal)) throw error;
    return `refused (${error.refused.kind}): ${error.message}`;
  }
}

const shop = Orders.build();
const executor = new LocalExecutor();

const ann = shop.instance({ price: 40 });
console.log(`Ann's order: ${await said(executor.run(ann))}`);
console.log(`Ann's order again: ${await said(executor.run(ann))}`);

const ben = executor.run(shop.instance({ price: 25 }));
const cat = executor.run(shop.instance({ price: 30 }));
console.log(`Cat's order, while Ben's runs: ${await said(cat)}`);
console.log(`Ben's order: ${await said(ben)}`);
// ANCHOR_END: run
