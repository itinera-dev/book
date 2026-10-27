// Declaring by hand: the shop's pricing, its audit and its workflow put
// together with the builder, the rest still declared with the decorators.

import { setImmediate } from "node:timers/promises";
import type { Event } from "itinera/event";
import { LocalExecutor } from "itinera/executor";
import {
  type Abort,
  type Contributor,
  contributor,
  type DataBag,
  dataBag,
  type Failure,
  type JourneyId,
  journeyId,
  type JourneyResult,
} from "itinera/journey";
import type { Option } from "itinera/option";
import {
  failureCause,
  fromStep,
  onStepAbnormalTermination,
  type OnFailure,
  onStepFailure,
  onStepRetry,
  onStepSuccess,
  type OnSuccess,
  onWorkflowFailure,
  onWorkflowSuccess,
  role,
  type StepFailureCause,
  StepPolicy,
  stepPolicy,
  WorkflowPolicy,
  workflowPolicy,
} from "itinera/policy";
import type { Reporter } from "itinera/report";
import { type Provided, service, serviceType } from "itinera/service";
import {
  attempt,
  Attempt,
  error,
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
  type PlainValue,
  string,
} from "itinera/value";
import { fromWorkflow, key, type Supplied, Workflow } from "itinera/workflow";

/**
 * A payment client: it charges an order's amount to a card, and gives back a
 * receipt, or `undefined` when the card has expired. It rejects with
 * `NetworkDown` when its network fails. Each answer comes over the network.
 */
interface Payments {
  charge(
    order: string,
    card: string,
    amount: number,
  ): Promise<string | undefined>;
}

/**
 * A card registry: it says which card replaced one, if any did. Each answer
 * comes over the network.
 */
interface Cards {
  replacement(card: string): Promise<string | undefined>;
}

const CARDS = serviceType<Cards>("Cards");

/**
 * The bank's own library: it connects to the bank's card registry for a
 * region, and hands back its client, an object with no class of its own.
 */
function cardRegistry(region: string): Cards {
  return { replacement: (card) => replacementIn(region, card) };
}

/**
 * A carrier: it books a parcel of this weight in kg and gives back its
 * tracking number, or rejects with `BookingFailed` when its booking system
 * fails, then says whether it collects the parcel. Each answer comes over the
 * network.
 */
interface Carrier {
  book(order: string, weight: number): Promise<string>;
  collects(weight: number): Promise<boolean>;
}

/** The shop's order book: it says whether an order was already completed. */
interface OrderBook {
  completed(order: string): boolean;
}

const ORDER_BOOK = serviceType<OrderBook>("OrderBook");

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
 * The network the fakes answer over: it waits for one turn of Node's event
 * loop before the answer comes, as a real call waits for the other side.
 */
function network(): Promise<void> {
  return setImmediate();
}

/**
 * The shop's bank, faked: a card is its holder's name and the year it
 * expires, and the bank declines a card that expired before this year. Its
 * network fails on any amount over 100. It prints what it does once it has
 * answered.
 */
class Bank implements Payments {
  constructor(private readonly thisYear: number) {}

