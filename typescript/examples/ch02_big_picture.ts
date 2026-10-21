// The shop's workflow, whole: three steps, a rescue step for the charge, a
// retry for shipping with the backup carrier on the second attempt, the audit
// and notify policies, and the services the steps use.

import type { Event } from "itinera/event";
import { LocalExecutor } from "itinera/executor";
import { type Contributor, contributor, type Failure } from "itinera/journey";
import type { Option } from "itinera/option";
import {
  fromStep,
  type HookReporter,
  hookReporter,
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
  type Attempt,
  attempt,
  failureReason,
  input,
  type Outcome,
  type Reason,
  RescueStep,
  rescueStep,
  type RescueOutcome,
  Step,
  step,
} from "itinera/step";
import { integer, optional, string } from "itinera/value";
import {
  fromWorkflow,
  type ListedService,
  type ListedStep,
  withStep,
  Workflow,
  workflow,
} from "itinera/workflow";

// ANCHOR: services
/** Why the bank refused a charge. */
type Declined = "expired card" | "refused";

/**
 * A payment client: it charges a card, giving back the receipt or why the bank
 * refused, and knows the card that replaced an expired one.
 */
interface Payments {
  charge(card: string, amount: number): { receipt: string } | Declined;
  renewedCard(card: string): string;
}

/** A stock store: it reserves an order's goods, then hands them over. */
interface StockStore {
  reserve(order: string): boolean;
  handOver(order: string): void;
}

/**
 * A carrier: it takes a parcel and gives back its tracking number, or
 * `undefined` when it cannot take it.
 */
interface Carrier {
  send(order: string, weight: number): string | undefined;
}

const PAYMENTS = serviceType<Payments>("Payments");
const STOCK_STORE = serviceType<StockStore>("StockStore");
const CARRIER = serviceType<Carrier>("Carrier");
// ANCHOR_END: services

/**
 * The customers' bank, faked. A card ends with the year it expires, such as
 * `ann-card-2028`; the bank refuses a card that expired before this year, and
 * any charge above its limit.
 */
class Bank implements Payments {
  constructor(
    private readonly thisYear: number,
    private readonly limit: number,
  ) {}

  charge(card: string, amount: number): { receipt: string } | Declined {
    if (expiry(card) < this.thisYear) return "expired card";
    if (amount > this.limit) return "refused";
    return { receipt: `R-${card}-${String(amount)}` };
  }

  renewedCard(card: string): string {
    return `${holder(card)}-${String(this.thisYear + 3)}`;
  }
}

/** The year a card expires, from the end of its number. */
function expiry(card: string): number {
  return Number(card.slice(card.lastIndexOf("-") + 1));
}

/** A card's number without the year it expires. */
function holder(card: string): string {
  return card.slice(0, card.lastIndexOf("-"));
}

/** The warehouse, faked: it lists the orders it has no stock for. */
class Warehouse implements StockStore {
  constructor(private readonly outOfStock: readonly string[]) {}

  reserve(order: string): boolean {
    return !this.outOfStock.includes(order);
  }

  handOver(): void {}
}

/** A carrier, faked: it takes parcels up to a weight. */
class Van implements Carrier {
  constructor(
    private readonly trackingPrefix: string,
    private readonly maxWeight: number,
  ) {}

  send(order: string, weight: number): string | undefined {
    if (weight > this.maxWeight) return undefined;
    return `${this.trackingPrefix}-${order}`;
  }
}

const BANK = new Bank(2026, 1000);
const WAREHOUSE = new Warehouse(["A-5"]);
const MAIN_CARRIER = new Van("T", 20);
const BACKUP_CARRIER = new Van("B", 50);

// ANCHOR: steps
/** Reserves the order's goods in the stock store. */
@step()
class Reserve extends Step {
  constructor(@input("order", string) private readonly order: string) {
    super();
  }

  override run(@service("stock", STOCK_STORE) stock: StockStore): Outcome {
    if (!stock.reserve(this.order)) return this.fail("out-of-stock");
    return this.succeed();
  }
}

/**
 * Charges the card the price, less any discount. It only says how it went:
 * what happens after a declined card is not its business.
 */
@step()
class Charge extends Step {
  constructor(
    @input("card", string) private readonly card: string,
    @input("price", integer) private readonly price: number,
    @input("discount", optional(integer))
    private readonly discount: Option<number>,
  ) {
    super();
  }

