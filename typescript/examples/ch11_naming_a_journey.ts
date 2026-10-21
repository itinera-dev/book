// Naming a journey: each journey is named after its order, so every event says
// which order it is about.

import type { Event } from "itinera/event";
import { LocalExecutor } from "itinera/executor";
import {
  type Contributor,
  contributor,
  type DataBag,
  dataBag,
} from "itinera/journey";
import type { Reporter } from "itinera/report";
import { type Provided, provider, service, serviceType } from "itinera/service";
import {
  input,
  type Outcome,
  Step,
  step,
  stepReporter,
  type StepReporter,
} from "itinera/step";
import { integer, optional, string } from "itinera/value";
import { idGenerator, withStep, Workflow, workflow } from "itinera/workflow";

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
 * Ships an order through the carrier, with its receipt in the parcel, gives
 * back its tracking number, and says when the parcel has left.
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

  override async run(
    @service("carrier", CARRIER) carrier: Carrier,
    @contributor() contributor: Contributor,
    @stepReporter() reporter: StepReporter,
  ): Promise<Outcome> {
    const tracking = carrier.book(this.order);
    contributor.contribute("tracking", tracking);
    if (!carrier.collects(this.weight)) return this.fail("too-heavy");
    await reporter.info(`the parcel leaves with receipt ${this.receipt}`);
    return this.succeed();
  }
}

// ANCHOR: log
/**
 * The shop's log: it writes a line for every event of a journey, starting
 * with the workflow's name and the journey's ID.
 */
class Log implements Reporter {
  report(event: Event): void {
    const said = detail(event);
    const line = `log ${event.workflow} ${event.journeyId} #${String(event.sequence)} ${event.kind}`;
    console.log(said === undefined ? line : `${line} ${said}`);
  }
}
// ANCHOR_END: log

/**
 * What the log says after an event's kind, for the kinds it says more about:
 * the step the event is about, and what it names.
 */
function detail(event: Event): string | undefined {
  switch (event.kind) {
    case "journey_started":
      return event.initialKeys.join(", ");
    case "attempt_started":
    case "step_succeeded":
    case "step_given_up":
      return event.step;
    case "service_provided":
      return `${event.step} gets ${event.key} from ${event.provider}, a ${event.implementation}`;
    case "step_failed":
      return `${event.step}: ${event.reason.code}`;
    case "contribution_committed":
      return event.key;
    case "journey_failed":
      return event.step;
    case "step_info":
      return `${event.step}: ${event.message}`;
    default:
      return undefined;
  }
}

// ANCHOR: desk
/**
 * The customer service desk: of all the events of a journey, it keeps only a
 * failed journey's.
 */
class Desk implements Reporter {
  report(event: Event): void {
    if (event.kind === "journey_failed") {
      console.log(`desk: order ${event.journeyId} failed at ${event.step}`);
    }
  }
}
// ANCHOR_END: desk

// ANCHOR: id
/**
 * The shop's workflow: it charges, then ships, and every journey goes to the
 * log and the desk.
 */
@workflow({ name: "orders", reporters: [Log, Desk] })
@withStep(Charge, { name: "charge" })
@withStep(Ship, { name: "ship" })
class Orders extends Workflow {
  /** Names each journey after its order. */
  @idGenerator()
  orderNumber(@dataBag() data: DataBag): string {
    return data.read("order", string).option().expect("the order's number");
  }

  /** The bank, as the payment client of the journey. */
  @provider(PAYMENTS, { key: "payments" })
  bank(): Provided<Payments> {
    return this.provide(new Bank());
  }

  /** The main carrier, as the carrier of the journey. */
  @provider(CARRIER, { key: "carrier" })
  mainCarrier(): Provided<Carrier> {
    return this.provide(new MainCarrier(20));
  }
}
// ANCHOR_END: id

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
