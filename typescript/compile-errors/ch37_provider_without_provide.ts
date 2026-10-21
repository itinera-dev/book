// A provider that returns its service itself, rather than through
// `this.provide(...)`.

import { LocalExecutor } from "itinera/executor";
import { provider, service, serviceType } from "itinera/service";
import { type Outcome, Step, step } from "itinera/step";
import { withStep, Workflow, workflow } from "itinera/workflow";

/** A payment client: it charges an amount, and gives back a receipt. */
interface Payments {
  charge(amount: number): string;
}

const PAYMENTS = serviceType<Payments>("Payments");

/** The shop's bank, faked. */
class Bank implements Payments {
  charge(amount: number): string {
    return `R-${String(amount)}`;
  }
}

/** Charges an order through the payment client. */
@step()
class Charge extends Step {
  override run(@service("payments", PAYMENTS) payments: Payments): Outcome {
    payments.charge(30);
    return this.succeed();
  }
}

/** The shop's workflow, whose provider hands the bank over as it is. */
@workflow({ name: "orders" })
@withStep(Charge, { name: "charge" })
class Orders extends Workflow {
  @provider(PAYMENTS, { key: "payments" })
  bank(): Payments {
    return new Bank();
  }
}

const result = await new LocalExecutor().run(Orders.build().instance());
console.log(`the journey ${result.status.kind}`);
