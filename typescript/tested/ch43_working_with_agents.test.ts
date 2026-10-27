// The tests of chapter 26's shop for chapter 43: after an agent changed the
// pricing, each approved path still holds, and a test runs the same orders
// through a proposed change to the flow, saying which paths it keeps and what
// it makes of the others.

import assert from "node:assert/strict";
import { mock, test } from "node:test";
import { LocalExecutor } from "itinera/executor";
import { type DataBag, dataBag } from "itinera/journey";
import { type Provided, provider } from "itinera/service";
import { fakeProvider, RecordingReporter } from "itinera/testing";
import { string } from "itinera/value";
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
import {
  AlreadyCompleted,
  Audit,
  Bank,
  CARRIER,
  type Carrier,
  Charge,
  Desk,
  FailedShipping,
  Ledger,
  Log,
  Lookup,
  MainCarrier,
  type Messages,
  NetworkDown,
  Notify,
  ORDER_BOOK,
  type OrderBook,
  Orders,
  Outage,
  PARCEL,
  type Parcel,
  PAYMENTS,
  type Payments,
  Price,
  type Refunds,
  RenewCard,
  Ship,
} from "./ch43_working_with_agents.js";

mock.method(console, "log", () => undefined);

/**
 * A bank for tests: it charges every card, or its network is down, and has
 * replaced no card.
 */
class TestBank implements Payments {
  constructor(private readonly answer: "charges" | "is down") {}

  async charge(order: string): Promise<string | undefined> {
    if (this.answer === "is down") throw new NetworkDown();
    return `R-${order}`;
  }

  async replacement(): Promise<string | undefined> {
    return undefined;
  }
}

/** A carrier for tests: it books every parcel, and collects every parcel or none. */
class TestCarrier implements Carrier {
  constructor(private readonly answer: "collects all" | "collects none") {}

  async book(order: string): Promise<string> {
    return `T-${order}`;
  }

  async collects(): Promise<boolean> {
    return this.answer === "collects all";
  }
}

// ANCHOR: proposed
/**
 * The shop's workflow as an agent proposes to change it: a parcel the carrier
 * won't collect is given up at once, without trying the backup carrier.
 */
@workflow({ name: "orders", reporters: [Log, Desk], policies: [Notify] })
@withStep(Lookup, { name: "lookup", policies: [AlreadyCompleted] })
@withStep(Price, { name: "price" })
@withStep(Charge, {
  name: "charge",
  rescue: RenewCard,
  policies: [Audit, Outage],
})
@withStep(Ship, { name: "ship", policies: [FailedShipping] })
class Proposed extends Workflow implements Messages, Refunds {
  async tell(customer: string, message: string): Promise<void> {
    console.log(`email to ${customer}: ${message}`);
  }

  async refund(receipt: string, amount: number): Promise<void> {
    console.log(`the bank gives ${String(amount)} back on receipt ${receipt}`);
  }

  /** Names each journey after its order. */
  @idGenerator()
  orderNumber(@dataBag() data: DataBag): string {
    return data.read("order", string).option().expect("the order's number");
  }

  /** Gives the shipping the weight of the order's parcel. */
  @inputAdapter({ steps: ["ship"] })
  parcels(
    @key() key: string,
    @fromWorkflow("parcel", PARCEL) parcel: Parcel,
  ): Supplied {
    if (key !== "weight") return this.fromDataBag();
    return this.supply(parcel.weight);
  }

  /** The order book, which has the orders completed before today. */
  @provider(ORDER_BOOK, { key: "orderBook" })
  ledger(): Provided<OrderBook> {
    return this.provide(new Ledger(["A-13"]));
  }

  /** The bank, as the payment client of the journey. */
  @provider(PAYMENTS, { key: "payments" })
  bank(): Provided<Payments> {
    return this.provide(new Bank(2026));
  }

  /** The main carrier. */
  @provider(CARRIER, { key: "carrier", perAttempt: true })
  carriers(): Provided<Carrier> {
    return this.provide(new MainCarrier());
  }
}
// ANCHOR_END: proposed

// ANCHOR: paths
/**
 * What the shop knows about one of Ann's orders, of 30, with a parcel of this
 * weight.
 */
function orderOf(order: string, weight: number) {
  return {
    order,
    customer: "Ann",
    price: 30,
    parcel: { weight, size: [20, 15, 5] },
    card: "ann-2028",
    giftCard: false,
  };
}

/**
 * Runs one of Ann's orders, of 30, with a parcel of this weight, through the
 * shop's real steps and policies, with the test's bank and carriers in place
 * of the shop's, and gives back its journey's path.
 */
