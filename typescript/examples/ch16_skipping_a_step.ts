// Skipping a step: the shipping skips itself for a gift card, which has
// nothing to ship.

import type { Event } from "itinera/event";
import { LocalExecutor } from "itinera/executor";
import {
  type Contributor,
  contributor,
  type DataBag,
  dataBag,
} from "itinera/journey";
import {
  fromStep,
  onStepSuccess,
  type OnSuccess,
  StepPolicy,
  stepPolicy,
} from "itinera/policy";
import type { Reporter } from "itinera/report";
import { type Provided, provider, service, serviceType } from "itinera/service";
import {
  attempt,
  Attempt,
  failureReason,
  input,
  type Outcome,
  type Reason,
  RescueStep,
  rescueStep,
  type RescueOutcome,
  Step,
  step,
  stepName,
  type StepName,
  stepReporter,
  type StepReporter,
} from "itinera/step";
import { boolean, integer, optional, string } from "itinera/value";
import {
  fromWorkflow,
  idGenerator,
  withStep,
  Workflow,
  workflow,
} from "itinera/workflow";

/**
 * A payment client: it charges an order's amount to a card, and gives back a
 * receipt, or `undefined` when the card has expired. It throws `NetworkDown`
 * when its network fails. And it says which card replaced one, if any did.
 */
interface Payments {
  charge(order: string, card: string, amount: number): string | undefined;
  replacement(card: string): string | undefined;
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

const PAYMENTS = serviceType<Payments>("Payments");
const CARRIER = serviceType<Carrier>("Carrier");

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

  /**
   * A card that has expired was replaced by its holder's card that expires in
   * three years.
   */
  replacement(card: string): string | undefined {
    const holder = card.slice(0, card.lastIndexOf("-"));
    const replacement = `${holder}-${String(this.thisYear + 3)}`;
    console.log(`the bank says card ${card} was replaced by ${replacement}`);
    return replacement;
  }
}

/** The year a card expires: the part of it after its last `-`. */
function expiry(card: string): number {
  return Number(card.slice(card.lastIndexOf("-") + 1));
}

/**
 * The shop's main carrier, faked: its booking system fails on a parcel over
 * 100 kg, and it collects a parcel only up to 20 kg.
 */
class MainCarrier implements Carrier {
  book(order: string, weight: number): string {
    if (weight > 100) {
      console.log(`the main carrier's booking system fails on order ${order}`);
      throw new BookingFailed();
    }
    console.log(`the main carrier books order ${order}`);
    return `T-${order}`;
  }

  collects(weight: number): boolean {
    return weight <= 20;
  }
}

/**
 * The shop's backup carrier, faked: its booking system fails on a parcel over
 * 100 kg, and it collects a parcel only up to 80 kg.
 */
class BackupCarrier implements Carrier {
  book(order: string, weight: number): string {
    if (weight > 100) {
      console.log(
        `the backup carrier's booking system fails on order ${order}`,
      );
      throw new BookingFailed();
    }
    console.log(`the backup carrier books order ${order}`);
    return `B-${order}`;
  }

  collects(weight: number): boolean {
    return weight <= 80;
  }
}

/**
 * Charges an order's price to its card through the payment client, and gives
 * back its receipt, and the gift card's code when the order is a gift card.
 */
@step()
class Charge extends Step {
  constructor(
    @input("order", string) private readonly order: string,
    @input("price", integer) private readonly price: number,
    @input("card", string) private readonly card: string,
    @input("giftCard", boolean) private readonly giftCard: boolean,
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
    if (this.giftCard) contributor.contribute("code", `G-${this.order}`);
    return this.succeed();
  }
}

/**
 * Repairs a charge that failed because its card had expired: it asks the bank
 * for the card that replaced it, and gives it back as the order's card.
 */
@rescueStep()
class RenewCard extends RescueStep {
  constructor(
    @input("card", string) private readonly card: string,
    @failureReason() private readonly reason: Reason,
  ) {
    super();
  }

  override run(
    @service("payments", PAYMENTS) payments: Payments,
    @contributor() contributor: Contributor,
  ): RescueOutcome {
    if (this.reason.code !== "card-expired") {
      return this.fail("not-repairable");
    }
    const card = payments.replacement(this.card);
    if (card === undefined) return this.fail("not-replaced");
    contributor.contribute("card", card);
    return this.succeed();
  }
}

/**
 * Writes an audit line for every step it is attached to that succeeds: what
 * the order was charged, and the receipt the step gave back.
 */
