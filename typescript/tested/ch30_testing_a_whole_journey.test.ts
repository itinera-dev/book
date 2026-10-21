// The tests of chapter 26's shop for chapter 30: each runs one of Ann's orders
// through the shop's real steps and policies, with a bank and a carrier faked
// for it, and compares its journey's path with the one the shop approved.

import assert from "node:assert/strict";
import { mock, test } from "node:test";
import { LocalExecutor } from "itinera/executor";
import { fakeProvider, RecordingReporter } from "itinera/testing";
import {
  CARRIER,
  type Carrier,
  NetworkDown,
  Orders,
  PAYMENTS,
  type Payments,
} from "./ch30_testing_a_whole_journey.js";

mock.method(console, "log", () => undefined);

// ANCHOR: fakes
/** A bank for tests: it charges every card, or its network is down. */
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
// ANCHOR_END: fakes

// ANCHOR: path
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
  const instance = Orders.build("email").instance(
    {
      order,
      customer: "Ann",
      price: 30,
      parcel: { weight, size: [20, 15, 5] },
      card: "ann-2028",
      giftCard: false,
    },
    { reporters: [recorder] },
  );
  fakeProvider(instance, "bank", PAYMENTS, (provide) => provide(bank));
  fakeProvider(instance, "carriers", CARRIER, (provide) => provide(carrier));
  await new LocalExecutor().run(instance);
  return recorder.path;
}
// ANCHOR_END: path

// ANCHOR: approved
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
// ANCHOR_END: approved
