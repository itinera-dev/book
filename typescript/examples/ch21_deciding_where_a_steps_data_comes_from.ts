// Deciding where a step's data comes from: the warehouse describes each parcel
// as one value, and the workflow's input adapter gives the shipping only the
// weight it asks for.

import type { Event } from "itinera/event";
import { LocalExecutor } from "itinera/executor";
import {
  type Contributor,
  contributor,
  type DataBag,
  dataBag,
} from "itinera/journey";
import type { Option } from "itinera/option";
import {
  fromStep,
  onStepAbnormalTermination,
  type OnFailure,
  onStepRetry,
  onStepSuccess,
  type OnSuccess,
  onWorkflowFailure,
  onWorkflowSuccess,
  role,
  StepPolicy,
  stepPolicy,
  WorkflowPolicy,
  workflowPolicy,
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
import {
  boolean,
  integer,
  listOf,
  objectOf,
  optional,
  string,
} from "itinera/value";
import {
  fromWorkflow,
  idGenerator,
  inputAdapter,
  key,
  type Supplied,
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
   * A card that expired in the last three years was replaced by its holder's
   * card that expires in three years. An older one was not.
   */
  replacement(card: string): string | undefined {
    if (expiry(card) + 3 < this.thisYear) {
      console.log(`the bank says nothing replaced card ${card}`);
      return undefined;
    }
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
 * Works out what an order costs: its price, less its discount if it has one,
 * given back as the amount to charge.
 */
@step()
class Price extends Step {
  constructor(
    @input("price", integer) private readonly price: number,
    @input("discount", optional(integer))
    private readonly discount: Option<number>,
  ) {
    super();
  }

  override run(@contributor() contributor: Contributor): Outcome {
    contributor.contribute("amount", this.price - this.discount.unwrapOr(0));
    return this.succeed();
  }
}

/**
 * Charges an order's amount to its card through the payment client, and gives
 * back its receipt, and the gift card's code when the order is a gift card.
 */
@step()
class Charge extends Step {
  constructor(
    @input("order", string) private readonly order: string,
    @input("amount", integer) private readonly amount: number,
    @input("card", string) private readonly card: string,
    @input("giftCard", boolean) private readonly giftCard: boolean,
  ) {
    super();
  }

  override run(
    @service("payments", PAYMENTS) payments: Payments,
    @contributor() contributor: Contributor,
  ): Outcome {
    if (this.amount <= 0) return this.fail("nothing-to-charge");
    const receipt = payments.charge(this.order, this.card, this.amount);
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
 * Writes an audit line for every step it is attached to that succeeds: the
 * amount the order was charged, and the receipt the step gave back.
 */
@stepPolicy({ name: "audit" })
class Audit extends StepPolicy {
  @onStepSuccess()
  charged(
    @stepName() step: StepName,
    @fromWorkflow("amount", integer) amount: number,
    @fromStep("receipt", string) receipt: string,
    @contributor() contributor: Contributor,
  ): OnSuccess {
    contributor.contribute(
      "audit",
      `${step} took ${String(amount)}, receipt ${receipt}`,
    );
    return this.keepDefault();
  }
}

/** What the shop does for its policies: tell a customer something. */
interface Messages {
  tell(customer: string, message: string): void;
}

/** Tells the customer how their order went, once its journey has ended. */
@workflowPolicy({ name: "notify" })
class Notify extends WorkflowPolicy<Messages> {
  @onWorkflowSuccess()
  succeeded(
    @fromWorkflow("customer", string) customer: string,
    @fromWorkflow("order", string) order: string,
    @fromWorkflow("tracking", optional(string)) tracking: Option<string>,
    @fromWorkflow("code", optional(string)) code: Option<string>,
    @role<Messages>() messages: Messages,
  ): void {
    let message = `Order ${order} is complete.`;
    if (tracking.isSome()) {
      message = `Order ${order} is on its way, tracking ${tracking.value}.`;
    } else if (code.isSome()) {
      message = `Order ${order} is paid: your gift card's code is ${code.value}.`;
    }
    messages.tell(customer, message);
  }

  @onWorkflowFailure()
  failed(
    @fromWorkflow("customer", string) customer: string,
    @fromWorkflow("order", string) order: string,
    @role<Messages>() messages: Messages,
  ): void {
    messages.tell(customer, `Sorry, order ${order} could not be completed.`);
  }
}

/**
 * Gives up a step that ends in an error at once: the bank's network is down,
 * and neither a retry nor a rescue can mend that.
 */
@stepPolicy({ name: "outage" })
class Outage extends StepPolicy {
  @onStepAbnormalTermination()
  endedInError(): OnFailure {
    return this.failWorkflow("payment-network-down");
  }
}

/**
 * Gives up the shipping of a parcel over the shop's limit, rather than try it
 * again: the limit is the same on every attempt, whichever carrier comes.
 */
@stepPolicy({ name: "heavyParcels" })
class HeavyParcels extends StepPolicy {
  @onStepRetry()
  beforeRetrying(
    @failureReason({ optional: true }) reason: Option<Reason>,
  ): OnFailure {
    if (reason.isSome() && reason.value.code === "over-parcel-limit") {
      return this.failWorkflow("no-carrier-can-take-it");
    }
    return this.keepDefault();
  }
}

/**
 * Ships an order through the carrier, with its receipt in the parcel, gives
 * back its tracking number, and says when the parcel has left. A parcel the
 * carrier won't collect may be taken on another attempt. A gift card has
 * nothing to ship, and the shop ships no parcel over 80 kg.
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
    if (this.weight > 80) return this.fail("over-parcel-limit");
    const tracking = carrier.book(this.order, this.weight);
    contributor.contribute("tracking", tracking);
    if (!carrier.collects(this.weight)) return this.fail("too-heavy");
    await reporter.info(`the parcel leaves with receipt ${this.receipt}`);
    return this.succeed();
  }
}

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
      if (event.cause === "FailWorkflow") {
        return `${event.step}: FailWorkflow from ${event.decidedBy.policy}, ${event.reason.code}`;
      }
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
      if (event.source.kind === "workflow hook") {
        return `${event.source.policy}: ${event.source.hook}`;
      }
      return `${event.source.policy}: ${event.source.hook} of ${event.source.step}`;
    case "optional_input_absent":
      return event.key;
    case "step_skipped":
      if (event.reason === undefined) return undefined;
      return `${event.step}: ${event.reason.code}`;
    case "contributions_discarded":
      return event.step;
    // ANCHOR: supplied
    case "input_adapter_supplied":
      return `${event.step} gets ${event.key} from ${event.adapter}`;
    // ANCHOR_END: supplied
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

// ANCHOR: parcel
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
// ANCHOR_END: parcel

/**
 * The shop's workflow: it works out what the order costs, then charges it,
 * repairing a charge that fails and auditing one that succeeds, then ships,
 * and shipping may be tried once more. Once a journey has ended, the customer
 * is told, through the channel the shop configures.
 */
@workflow({ name: "orders", reporters: [Log, Desk], policies: [Notify] })
@withStep(Price, { name: "price" })
@withStep(Charge, {
  name: "charge",
  rescue: RenewCard,
  policies: [Audit, Outage],
})
@withStep(Ship, { name: "ship", retryBudget: 1, policies: [HeavyParcels] })
class Orders extends Workflow implements Messages {
  constructor(private readonly channel: string) {
    super();
  }

  tell(customer: string, message: string): void {
    console.log(`${this.channel} to ${customer}: ${message}`);
  }

  /** Names each journey after its order. */
  @idGenerator()
  orderNumber(@dataBag() data: DataBag): string {
    return data.read("order", string).option().expect("the order's number");
  }

  // ANCHOR: adapter
  /**
   * Gives the shipping the weight of the order's parcel, as the warehouse
   * describes it. Every other input of the shipping is read from the data bag.
   */
  @inputAdapter({ steps: ["ship"] })
  parcels(
    @key() key: string,
    @fromWorkflow("parcel", PARCEL) parcel: Parcel,
  ): Supplied {
    if (key !== "weight") return this.fromDataBag();
    return this.supply(parcel.weight);
  }
  // ANCHOR_END: adapter

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

const shop = Orders.build("email");
// ANCHOR: data
// The order, the customer, the price, the discount when there is one, the
// parcel, the card, and whether the order is a gift card.
const orders = [
  {
    order: "A-1",
    customer: "Ann",
    price: 40,
    discount: 10,
    parcel: { weight: 2, size: [30, 20, 10] },
    card: "ann-2028",
    giftCard: false,
  },
  {
    order: "A-7",
    customer: "Gil",
    price: 45,
    parcel: { weight: 60, size: [120, 80, 60] },
    card: "gil-2027",
    giftCard: false,
  },
  {
    order: "A-8",
    customer: "Jo",
    price: 30,
    parcel: { weight: 1, size: [20, 15, 5] },
    card: "jo-2024",
    giftCard: false,
  },
  {
    order: "A-9",
    customer: "Kim",
    price: 25,
    parcel: { weight: 0, size: [0, 0, 0] },
    card: "kim-2027",
    giftCard: true,
  },
  {
    order: "A-10",
    customer: "Lee",
    price: 35,
    parcel: { weight: 3, size: [40, 30, 20] },
    card: "lee-2019",
    giftCard: false,
  },
];
// ANCHOR_END: data
for (const order of orders) {
  console.log(`${order.customer}'s order:`);
  const result = await new LocalExecutor().run(shop.instance(order));
  const data = result.output(orderOutput);
  const receipt = data.receipt.unwrapOr("none");
  const tracking = data.tracking.unwrapOr("none");
  console.log(
    `the journey ${result.status.kind}, with receipt ${receipt} and tracking ${tracking}`,
  );
  console.log(`audit: ${data.audit.unwrapOr("none")}`);
}
