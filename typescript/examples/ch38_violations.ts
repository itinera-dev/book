// Declaring by hand: a workflow put together wrongly is refused with every
// violation when its builder builds it, before any journey.

import { type Contributor, contributor } from "itinera/journey";
import { fromStep, type OnSuccess, StepPolicy } from "itinera/policy";
import { input, type Outcome, Step } from "itinera/step";
import { integer, listOf, objectOf } from "itinera/value";
import {
  fromWorkflow,
  key,
  type Supplied,
  Violations,
  Workflow,
} from "itinera/workflow";

// ANCHOR: shop
/** Charges an order's price, and gives back the amount it charged. */
class Charge extends Step {
  constructor(private readonly price: number) {
    super();
  }

  override run(contributor: Contributor): Outcome {
    if (this.price <= 0) return this.fail("nothing-to-charge");
    contributor.contribute("charged", this.price);
    return this.succeed();
  }
}

/** Ships an order's parcel, of this weight in kg, up to the shop's limit. */
class Ship extends Step {
  constructor(private readonly weight: number) {
    super();
  }

  override run(): Outcome {
    if (this.weight > 80) return this.fail("over-parcel-limit");
    return this.succeed();
  }
}

/** Audits every successful charge. */
class Audit extends StepPolicy {
  audited(charged: number, contributor: Contributor): OnSuccess {
    contributor.contribute("audit", `charged ${String(charged)}`);
    return this.keepDefault();
  }
}

/** Writes a receipt for every successful charge. */
class Receipts extends StepPolicy {
  written(charged: number, contributor: Contributor): OnSuccess {
    contributor.contribute("receipt", `R-${String(charged)}`);
    return this.keepDefault();
  }
}

/**
 * How the warehouse describes an order's parcel: its weight in kg, and its
 * length, width and height in cm.
 */
interface Parcel {
  weight: number;
  size: number[];
}

/** The type token of a parcel. */
const PARCEL = objectOf({ weight: integer, size: listOf(integer) });

/** The shop's workflow class, whose one method is the parcels adapter. */
class Orders extends Workflow {
  /**
   * Gives the shipping the weight of the order's parcel. Every other input is
   * read from the data bag.
   */
  parcels(key: string, parcel: Parcel): Supplied {
    if (key !== "weight") return this.fromDataBag();
    return this.supply(parcel.weight);
  }
}

/** The charge, under its name, with the requests of its parameters. */
function charge() {
  return Charge.builder("charge")
    .needs(input("price", integer))
    .run(contributor());
}

/** The shipping, under its name, with the request of its parameter. */
function ship() {
  return Ship.builder("ship").needs(input("weight", integer));
}

const audit = Audit.builder("audit").onStepSuccess(
  "audited",
  fromStep("charged", integer),
  contributor(),
);
const receipts = Receipts.builder("receipts").onStepSuccess(
  "written",
  fromStep("charged", integer),
  contributor(),
);
// ANCHOR_END: shop

// ANCHOR: wrong
const wrong = Orders.builder()
  .name("orders")
  .step(charge().policy(audit).policy(receipts))
  .step(ship())
  .step(ship().retryBudget(1))
  .inputAdapter(
    "parcels",
    { steps: ["shipping"] },
    key(),
    fromWorkflow("parcel", PARCEL),
  );
// ANCHOR_END: wrong

// ANCHOR: report
try {
  wrong.build();
} catch (error) {
  if (!(error instanceof Violations)) throw error;
  console.log("the workflow was refused:");
  console.log(error.message);
}
// ANCHOR_END: report

// ANCHOR: fixed
const fixed = Orders.builder()
  .name("orders")
  .step(charge().policy(audit))
  .step(ship().retryBudget(1))
  .inputAdapter(
    "parcels",
    { steps: ["ship"] },
    key(),
    fromWorkflow("parcel", PARCEL),
  );
console.log("the fixed workflow is put together as:");
for (const listed of fixed.listing()) {
  console.log(`${String(listed.position)}. ${listed.step}`);
}
// ANCHOR_END: fixed
