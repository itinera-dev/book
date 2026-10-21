// The tests of chapter 26's shop for chapter 28: each calls a hook of one of
// its policies, or its workflow's input adapter, outside any journey.

import assert from "node:assert/strict";
import { test } from "node:test";
import { None, type Option, Some } from "itinera/option";
import type { OnFailure } from "itinera/policy";
import type { Reason, StepName } from "itinera/step";
import { Contributions } from "itinera/testing";
import { string } from "itinera/value";
import {
  Audit,
  FailedShipping,
  type Messages,
  Notify,
  Orders,
  type Parcel,
  type Refunds,
} from "./ch28_testing_a_hook_or_an_adapter_alone.js";

// ANCHOR: fakes
/**
 * Messages for tests: it keeps each message it is asked to send, with the
 * customer's name.
 */
class TestMessages implements Messages {
  readonly told: string[] = [];

  async tell(customer: string, message: string): Promise<void> {
    this.told.push(`${customer}: ${message}`);
  }
}

/** Refunds for tests: it keeps each refund it is asked to give, with its receipt. */
class TestRefunds implements Refunds {
  readonly given: string[] = [];

  async refund(receipt: string, amount: number): Promise<void> {
    this.given.push(`${String(amount)} back on ${receipt}`);
  }
}
// ANCHOR_END: fakes

// ANCHOR: failed_shipping
/**
 * What `beforeRetrying` decides for Ned's shipping, which failed with this
 * reason after he paid 50, and the refunds it asked for.
 */
async function beforeRetrying(
  reason: Option<Reason>,
): Promise<[OnFailure, string[]]> {
  const refunds = new TestRefunds();
  const contributions = new Contributions();
  const decision = await new FailedShipping().beforeRetrying(
    reason,
    "R-A-12",
    50,
    refunds,
    contributions.contributor,
  );
  return [decision, refunds.given];
}

test("failedShipping refunds and gives up a shipping that failed with over-parcel-limit", async () => {
  const [decision, refunds] = await beforeRetrying(
    Some({ code: "over-parcel-limit" }),
  );
  assert.deepEqual(decision, {
    kind: "FailWorkflow",
    reason: { code: "no-carrier-can-take-it" },
  });
  assert.deepEqual(refunds, ["50 back on R-A-12"]);
});

test("failedShipping lets a shipping that failed with too-heavy be retried", async () => {
  const [decision, refunds] = await beforeRetrying(Some({ code: "too-heavy" }));
  assert.deepEqual(decision, { kind: "KeepDefault" });
  assert.deepEqual(refunds, []);
});

test("failedShipping lets a shipping that ended in an error be retried", async () => {
  const [decision, refunds] = await beforeRetrying(None);
  assert.deepEqual(decision, { kind: "KeepDefault" });
  assert.deepEqual(refunds, []);
});

test("failedShipping refunds a shipping the executor gave up", async () => {
  const refunds = new TestRefunds();
  const contributions = new Contributions();
  const decision = await new FailedShipping().givenUp(
    "retries exhausted",
    "R-A-14",
    20,
    refunds,
    contributions.contributor,
  );
  assert.deepEqual(decision, { kind: "KeepDefault" });
  assert.deepEqual(refunds.given, ["20 back on R-A-14"]);
  assert.deepEqual(
    contributions.read("refund", string).option(),
    Some("20 back on R-A-14: retries exhausted"),
  );
});
// ANCHOR_END: failed_shipping

// ANCHOR: audit
test("audit writes the step, the amount and the receipt", () => {
  const contributions = new Contributions();
  new Audit().charged(
    "charge" as StepName,
    30,
    "R-A-1",
    contributions.contributor,
  );
  assert.deepEqual(
    contributions.read("audit", string).option(),
    Some("charge took 30, receipt R-A-1"),
  );
});
// ANCHOR_END: audit

// ANCHOR: notify
test("notify gives a gift card's code and says sorry for a failed order", async () => {
  const messages = new TestMessages();
  await new Notify().succeeded("Kim", "A-9", None, Some("G-A-9"), messages);
  await new Notify().failed("Lee", "A-10", messages);
  assert.deepEqual(messages.told, [
    "Kim: Order A-9 is paid: your gift card's code is G-A-9.",
    "Lee: Sorry, order A-10 could not be completed.",
  ]);
});
// ANCHOR_END: notify

// ANCHOR: adapter
/** Gil's parcel, as the warehouse describes it. */
const gilsParcel: Parcel = { weight: 60, size: [120, 80, 60] };

test("parcels gives the shipping the parcel's weight", () => {
  const supplied = Orders.build("email").parcels("weight", gilsParcel);
  assert.deepEqual(supplied, { kind: "supplied", value: 60 });
});

test("parcels leaves every other input to the data bag", () => {
  const supplied = Orders.build("email").parcels("receipt", gilsParcel);
  assert.deepEqual(supplied, { kind: "fromDataBag" });
});
// ANCHOR_END: adapter