async function pathOf(
  order: string,
  weight: number,
  bank: TestBank,
  carrier: TestCarrier,
): Promise<readonly string[]> {
  const recorder = new RecordingReporter();
  const instance = Orders.build("email").instance(orderOf(order, weight), {
    reporters: [recorder],
  });
  fakeProvider(instance, "bank", PAYMENTS, (provide) => provide(bank));
  fakeProvider(instance, "carriers", CARRIER, (provide) => provide(carrier));
  await new LocalExecutor().run(instance);
  return recorder.path;
}

/** Runs the same order as `pathOf`, through the proposed workflow. */
async function proposedPathOf(
  order: string,
  weight: number,
  bank: TestBank,
  carrier: TestCarrier,
): Promise<readonly string[]> {
  const recorder = new RecordingReporter();
  const instance = Proposed.build().instance(orderOf(order, weight), {
    reporters: [recorder],
  });
  fakeProvider(instance, "bank", PAYMENTS, (provide) => provide(bank));
  fakeProvider(instance, "carriers", CARRIER, (provide) => provide(carrier));
  await new LocalExecutor().run(instance);
  return recorder.path;
}
// ANCHOR_END: paths

test("an order the bank charges and the carrier takes is shipped", async () => {
  assert.deepEqual(
    await pathOf(
      "A-1",
      2,
      new TestBank("charges"),
      new TestCarrier("collects all"),
    ),
    [
      "lookup attempt 1 succeeded",
      "price attempt 1 succeeded",
      "charge attempt 1 succeeded",
      "ship attempt 1 succeeded",
      "journey succeeded",
    ],
  );
});

test("a parcel over the limit is given up after its first attempt", async () => {
  assert.deepEqual(
    await pathOf(
      "A-2",
      90,
      new TestBank("charges"),
      new TestCarrier("collects all"),
    ),
    [
      "lookup attempt 1 succeeded",
      "price attempt 1 succeeded",
      "charge attempt 1 succeeded",
      "ship attempt 1 failed: over-parcel-limit",
      "ship given up: FailWorkflow from failedShipping",
      "journey failed at ship: no-carrier-can-take-it",
    ],
  );
});

test("a charge whose bank is down is given up and never rescued", async () => {
  assert.deepEqual(
    await pathOf(
      "A-3",
      2,
      new TestBank("is down"),
      new TestCarrier("collects all"),
    ),
    [
      "lookup attempt 1 succeeded",
      "price attempt 1 succeeded",
      "charge attempt 1 terminated abnormally: the payment network is down",
      "charge given up: FailWorkflow from outage",
      "journey failed at charge: payment-network-down",
    ],
  );
});

test("a parcel no carrier collects is tried once more, then given up", async () => {
  assert.deepEqual(
    await pathOf(
      "A-4",
      60,
      new TestBank("charges"),
      new TestCarrier("collects none"),
    ),
    [
      "lookup attempt 1 succeeded",
      "price attempt 1 succeeded",
      "charge attempt 1 succeeded",
      "ship attempt 1 failed: too-heavy",
      "ship retrying",
      "ship attempt 2 failed: too-heavy",
      "ship given up: retries exhausted",
      "journey failed at ship: too-heavy",
    ],
  );
});

// ANCHOR: compared
test("the proposed flow keeps two approved paths and changes two", async () => {
  const kept = [
    {
      order: "A-1",
      weight: 2,
      bank: new TestBank("charges"),
      carrier: new TestCarrier("collects all"),
    },
    {
      order: "A-3",
      weight: 2,
      bank: new TestBank("is down"),
      carrier: new TestCarrier("collects all"),
    },
  ];
  for (const { order, weight, bank, carrier } of kept) {
    assert.deepEqual(
      await proposedPathOf(order, weight, bank, carrier),
      await pathOf(order, weight, bank, carrier),
    );
  }
  assert.deepEqual(
    await proposedPathOf(
      "A-2",
      90,
      new TestBank("charges"),
      new TestCarrier("collects all"),
    ),
    [
      "lookup attempt 1 succeeded",
      "price attempt 1 succeeded",
      "charge attempt 1 succeeded",
      "ship attempt 1 failed: over-parcel-limit",
      "ship given up: failure",
      "journey failed at ship: over-parcel-limit",
    ],
  );
  assert.deepEqual(
    await proposedPathOf(
      "A-4",
      60,
      new TestBank("charges"),
      new TestCarrier("collects none"),
    ),
    [
      "lookup attempt 1 succeeded",
      "price attempt 1 succeeded",
      "charge attempt 1 succeeded",
      "ship attempt 1 failed: too-heavy",
      "ship given up: failure",
      "journey failed at ship: too-heavy",
    ],
  );
});
// ANCHOR_END: compared