@stepPolicy({ name: "audit" })
class Audit extends StepPolicy {
  @onStepSuccess()
  charged(
    @stepName() step: StepName,
    @fromWorkflow("price", integer) price: number,
    @fromStep("receipt", string) receipt: string,
    @contributor() contributor: Contributor,
  ): OnSuccess {
    contributor.contribute(
      "audit",
      `${step} took ${String(price)}, receipt ${receipt}`,
    );
    return this.keepDefault();
  }
}

// ANCHOR: ship
/**
 * Ships an order through the carrier, with its receipt in the parcel, gives
 * back its tracking number, and says when the parcel has left. A parcel the
 * carrier won't collect may be taken on another attempt. A gift card has
 * nothing to ship.
 */
@step()
class Ship extends Step {
  constructor(
    @input("order", string) private readonly order: string,
    @input("weight", integer) private readonly weight: number,
    @input("receipt", string) private readonly receipt: string,
    @input("giftCard", boolean) private readonly giftCard: boolean,
  ) {
    super();
  }

  override async run(
    @service("carrier", CARRIER) carrier: Carrier,
    @contributor() contributor: Contributor,
    @stepReporter() reporter: StepReporter,
  ): Promise<Outcome> {
    if (this.giftCard) return this.skipBecause("nothing-to-ship");
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
    case "step_rescuing":
      return `${event.step}: ${event.cause}, rescued by ${event.rescue}`;
    case "step_rescued":
      return `${event.step} by ${event.rescue}`;
    case "contribution_committed":
    case "data_overwritten":
      return event.key;
    case "journey_failed":
      return event.step;
    case "step_info":
      return `${event.step}: ${event.message}`;
    case "hook_called":
      if (event.source.kind !== "step hook") return undefined;
      return `${event.source.policy}: ${event.source.hook} of ${event.source.step}`;
    // ANCHOR: skipped
    case "step_skipped":
      if (event.reason === undefined) return undefined;
      return `${event.step}: ${event.reason.code}`;
    case "contributions_discarded":
      return event.step;
    // ANCHOR_END: skipped
    default:
      return undefined;
  }
}

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

/**
 * The shop's workflow: it charges, repairing a charge that fails and auditing
 * one that succeeds, then ships, and shipping may be tried once more.
 */
@workflow({ name: "orders", reporters: [Log, Desk] })
@withStep(Charge, { name: "charge", rescue: RenewCard, policies: [Audit] })
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

  /**
   * A carrier for every attempt of a step: the main carrier first, the backup
   * carrier after.
   */
  @provider(CARRIER, { key: "carrier", perAttempt: true })
  carriers(@attempt() attempt: Attempt): Provided<Carrier> {
    if (attempt.isFirst()) return this.provide(new MainCarrier());
    return this.provide(new BackupCarrier());
  }
}

/**
 * What the shop reads back once the journey has ended: the type of the value
 * under each name.
 */
const orderOutput = {
  receipt: optional(string),
  tracking: optional(string),
  audit: optional(string),
};

const shop = Orders.build();
// The order, the customer, the price, the parcel's weight in kg, the card, and
// whether the order is a gift card.
const orders = [
  {
    order: "A-1",
    customer: "Ann",
    price: 40,
    weight: 2,
    card: "ann-2028",
    giftCard: false,
  },
  {
    order: "A-7",
    customer: "Gil",
    price: 45,
    weight: 60,
    card: "gil-2027",
    giftCard: false,
  },
  {
    order: "A-8",
    customer: "Jo",
    price: 30,
    weight: 1,
    card: "jo-2024",
    giftCard: false,
  },
  {
    order: "A-9",
    customer: "Kim",
    price: 25,
    weight: 0,
    card: "kim-2027",
    giftCard: true,
  },
  {
    order: "A-12",
    customer: "Ned",
    price: 50,
    weight: 90,
    card: "ned-2028",
    giftCard: false,
  },
];
for (const { order, customer, price, weight, card, giftCard } of orders) {
  console.log(`${customer}'s order:`);
  const instance = shop.instance({ order, price, weight, card, giftCard });
  const result = await new LocalExecutor().run(instance);
  const data = result.output(orderOutput);
  const receipt = data.receipt.unwrapOr("none");
  const tracking = data.tracking.unwrapOr("none");
  console.log(
    `the journey ${result.status.kind}, with receipt ${receipt} and tracking ${tracking}`,
  );
  console.log(`audit: ${data.audit.unwrapOr("none")}`);
}