  override run(
    @service("payments", PAYMENTS) payments: Payments,
    @contributor() contributor: Contributor,
  ): Outcome {
    const amount = this.price - this.discount.unwrapOr(0);
    const charged = payments.charge(this.card, amount);
    if (charged === "expired card") return this.fail("expired-card");
    if (charged === "refused") return this.fail("declined");
    contributor.contribute("charged", amount);
    contributor.contribute("receipt", charged.receipt);
    return this.succeed();
  }
}

/**
 * The charge's rescue step: asks the customer's bank for the card that
 * replaced an expired one. A charge that failed for any other reason, or
 * ended in an abnormal termination, which has no reason, cannot be repaired
 * this way.
 */
@rescueStep()
class RenewCard extends RescueStep {
  constructor(
    @input("card", string) private readonly card: string,
    @failureReason({ optional: true }) private readonly reason: Option<Reason>,
  ) {
    super();
  }

  override run(
    @service("payments", PAYMENTS) payments: Payments,
    @contributor() contributor: Contributor,
  ): RescueOutcome {
    if (this.reason.mapOr("", (reason) => reason.code) !== "expired-card") {
      return this.fail("nothing-to-renew");
    }
    contributor.contribute("card", payments.renewedCard(this.card));
    return this.succeed();
  }
}

/**
 * Ships the order through the carrier it is given, without knowing which one
 * it is. A carrier that cannot take the parcel might be replaced on another
 * try, so that failure is worth trying again.
 */
@step()
class Ship extends Step {
  constructor(
    @input("order", string) private readonly order: string,
    @input("weight", integer) private readonly weight: number,
  ) {
    super();
  }

  override run(
    @service("carrier", CARRIER) carrier: Carrier,
    @service("stock", STOCK_STORE) stock: StockStore,
    @contributor() contributor: Contributor,
  ): Outcome {
    const tracking = carrier.send(this.order, this.weight);
    if (tracking === undefined) return this.fail("parcel-refused");
    stock.handOver(this.order);
    contributor.contribute("tracking", tracking);
    return this.succeed();
  }
}
// ANCHOR_END: steps

// ANCHOR: policies
/**
 * Audits every successful charge, in the journey's events, including one
 * after a rescue.
 */
@stepPolicy({ name: "audit" })
class Audit extends StepPolicy {
  @onStepSuccess()
  async audited(
    @fromStep("charged", integer) charged: number,
    @fromStep("receipt", string) receipt: string,
    @hookReporter() reporter: HookReporter,
  ): Promise<OnSuccess> {
    await reporter.info(`charged ${String(charged)}, receipt ${receipt}`);
    return this.keepDefault();
  }
}

/** A role the shop provides: telling a customer something. */
interface Messages {
  tell(customer: string, message: string): void;
}

/** Tells the customer how the order went, once, when the journey ends. */
@workflowPolicy({ name: "notify" })
class Notify extends WorkflowPolicy<Messages> {
  @onWorkflowSuccess()
  shipped(
    @fromWorkflow("customer", string) customer: string,
    @fromWorkflow("order", string) order: string,
    @fromWorkflow("tracking", string) tracking: string,
    @role<Messages>() messages: Messages,
  ): void {
    messages.tell(customer, `order ${order} is on its way: ${tracking}`);
  }

  @onWorkflowFailure()
  failed(
    @fromWorkflow("customer", string) customer: string,
    @fromWorkflow("order", string) order: string,
    @role<Messages>() messages: Messages,
  ): void {
    messages.tell(customer, `order ${order} could not be completed`);
  }
}
// ANCHOR_END: policies

/**
 * The shop's log: every failure, every decision and what the hooks report,
 * for every journey, in one format.
 */
class Log implements Reporter {
  report(event: Event): void {
    switch (event.kind) {
      case "step_failed":
        console.log(
          `  WARN ${event.step}, attempt ${String(event.attempt)}: ${event.reason.code}`,
        );
        break;
      case "step_retrying":
        console.log(`  INFO ${event.step} tried again`);
        break;
      case "step_rescuing":
        console.log(`  INFO ${event.step} to be rescued by ${event.rescue}`);
        break;
      case "step_given_up":
        console.log(`  WARN ${event.step} given up: ${event.cause}`);
        break;
      case "journey_info":
        if (event.source.kind === "step hook") {
          console.log(`  INFO ${event.source.policy}: ${event.message}`);
        }
        break;
    }
  }
}

