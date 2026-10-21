// Chapter 1's checkout with its payment moved into a workflow: the charge is a step, renewing an
// expired card is its rescue step and the audit is a policy, while the bank's client, the audit
// trail and the log stay as they were, and the rest of the checkout awaits the payment.

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
import { integer, string } from "itinera/value";
import {
  fromWorkflow,
  idGenerator,
  withStep,
  Workflow,
  workflow,
} from "itinera/workflow";

// The outside world the shop talks to, faked so that every run is the same.

/** Why the bank refused a charge: the card has expired, or the bank refused it. */
type Why = "expired card" | "refused";

/** The bank refused a charge. */
class Declined extends Error {
  constructor(readonly why: Why) {
    super(why === "expired card" ? "card expired" : "payment declined");
  }
}

/**
 * The customers' bank. A card ends with the year it expires, such as `ann-card-2028`; the bank
 * refuses a card that expired before this year, and any charge above its limit.
 */
class Bank {
  constructor(
    private readonly thisYear: number,
    private readonly limit: number,
  ) {}

  /** Charges an amount to a card, and returns the receipt. */
  charge(card: string, amount: number): string {
    if (expiry(card) < this.thisYear) throw new Declined("expired card");
    if (amount > this.limit) throw new Declined("refused");
    return `R-${card}-${String(amount)}`;
  }

  /** The card that replaced an expired one: the same card, expiring three years from now. */
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

/** The warehouse, which holds the stock. */
class Warehouse {
  constructor(private readonly outOfStock: readonly string[]) {}

  /** Sets the order's goods aside. */
  reserve(order: string): void {
    if (this.outOfStock.includes(order)) {
      throw new Error(`nothing left for ${order}`);
    }
  }
}

/** A carrier, which takes parcels up to a weight. */
class Carrier {
  constructor(
    readonly name: string,
    private readonly trackingPrefix: string,
    private readonly maxWeight: number,
  ) {}

  /** Takes the order's parcel, and returns its tracking number. */
  ship(order: string, weight: number): string {
    if (weight > this.maxWeight) {
      throw new Error(
        `${String(weight)} kg is over its ${String(this.maxWeight)} kg limit`,
      );
    }
    return `${this.trackingPrefix}-${order}`;
  }
}

/** Sends messages to customers. */
class Messages {
  /** Tells a customer something. */
  tell(customer: string, message: string): void {
    console.log(`to ${customer}: ${message}`);
  }
}

// The shop's logging, used here and there.

const log = {
  info(message: string): void {
    console.log(`INFO ${message}`);
  },
  warn(message: string): void {
    console.log(`WARN ${message}`);
  },
};

/** What was thrown, as a line of the log. */
function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

// The shop's audit trail.

const audit = {
  record(entry: string): void {
    console.log(`AUDIT ${entry}`);
  },
};

// Taking the payment for an order, as a workflow.

// ANCHOR: payments
/** What the payment's steps ask of the bank. `charge` throws `Declined` when the bank refuses. */
interface Payments {
  charge(card: string, amount: number): string;
  renewedCard(card: string): string;
}

const PAYMENTS = serviceType<Payments>("Payments");
// ANCHOR_END: payments

// ANCHOR: steps
/** Charges the amount to the card, and gives back the receipt. */
@step()
class Charge extends Step {
  constructor(
    @input("card", string) private readonly card: string,
    @input("amount", integer) private readonly amount: number,
  ) {
    super();
  }

  override run(
    @service("payments", PAYMENTS) payments: Payments,
    @contributor() contributor: Contributor,
  ): Outcome {
    let receipt: string;
    try {
      receipt = payments.charge(this.card, this.amount);
    } catch (error) {
      if (!(error instanceof Declined)) throw error;
      if (error.why === "expired card") return this.fail("card-expired");
      return this.fail("declined");
    }
    contributor.contribute("receipt", receipt);
    return this.succeed();
  }
}

/** Repairs a charge whose card had expired: it asks the bank for the card that replaced it. */
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
    if (this.reason.code !== "card-expired") return this.fail("not-repairable");
    contributor.contribute("card", payments.renewedCard(this.card));
    return this.succeed();
  }
}
// ANCHOR_END: steps

// ANCHOR: audit
/** Writes every successful charge to the shop's audit trail. */
@stepPolicy({ name: "audit" })
class Audit extends StepPolicy {
  @onStepSuccess()
  charged(
    @fromWorkflow("amount", integer) amount: number,
    @fromStep("receipt", string) receipt: string,
  ): OnSuccess {
    audit.record(`charged ${String(amount)}, receipt ${receipt}`);
    return this.keepDefault();
  }
}
// ANCHOR_END: audit

