// Passing data on: the charge gives back a receipt that the shipping puts in
// the parcel, and a failed attempt's contributions never reach the data bag.

import { LocalExecutor } from "itinera/executor";
import { type Contributor, contributor } from "itinera/journey";
import { input, type Outcome, Step, step } from "itinera/step";
import { integer, optional, string } from "itinera/value";
import { withStep, Workflow, workflow } from "itinera/workflow";

/** The shop's bank, faked: it prints what it charges, and gives a receipt for it. */
function bankCharges(order: string, amount: number): string {
  console.log(`the bank charges ${String(amount)} for order ${order}`);
  return `R-${order}`;
}

/**
 * The shop's carrier, faked: it books any parcel, and gives its tracking
 * number at once.
 */
function carrierBooks(order: string): string {
  console.log(`the carrier books order ${order}`);
  return `T-${order}`;
}

/**
 * The carrier, faked: it weighs a parcel when it comes to collect it, and
 * takes it only up to 20 kg.
 */
function carrierCollects(weight: number): boolean {
  return weight <= 20;
}

// ANCHOR: charge
/** Charges an order's price, and gives back the bank's receipt. */
@step()
class Charge extends Step {
  constructor(
    @input("order", string) private readonly order: string,
    @input("price", integer) private readonly price: number,
  ) {
    super();
  }

  override run(@contributor() contributor: Contributor): Outcome {
    if (this.price <= 0) return this.fail("nothing-to-charge");
    const receipt = bankCharges(this.order, this.price);
    contributor.contribute("receipt", receipt);
    return this.succeed();
  }
}
// ANCHOR_END: charge

// ANCHOR: ship
/**
 * Ships an order, with its receipt in the parcel, and gives back its tracking
 * number.
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

  override run(@contributor() contributor: Contributor): Outcome {
    const tracking = carrierBooks(this.order);
    contributor.contribute("tracking", tracking);
    if (!carrierCollects(this.weight)) return this.fail("too-heavy");
    console.log(`the parcel leaves with receipt ${this.receipt}`);
    return this.succeed();
  }
}
// ANCHOR_END: ship

// ANCHOR: workflow
/** The shop's workflow: it charges, then ships. */
@workflow({ name: "orders" })
@withStep(Charge, { name: "charge" })
@withStep(Ship, { name: "ship" })
class Orders extends Workflow {}
// ANCHOR_END: workflow

// ANCHOR: output
/**
 * What the shop reads back once the journey has ended: the type of the value
 * under each name.
 */
const orderOutput = { receipt: string, tracking: optional(string) };
// ANCHOR_END: output

// ANCHOR: run
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
// ANCHOR_END: run
