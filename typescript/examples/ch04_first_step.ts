// A first step: the shop's charge, declared as a step. Nothing builds or runs
// it yet, so the program prints nothing.

import { input, type Outcome, Step, step } from "itinera/step";
import { integer } from "itinera/value";

/** The shop's bank, faked: it prints what it charges. */
function bankCharges(amount: number): void {
  console.log(`the bank charges ${String(amount)}`);
}

// ANCHOR: step
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
// ANCHOR_END: step