  async charge(
    order: string,
    card: string,
    amount: number,
  ): Promise<string | undefined> {
    await network();
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

/**
 * The bank's card registry in a region, faked: a card that expired in the
 * last three years was replaced by its holder's card that expires in three
 * years, counted from 2026. An older one was not. It prints what it does once
 * it has answered.
 */
async function replacementIn(
  region: string,
  card: string,
): Promise<string | undefined> {
  await network();
  const thisYear = 2026;
  if (expiry(card) + 3 < thisYear) {
    console.log(
      `the card registry in ${region} says nothing replaced card ${card}`,
    );
    return undefined;
  }
  const holder = card.slice(0, card.lastIndexOf("-"));
  const replacement = `${holder}-${String(thisYear + 3)}`;
  console.log(
    `the card registry in ${region} says card ${card} was replaced by ${replacement}`,
  );
  return replacement;
}

/** The year a card expires: the part of it after its last `-`. */
function expiry(card: string): number {
  return Number(card.slice(card.lastIndexOf("-") + 1));
}

/**
 * The shop's main carrier, faked: its booking system fails on a parcel that
 * weighs nothing, and it collects a parcel only up to 20 kg. It prints what
 * it does once it has answered.
 */
class MainCarrier implements Carrier {
  async book(order: string, weight: number): Promise<string> {
    await network();
    if (weight === 0) {
      console.log(`the main carrier's booking system fails on order ${order}`);
      throw new BookingFailed();
    }
    console.log(`the main carrier books order ${order}`);
    return `T-${order}`;
  }

  async collects(weight: number): Promise<boolean> {
    await network();
    return weight <= 20;
  }
}

/**
 * The shop's backup carrier, faked: its booking system fails on a parcel that
 * weighs nothing, and it collects a parcel only up to 80 kg. It prints what it
 * does once it has answered.
 */
class BackupCarrier implements Carrier {
  async book(order: string, weight: number): Promise<string> {
    await network();
    if (weight === 0) {
      console.log(
        `the backup carrier's booking system fails on order ${order}`,
      );
      throw new BookingFailed();
    }
    console.log(`the backup carrier books order ${order}`);
    return `B-${order}`;
  }

  async collects(weight: number): Promise<boolean> {
    await network();
    return weight <= 80;
  }
}

/**
 * The shop's order book, faked: it holds the orders completed before today,
 * and prints what it finds.
 */
class Ledger implements OrderBook {
  constructor(private readonly done: readonly string[]) {}

  completed(order: string): boolean {
    if (this.done.includes(order)) {
      console.log(`the order book has order ${order} completed already`);
      return true;
    }
    console.log(`the order book has no order ${order} yet`);
    return false;
  }
}

/**
 * Looks the order up in the order book, and gives back whether it was already
 * completed.
 */
@step()
class Lookup extends Step {
  constructor(@input("order", string) private readonly order: string) {
    super();
  }

  override run(
    @service("orderBook", ORDER_BOOK) orderBook: OrderBook,
    @contributor() contributor: Contributor,
  ): Outcome {
    contributor.contribute("completed", orderBook.completed(this.order));
    return this.succeed();
  }
}

/**
 * Finishes the journey of an order that was already completed: there is
 * nothing left to do for it.
 */
@stepPolicy({ name: "alreadyCompleted" })
class AlreadyCompleted extends StepPolicy {
  @onStepSuccess()
  lookedUp(@fromStep("completed", boolean) completed: boolean): OnSuccess {
    if (completed) return this.finishWorkflow();
    return this.keepDefault();
  }
}

// ANCHOR: price
/**
 * Works out what an order costs: its price, less its discount if it has one,
 * given back as the amount to charge.
 */
class Price extends Step {
  constructor(
    private readonly price: number,
    private readonly discount: Option<number>,
  ) {
    super();
  }

  override run(contributor: Contributor): Outcome {
    contributor.contribute("amount", this.price - this.discount.unwrapOr(0));
    return this.succeed();
  }
}

/** The pricing, under its name, with the requests of its parameters. */
const pricing = Price.builder("price")
  .needs(input("price", integer), input("discount", optional(integer)))
  .run(contributor());
// ANCHOR_END: price

/**
 * Charges an order's amount to its card through the payment client, and gives
 * back its receipt, and the gift card's code when the order is a gift card.
 * It gives every reporter the receipt as soon as the bank gives it.
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

  override async run(
    @service("payments", PAYMENTS) payments: Payments,
    @contributor() contributor: Contributor,
    @stepReporter() reporter: StepReporter,
  ): Promise<Outcome> {
    if (this.amount <= 0) return this.fail("nothing-to-charge");
    const receipt = await payments.charge(this.order, this.card, this.amount);
    if (receipt === undefined) return this.fail("card-expired");
    await reporter.setGlobalMetadata("receipt", receipt);
    contributor.contribute("receipt", receipt);
    if (this.giftCard) contributor.contribute("code", `G-${this.order}`);
    return this.succeed();
  }
}

/**
 * Repairs a charge that failed because its card had expired: it asks the
 * bank's card registry for the card that replaced it, and gives it back as
 * the order's card.
 */
@rescueStep()
class RenewCard extends RescueStep {
  constructor(
    @input("card", string) private readonly expired: string,
    @failureReason() private readonly reason: Reason,
  ) {
    super();
  }

  override async run(
    @service("cards", CARDS) registry: Cards,
    @contributor() contributor: Contributor,
  ): Promise<RescueOutcome> {
    if (this.reason.code !== "card-expired") {
      return this.fail("not-repairable");
    }
    const card = await registry.replacement(this.expired);
    if (card === undefined) return this.fail("not-replaced");
    contributor.contribute("card", card);
    return this.succeed();
  }
}

// ANCHOR: audit
/**
 * Writes an audit line for every step it is attached to that succeeds: the
 * journey, the attempt that took the money, the amount the order was charged,
 * and the receipt the step gave back.
 */
class Audit extends StepPolicy {
  charged(
    journey: JourneyId,
    step: StepName,
    attempt: Attempt,
    amount: number,
    receipt: string,
    contributor: Contributor,
  ): OnSuccess {
    contributor.contribute(
      "audit",
      `${journey}: ${step} took ${String(amount)} on attempt ${String(attempt)}, receipt ${receipt}`,
    );
    return this.keepDefault();
  }
}

/** The audit, under its name, with its hook and the requests of its parameters. */
const audit = Audit.builder("audit").onStepSuccess(
  "charged",
  journeyId(),
  stepName(),
  attempt(),
  fromWorkflow("amount", integer),
  fromStep("receipt", string),
  contributor(),
);
// ANCHOR_END: audit

/** What the shop does for its policies: tell a customer something. */
interface Messages {
  tell(customer: string, message: string): Promise<void>;
}

/** What the shop does for its policies: give an order's money back. */
interface Refunds {
  refund(receipt: string, amount: number): Promise<void>;
}

/** Tells the customer how their order went, once its journey has ended. */
@workflowPolicy({ name: "notify" })
class Notify extends WorkflowPolicy<Messages> {
  @onWorkflowSuccess()
  async succeeded(
    @fromWorkflow("customer", string) customer: string,
    @fromWorkflow("order", string) order: string,
    @fromWorkflow("tracking", optional(string)) tracking: Option<string>,
    @fromWorkflow("code", optional(string)) code: Option<string>,
    @role<Messages>() messages: Messages,
  ): Promise<void> {
    let message = `Order ${order} is complete.`;
    if (tracking.isSome()) {
      message = `Order ${order} is on its way, tracking ${tracking.value}.`;
    } else if (code.isSome()) {
      message = `Order ${order} is paid: your gift card's code is ${code.value}.`;
    }
    await messages.tell(customer, message);
  }

  @onWorkflowFailure()
  async failed(
    @fromWorkflow("customer", string) customer: string,
    @fromWorkflow("order", string) order: string,
    @role<Messages>() messages: Messages,
  ): Promise<void> {
    await messages.tell(
      customer,
      `Sorry, order ${order} could not be completed.`,
    );
  }
}

/**
 * Gives up a step that ends in an error at once: the bank's network is down,
 * and neither a retry nor a rescue can mend that. Since the bank may have
 * taken the money before its network failed, it writes an audit line with the
 * error, for the accountants to check with the bank.
 */
@stepPolicy({ name: "outage" })
class Outage extends StepPolicy {
  @onStepAbnormalTermination()
  endedInError(
    @journeyId() journey: JourneyId,
    @stepName() step: StepName,
    @attempt() attempt: Attempt,
    @error() thrown: unknown,
    @fromWorkflow("amount", integer) amount: number,
    @contributor() contributor: Contributor,
  ): OnFailure {
    const message = thrown instanceof Error ? thrown.message : String(thrown);
    contributor.contribute(
      "audit",
      `${journey}: ${step} ended in an error on attempt ${String(attempt)} (${message}); check whether the bank took ${String(amount)}`,
    );
    return this.failWorkflow("payment-network-down");
  }
}

/**
 * Deals with a shipping that fails for good. It gives up a parcel over the
 * shop's limit rather than try it again, since the limit is the same on every
 * attempt, whichever carrier comes. And whatever gave the shipping up, it
 * refunds the order's charge.
 */
@stepPolicy({ name: "failedShipping" })
class FailedShipping extends StepPolicy<Refunds> {
  @onStepRetry()
  async beforeRetrying(
    @failureReason({ optional: true }) reason: Option<Reason>,
    @fromWorkflow("receipt", string) receipt: string,
    @fromWorkflow("amount", integer) amount: number,
    @role<Refunds>() refunds: Refunds,
    @contributor() contributor: Contributor,
  ): Promise<OnFailure> {
    if (reason.isSome() && reason.value.code === "over-parcel-limit") {
      await this.refund(
        refunds,
        contributor,
        receipt,
        amount,
        "over-parcel-limit",
      );
      return this.failWorkflow("no-carrier-can-take-it");
    }
    return this.keepDefault();
  }

  @onStepFailure()
  async givenUp(
    @failureCause() cause: StepFailureCause,
    @fromWorkflow("receipt", string) receipt: string,
    @fromWorkflow("amount", integer) amount: number,
    @role<Refunds>() refunds: Refunds,
    @contributor() contributor: Contributor,
  ): Promise<OnFailure> {
    await this.refund(refunds, contributor, receipt, amount, cause);
    return this.keepDefault();
  }

  /** Gives the order's money back, and records why. */
  private async refund(
    refunds: Refunds,
    contributor: Contributor,
    receipt: string,
    amount: number,
    why: string,
  ): Promise<void> {
    await refunds.refund(receipt, amount);
    contributor.contribute(
      "refund",
      `${String(amount)} back on ${receipt}: ${why}`,
    );
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
    const tracking = await carrier.book(this.order, this.weight);
    contributor.contribute("tracking", tracking);
    if (!(await carrier.collects(this.weight))) return this.fail("too-heavy");
    await reporter.info(`the parcel leaves with receipt ${this.receipt}`);
    return this.succeed();
  }
}

/** The logging service's error: it can't take a line. */
class LoggingDown extends Error {
  constructor() {
    super("the logging service is down");
  }
}

/**
 * The logging service's error when a journey's lines have no customer to be
 * filed under.
 */
class NoCustomer extends Error {
  constructor() {
    super("the logging service files no line without a customer's name");
  }
}

/**
 * The shop's logging service, faked: it prints each line it takes, once the
 * network has answered. It is down when the line saying that order A-15's
 * charge succeeded reaches it.
 */
class LoggingService {
  /**
   * Files the journey's lines under a customer's name, which must not be
   * empty.
   */
  async fileUnder(customer: string): Promise<void> {
    await network();
    if (customer === "") throw new NoCustomer();
  }

  async write(line: string): Promise<void> {
    await network();
    if (line.includes(" A-15 ") && line.endsWith(" step_succeeded charge")) {
      throw new LoggingDown();
    }
    console.log(line);
  }
}

/**
 * The shop's log: it writes a line for every event of a journey to the
 * logging service, starting with the workflow's name, the journey's ID, the
 * customer's name and, once the bank has given it, the receipt. A line the
 * service refuses fails the report.
 */
class Log implements Reporter {
  private readonly service = new LoggingService();
  private readonly metadata = new Map<string, PlainValue>();

  async receiveGlobalMetadata(key: string, value: PlainValue): Promise<void> {
    this.metadata.set(key, value);
    if (key === "customer") {
      await this.service.fileUnder(typeof value === "string" ? value : "");
    }
  }

  async report(event: Event): Promise<void> {
    const said = detail(event);
    const line = `log ${event.workflow} ${event.journeyId} ${this.whose()} #${String(event.sequence)} ${event.kind}`;
    await this.service.write(said === undefined ? line : `${line} ${said}`);
  }

  /**
   * The customer the journey is for and, once the bank has given it, the
   * charge's receipt.
   */
  private whose(): string {
    const customer = this.text("customer") ?? "someone";
    const receipt = this.text("receipt");
    if (receipt === undefined) return `for ${customer}`;
    return `for ${customer}, receipt ${receipt}`;
  }

  /** The entry of `key`, when it is text. */
  private text(key: string): string | undefined {
    const value = this.metadata.get(key);
    return typeof value === "string" ? value : undefined;
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
      return `${event.step} gets ${event.key} from ${event.provider}, a ${event.implementation}${supplied(event.data)}`;
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
    case "input_adapter_supplied":
      return `${event.step} gets ${event.key} from ${event.adapter}`;
    case "global_metadata_set":
      return event.key;
    case "journey_succeeded":
      if (event.decidedBy === "default") return undefined;
      return `FinishWorkflow from ${event.decidedBy.policy}`;
    default:
      return undefined;
  }
}

/** The data a provider supplied with a service, when it is text. */
function supplied(data: PlainValue | undefined): string {
  return typeof data === "string" ? `, with ${data}` : "";
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
 * How the warehouse describes an order's parcel: its weight in kg, and its
 * length, width and height in cm.
 */
interface Parcel {
  weight: number;
  size: number[];
}

/** The type token of a parcel. */
const PARCEL = objectOf({ weight: integer, size: listOf(integer) });

// ANCHOR: parts
/**
 * The shop's workflow class: the roles its policies ask for, through the
 * channel the shop configures, and the methods that name its journeys,
 * adapt the shipping's inputs and provide its services.
 */
class Orders extends Workflow implements Messages, Refunds {
  constructor(private readonly channel: string) {
    super();
  }

  async tell(customer: string, message: string): Promise<void> {
    await network();
    console.log(`${this.channel} to ${customer}: ${message}`);
  }

  async refund(receipt: string, amount: number): Promise<void> {
    await network();
    console.log(`the bank gives ${String(amount)} back on receipt ${receipt}`);
  }

  /** Names each journey after its order. */
  orderNumber(data: DataBag): string {
    return data.read("order", string).option().expect("the order's number");
  }

  /**
   * Gives the shipping the weight of the order's parcel, as the warehouse
   * describes it. Every other input of the shipping is read from the data bag.
   */
  parcels(key: string, parcel: Parcel): Supplied {
    if (key !== "weight") return this.fromDataBag();
    return this.supply(parcel.weight);
  }

  /** The bank, as the payment client of the journey. */
  bank(): Provided<Payments> {
    return this.provide(new Bank(2026));
  }

  /**
   * A carrier for every attempt of a step: the main carrier first, the backup
   * carrier after.
   */
  carriers(attempt: Attempt): Provided<Carrier> {
    if (attempt.isFirst()) return this.provide(new MainCarrier());
    return this.provide(new BackupCarrier());
  }

  /** The bank's card registry in Europe, for the charge's rescue step only. */
  cards(): Provided<Cards> {
    return this.provide(cardRegistry("eu"), "eu");
  }

  /** The order book, which has the orders completed before today. */
  ledger(): Provided<OrderBook> {
    return this.provide(new Ledger(["A-13"]));
  }
}
// ANCHOR_END: parts

// ANCHOR: workflow
/**
 * The shop's workflow: it looks the order up, and finishes an order already
 * completed. Otherwise it works out what the order costs, then charges it,
 * repairing a charge that fails and auditing one that succeeds, then ships,
 * and shipping may be tried once more, its charge refunded if it is given up.
 * Once a journey has ended, the customer is told, through the channel the shop
 * configures.
 */
const shop = Orders.builder()
  .name("orders")
  .reporter(Log)
  .reporter(Desk)
  .policy(Notify)
  .step(Lookup.builder("lookup").policy(AlreadyCompleted))
  .step(pricing)
  .step(Charge.builder("charge").rescue(RenewCard).policy(audit).policy(Outage))
  .step(Ship.builder("ship").retryBudget(1).policy(FailedShipping))
  .idGenerator("orderNumber", dataBag())
  .inputAdapter(
    "parcels",
    { steps: ["ship"] },
    key(),
    fromWorkflow("parcel", PARCEL),
  )
  .provider("bank", PAYMENTS, { key: "payments" })
  .provider(
    "carriers",
    CARRIER,
    { key: "carrier", perAttempt: true },
    attempt(),
  )
  .provider("cards", CARDS, { key: "cards", steps: ["charge_rescue"] })
  .provider("ledger", ORDER_BOOK, { key: "orderBook" })
  .constructWith("email")
  .build();
// ANCHOR_END: workflow

/**
 * What the shop reads back once the journey has ended: the type of the value
 * under each name.
 */
const orderOutput = {
  receipt: optional(string),
  tracking: optional(string),
  audit: optional(string),
};

/** Says how an order's journey ended, and what explains it. */
function followUp(result: JourneyResult): void {
  if (result.isAborted()) {
    console.log(
      `the journey was aborted (${whatWentWrong(result.status.abort)})`,
    );
    return;
  }
  const data = result.output(orderOutput);
  const receipt = data.receipt.unwrapOr("none");
  if (result.isSucceeded()) {
    const tracking = data.tracking.unwrapOr("none");
    console.log(
      `the journey succeeded, with receipt ${receipt} and tracking ${tracking}`,
    );
    console.log(`audit: ${data.audit.unwrapOr("none")}`);
  }
  if (result.isFailed()) {
    console.log(
      `the journey failed (${why(result.status.failure)}), with receipt ${receipt}`,
    );
    console.log(`audit: ${data.audit.unwrapOr("none")}`);
  }
}

/**
 * The cause of a failure, with the reason a step gave for it when it gave
 * one.
 */
function why(failure: Failure): string {
  if (failure.cause === "failure") {
    return `${failure.cause}: ${failure.reason.code}`;
  }
  return failure.cause;
}

/**
 * The reason of an abort, with the key that was missing or the reporter's
 * error when that is what happened.
 */
function whatWentWrong(abort: Abort): string {
  if (
    abort.reason === "required data missing" &&
    abort.missing.kind === "key"
  ) {
    return `${abort.reason}: ${abort.missing.key}`;
  }
  if (abort.reason === "reporter failed") {
    return `${abort.reason}: ${abort.errorMessage}`;
  }
  return abort.reason;
}

const executor = new LocalExecutor();
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
    order: "A-8",
    customer: "Jo",
    price: 30,
    parcel: { weight: 1, size: [20, 15, 5] },
    card: "jo-2024",
    giftCard: false,
  },
];
for (const order of orders) {
  console.log(`${order.customer}'s order:`);
  const instance = shop.instance(order, {
    reportersGlobalMetadata: { customer: order.customer },
  });
  followUp(await executor.run(instance));
}