// ANCHOR: workflow
/**
 * The shop's workflow: reserve, charge and ship, auditing the charge,
 * rescuing it when the card has expired, and telling the customer how the
 * order went.
 */
@workflow({ name: "orders", policies: [Notify], reporters: [Log] })
// The steps run in this order, and nothing reorders them.
@withStep(Reserve, { name: "reserve" })
// The charge runs at most twice: once, and once more after its rescue step
// succeeds. Every successful charge is audited.
@withStep(Charge, { name: "charge", rescue: RenewCard, policies: [Audit] })
// Shipping may be tried once more, after a failure worth trying again.
@withStep(Ship, { name: "ship", retryBudget: 1 })
class Orders extends Workflow implements Messages {
  /** One payment client for the journey, used by the charge and its rescue. */
  @provider(PAYMENTS, { key: "payments" })
  bank(): Provided<Payments> {
    return this.provide(BANK);
  }

  /** One stock store for the journey, shared by reserve and ship. */
  @provider(STOCK_STORE, { key: "stock" })
  warehouse(): Provided<StockStore> {
    return this.provide(WAREHOUSE);
  }

  /** A carrier for every attempt: the main carrier first, the backup after. */
  @provider(CARRIER, { perAttempt: true })
  carrier(@attempt() attempt: Attempt): Provided<Carrier> {
    return this.provide(attempt.isFirst() ? MAIN_CARRIER : BACKUP_CARRIER);
  }

  /** How the shop tells a customer something, for notify: by printing it. */
  tell(customer: string, message: string): void {
    console.log(`  to ${customer}: ${message}`);
  }
}
// ANCHOR_END: workflow

/** A service in the listing: its key, and the provider that serves it. */
function served(listed: ListedService): string {
  return `${listed.key} from ${listed.provider}`;
}

/** The services of a listing's entry, separated by commas. */
function services(listed: readonly ListedService[]): string {
  return listed.map(served).join(", ");
}

/**
 * One entry of the listing: a step's position and name, its policies, its
 * services with the provider that serves each, and its rescue step with its
 * services.
 */
function entry(listed: ListedStep): string {
  const policies =
    listed.policies.length > 0 ? `, with ${listed.policies.join(", ")}` : "";
  const rescue =
    listed.rescue === undefined
      ? ""
      : `; rescued by ${listed.rescue.step}, services: ${services(listed.rescue.services)}`;
  return `${String(listed.position)}. ${listed.step}${policies}; services: ${services(listed.services)}${rescue}`;
}

// ANCHOR: list
// The workflow lists its steps, their policies, services and rescue step,
// without running anything.
for (const listed of Orders.listing()) {
  console.log(entry(listed));
}
// ANCHOR_END: list

// ANCHOR: orders
// The order, the customer, the card, the price, the discount when there is
// one, and the parcel's weight in kg.
const orders = [
  {
    order: "A-1",
    customer: "Ann",
    card: "ann-card-2028",
    price: 40,
    discount: 5,
    weight: 2,
  },
  {
    order: "A-2",
    customer: "Ben",
    card: "ben-card-2024",
    price: 25,
    weight: 1,
  },
  {
    order: "A-3",
    customer: "Cat",
    card: "cat-card-2027",
    price: 30,
    weight: 30,
  },
  {
    order: "A-4",
    customer: "Dan",
    card: "dan-card-2025",
    price: 1200,
    weight: 3,
  },
  {
    order: "A-5",
    customer: "Eve",
    card: "eve-card-2027",
    price: 20,
    weight: 2,
  },
  {
    order: "A-6",
    customer: "Fin",
    card: "fin-card-2028",
    price: 1500,
    weight: 4,
  },
  {
    order: "A-7",
    customer: "Gil",
    card: "gil-card-2029",
    price: 45,
    weight: 60,
  },
];
const executor = new LocalExecutor();
const shop = Orders.build();
for (const order of orders) {
  console.log(`${order.order}:`);
  const result = await executor.run(shop.instance(order));
  if (result.isFailed()) {
    console.log(`failed: ${why(result.status.failure)}`);
  } else {
    console.log(result.status.kind);
  }
}
// ANCHOR_END: orders

/** Why a journey failed: its cause, and the reason a step gave when it gave one. */
function why(failure: Failure): string {
  return failure.reason === undefined
    ? failure.cause
    : `${failure.cause}, ${failure.reason.code}`;
}