// ANCHOR: log
/** Writes to the shop's log when a step fails, and when a rescue step repairs it. */
class Log implements Reporter {
  report(event: Event): void {
    if (event.kind === "step_failed") {
      log.info(
        `payment ${event.journeyId}: ${event.step} failed, ${event.reason.code}`,
      );
    }
    if (event.kind === "step_rescued") {
      log.info(
        `payment ${event.journeyId}: ${event.step} repaired by ${event.rescue}`,
      );
    }
  }
}
// ANCHOR_END: log

// ANCHOR: workflow
/** Charges an order, renewing a card that has expired, and audits the charge. */
@workflow({ name: "payment", reporters: [Log] })
@withStep(Charge, { name: "charge", rescue: RenewCard, policies: [Audit] })
class Payment extends Workflow {
  /** Names each journey after its order. */
  @idGenerator()
  orderNumber(@dataBag() data: DataBag): string {
    return data.read("order", string).option().expect("the order's number");
  }

  /** The bank, as it is configured today. */
  @provider(PAYMENTS, { key: "payments" })
  bank(): Provided<Payments> {
    return this.provide(new Bank(2026, 1000));
  }
}

/** What a payment gives back: the type of the value under each name. */
const paid = { receipt: string };

const payment = Payment.build();
// ANCHOR_END: workflow

// Shipping an order.

/**
 * Ships through the main carrier, or through the backup carrier when the main one cannot take
 * the parcel. Returns the tracking number, or `undefined` when neither can take it.
 */
function ship(
  main: Carrier,
  backup: Carrier,
  order: string,
  weight: number,
): string | undefined {
  try {
    return main.ship(order, weight);
  } catch (error) {
    log.warn(
      `${main.name} cannot take ${order}: ${messageOf(error)}; trying ${backup.name}`,
    );
    try {
      return backup.ship(order, weight);
    } catch {
      return undefined;
    }
  }
}

// The checkout: what happens to an order once the customer has paid.

// ANCHOR: checkout
/** An order, as the shop's website hands it over. */
interface Order {
  readonly order: string;
  readonly customer: string;
  readonly card: string;
  readonly price: number;
  readonly discount?: number;
  readonly weight: number;
}

/** The payment workflow ended without saying how the payment went. */
class PaymentNotRun extends Error {
  constructor(why: string) {
    super(`the payment did not run: ${why}`);
  }
}

/** The shop, with everything it talks to. */
class Shop {
  constructor(
    private readonly warehouse: Warehouse,
    private readonly main: Carrier,
    private readonly backup: Carrier,
    private readonly messages: Messages,
  ) {}

  /** Reserves, charges and ships an order, and tells the customer how it went. */
  async checkout(order: Order): Promise<string> {
    log.info(`checkout ${order.order}`);
    try {
      const tracking = await this.reserveChargeAndShip(order);
      this.messages.tell(
        order.customer,
        `order ${order.order} is on its way: ${tracking}`,
      );
      return tracking;
    } catch (error) {
      log.warn(`checkout ${order.order} failed: ${messageOf(error)}`);
      this.messages.tell(
        order.customer,
        `order ${order.order} could not be completed`,
      );
      throw error;
    }
  }

  private async reserveChargeAndShip(order: Order): Promise<string> {
    this.warehouse.reserve(order.order);
    const amount = order.price - (order.discount ?? 0);
    await this.pay(order.order, order.card, amount);
    const tracking = ship(this.main, this.backup, order.order, order.weight);
    if (tracking === undefined) throw new Error("no carrier could take it");
    return tracking;
  }
  // ANCHOR_END: checkout

  // ANCHOR: pay
  /** Charges an order's card through the payment workflow, and gives back the receipt. */
  private async pay(
    order: string,
    card: string,
    amount: number,
  ): Promise<string> {
    const instance = payment.instance({ order, card, amount });
    const result = await new LocalExecutor().run(instance);
    if (result.isSucceeded()) return result.output(paid).receipt;
    if (result.isFailed()) {
      const failure = result.status.failure;
      if (
        failure.cause === "failure" &&
        failure.reason.code === "card-expired"
      ) {
        throw new Declined("expired card");
      }
      throw new Declined("refused");
    }
    throw new PaymentNotRun(result.status.kind);
  }
  // ANCHOR_END: pay
}

// ANCHOR: main
const shop = new Shop(
  new Warehouse([]),
  new Carrier("Rapid", "T", 20),
  new Carrier("Steady", "B", 50),
  new Messages(),
);
const orders: Order[] = [
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
];
for (const order of orders) {
  try {
    console.log(`${order.order}: shipped, ${await shop.checkout(order)}`);
  } catch (error) {
    console.log(`${order.order}: failed, ${messageOf(error)}`);
  }
  console.log();
}
// ANCHOR_END: main
