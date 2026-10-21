// Trying again: the workflow gives the charge and the shipping a retry budget,
// so a failure or an error is tried once more before the order is given up,
// except an error from the bank.

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

// ANCHOR: services
/**
 * A payment client: it charges an order's amount to a card, and gives back a
 * receipt, or `undefined` when the card has expired. It throws `NetworkDown`
 * when its network fails.
 */
interface Payments {
  charge(order: string, card: string, amount: number): string | undefined;
}

/**
 * A carrier: it books a parcel of this weight in kg and gives back its
 * tracking number, or throws `BookingFailed` when its booking system fails,
 * then says whether it collects the parcel.
 */
interface Carrier {
  book(order: string, weight: number): string;
  collects(weight: number): boolean;
}
// ANCHOR_END: services

const PAYMENTS = serviceType<Payments>("Payments");
const CARRIER = serviceType<Carrier>("Carrier");

// ANCHOR: errors
/** The error of the bank's network when it fails. */
class NetworkDown extends Error {
  constructor() {
    super("the payment network is down");
  }
}

/** The error of the carrier's booking system when it fails. */
class BookingFailed extends Error {
  constructor() {
    super("the booking system failed");
  }
}
// ANCHOR_END: errors

/**
 * The shop's bank, faked: a card is its holder's name and the year it
 * expires, and the bank declines a card that expired before this year. Its
 * network fails on any amount over 100. It prints what it does.
 */
class Bank implements Payments {
  constructor(private readonly thisYear: number) {}

  charge(order: string, card: string, amount: number): string | undefined {
    if (amount > 100) {
      console.log(`the bank's network fails while charging order ${order}`);
      throw new NetworkDown();
    }
    if (expiry(card) < this.thisYear) {
      console.log(`the bank declines card ${card}: it has expired`);
      return undefined;
    }
    console.log(`the bank charges ${String(amount)} for order ${order}`);
    return `R-${order}`;
  }
}

/** The year a card expires: the part of it after its last `-`. */
function expiry(card: string): number {
  return Number(card.slice(card.lastIndexOf("-") + 1));
}

/**
 * The shop's carrier, faked: its booking system fails on a parcel over
 * 100 kg, and it collects a parcel only up to its weight limit.
 */
class MainCarrier implements Carrier {
  constructor(private readonly maxWeight: number) {}

  book(order: string, weight: number): string {
    if (weight > 100) {
      console.log(`the carrier's booking system fails on order ${order}`);
      throw new BookingFailed();
    }
    console.log(`the carrier books order ${order}`);
    return `T-${order}`;
  }

  collects(weight: number): boolean {
    return weight <= this.maxWeight;
  }
}

// ANCHOR: charge
/**
 * Charges an order's price to its card through the payment client, and gives
 * back its receipt.
 */
@step()
class Charge extends Step {
  constructor(
    @input("order", string) private readonly order: string,
    @input("price", integer) private readonly price: number,
    @input("card", string) private readonly card: string,
  ) {
    super();
  }

  override run(
    @service("payments", PAYMENTS) payments: Payments,
    @contributor() contributor: Contributor,
  ): Outcome {
    if (this.price <= 0) return this.fail("nothing-to-charge");
    const receipt = payments.charge(this.order, this.card, this.price);
    if (receipt === undefined) return this.fail("card-expired");
    contributor.contribute("receipt", receipt);
    return this.succeed();
  }
}
// ANCHOR_END: charge

// ANCHOR: ship
/**
 * Ships an order through the carrier, with its receipt in the parcel, gives
 * back its tracking number, and says when the parcel has left. A parcel the
 * carrier won't collect may be taken on another attempt.
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
    const tracking = carrier.book(this.order, this.weight);
    contributor.contribute("tracking", tracking);
    if (!carrier.collects(this.weight)) return this.fail("too-heavy");
    await reporter.info(`the parcel leaves with receipt ${this.receipt}`);
    return this.succeed();
  }
}
// ANCHOR_END: ship

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

// ANCHOR: detail
/**
 * What the log says after an event's kind, for the kinds it says more about:
 * the step the event is about, and what it names.
 */
function detail(event: Event): string | undefined {
  switch (event.kind) {
    case "journey_started":
      return event.initialKeys.join(", ");
    case "attempt_started":
      return `${event.step}, attempt ${String(event.attempt)}`;
    case "step_succeeded":
    case "step_retrying":
      return event.step;
    case "service_provided":
      return `${event.step} gets ${event.key} from ${event.provider}, a ${event.implementation}`;
    case "step_failed":
      return `${event.step}: ${event.reason.code}`;
    case "step_abnormal_termination":
      return `${event.step}: ${event.errorMessage}`;
    case "step_given_up":
      return `${event.step}: ${event.cause}`;
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
// ANCHOR_END: detail

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

// ANCHOR: workflow
/**
 * The shop's workflow: it charges, then ships, and each may be tried once
 * more, except a charge that ended in an error.
 */
@workflow({ name: "orders", reporters: [Log, Desk] })
@withStep(Charge, {
  name: "charge",
  retryBudget: 1,
  noRetryOnAbnormalTermination: true,
})
@withStep(Ship, { name: "ship", retryBudget: 1 })
class Orders extends Workflow {
  /** Names each journey after its order. */
  @idGenerator()
  orderNumber(@dataBag() data: DataBag): string {
    return data.read("order", string).option().expect("the order's number");
  }

  /** The bank, as the payment client of the journey. */
  @provider(PAYMENTS, { key: "payments" })
  bank(): Provided<Payments> {
    return this.provide(new Bank(2026));
  }

  /** The main carrier, as the carrier of the journey. */
  @provider(CARRIER, { key: "carrier" })
  mainCarrier(): Provided<Carrier> {
    return this.provide(new MainCarrier(20));
  }
}
// ANCHOR_END: workflow

// ANCHOR: run
/**
 * What the shop reads back once the journey has ended: the type of the value
 * under each name.
 */
const orderOutput = { receipt: optional(string), tracking: optional(string) };

const shop = Orders.build();
// The order, the customer, the price, the parcel's weight in kg and the card.
const orders = [
  { order: "A-1", customer: "Ann", price: 40, weight: 2, card: "ann-2028" },
  { order: "A-7", customer: "Gil", price: 45, weight: 60, card: "gil-2027" },
  { order: "A-8", customer: "Jo", price: 30, weight: 1, card: "jo-2024" },
  { order: "A-3", customer: "Cat", price: 120, weight: 3, card: "cat-2027" },
  { order: "A-4", customer: "Dan", price: 50, weight: 150, card: "dan-2028" },
];
for (const { order, customer, price, weight, card } of orders) {
  console.log(`${customer}'s order:`);
  const instance = shop.instance({ order, price, weight, card });
  const result = await new LocalExecutor().run(instance);
  const data = result.output(orderOutput);
  const receipt = data.receipt.unwrapOr("none");
  const tracking = data.tracking.unwrapOr("none");
  console.log(
    `the journey ${result.status.kind}, with receipt ${receipt} and tracking ${tracking}`,
  );
}
// ANCHOR_END: run
