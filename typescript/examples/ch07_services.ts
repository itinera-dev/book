// Giving a step the tools it needs: the workflow provides the bank and the
// carrier, and the steps use them without knowing which ones they got.

import { LocalExecutor } from "itinera/executor";
import { type Contributor, contributor } from "itinera/journey";
import { type Provided, provider, service, serviceType } from "itinera/service";
import { input, type Outcome, Step, step } from "itinera/step";
import { integer, optional, string } from "itinera/value";
import { withStep, Workflow, workflow } from "itinera/workflow";

// ANCHOR: services
/** A payment client: it charges an order's amount, and gives back a receipt. */
interface Payments {
  charge(order: string, amount: number): string;
}

/**
 * A carrier: it books a parcel and gives back its tracking number, then says
 * whether it collects a parcel of this weight in kg.
 */
interface Carrier {
  book(order: string): string;
  collects(weight: number): boolean;
}

const PAYMENTS = serviceType<Payments>("Payments");
const CARRIER = serviceType<Carrier>("Carrier");
// ANCHOR_END: services

/** The shop's bank, faked: it prints what it charges, and gives a receipt for it. */
class Bank implements Payments {
  charge(order: string, amount: number): string {
    console.log(`the bank charges ${String(amount)} for order ${order}`);
    return `R-${order}`;
  }
}

/**
 * The shop's carrier, faked: it books any parcel, and collects one only up to
 * its weight limit.
 */
class MainCarrier implements Carrier {
  constructor(private readonly maxWeight: number) {}

  book(order: string): string {
    console.log(`the carrier books order ${order}`);
    return `T-${order}`;
  }

  collects(weight: number): boolean {
    return weight <= this.maxWeight;
  }
}

// ANCHOR: steps
/**
 * Charges an order's price through the payment client, and gives back its
 * receipt.
 */
@step()
class Charge extends Step {
  constructor(
    @input("order", string) private readonly order: string,
    @input("price", integer) private readonly price: number,
  ) {
    super();
  }

  override run(
    @service("payments", PAYMENTS) payments: Payments,
    @contributor() contributor: Contributor,
  ): Outcome {
    if (this.price <= 0) return this.fail("nothing-to-charge");
    const receipt = payments.charge(this.order, this.price);
    contributor.contribute("receipt", receipt);
    return this.succeed();
  }
}

/**
 * Ships an order through the carrier, with its receipt in the parcel, and
 * gives back its tracking number.
 */
@step()
class Ship extends Step {
  constructor(
    @input("order", string) private readonly order: string,
    @input("weight", integer) private readonly weight: number,
    @input("receipt", string) private readonly receipt: string,
  ) {
    super();
  }

  override run(
    @service("carrier", CARRIER) carrier: Carrier,
    @contributor() contributor: Contributor,
  ): Outcome {
    const tracking = carrier.book(this.order);
    contributor.contribute("tracking", tracking);
    if (!carrier.collects(this.weight)) return this.fail("too-heavy");
    console.log(`the parcel leaves with receipt ${this.receipt}`);
    return this.succeed();
  }
}
// ANCHOR_END: steps

// ANCHOR: workflow
/**
 * The shop's workflow: it charges, then ships, with the services each step
 * needs.
 */
@workflow({ name: "orders" })
@withStep(Charge, { name: "charge" })
@withStep(Ship, { name: "ship" })
class Orders extends Workflow {
  /** The bank, as the payment client of the journey. */
  @provider(PAYMENTS, { key: "payments" })
  bank(): Provided<Payments> {
    console.log("the shop connects to the bank");
    return this.provide(new Bank());
  }

  /** The main carrier, for the journey. */
  @provider(CARRIER)
  carrier(): Provided<Carrier> {
    console.log("the shop calls the carrier");
    return this.provide(new MainCarrier(20));
  }
}
// ANCHOR_END: workflow

/**
 * What the shop reads back once the journey has ended: the type of the value
 * under each name.
 */
const orderOutput = { receipt: string, tracking: optional(string) };

const shop = Orders.build();
// The order, the customer, the price and the parcel's weight in kg.
const orders = [
  { order: "A-1", customer: "Ann", price: 40, weight: 2 },
  { order: "A-7", customer: "Gil", price: 45, weight: 60 },
];
for (const { order, customer, price, weight } of orders) {
  console.log(`${customer}'s order:`);
  const instance = shop.instance({ order, price, weight });
  const result = await new LocalExecutor().run(instance);
  const data = result.output(orderOutput);
  const tracking = data.tracking.unwrapOr("none");
  console.log(
    `the journey ${result.status.kind}, with receipt ${data.receipt} and tracking ${tracking}`,
  );
}
