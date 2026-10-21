// The tests of chapter 26's shop for chapter 29: each runs one of Ann's orders
// with a script saying how the steps' attempts end.

import assert from "node:assert/strict";
import { mock, test } from "node:test";
import type { WorkflowInstance } from "itinera/instance";
import type { JourneyResult } from "itinera/journey";
import { Some } from "itinera/option";
import { type ScriptedRun, TestMistake, scripted } from "itinera/testing";
import { integer, string } from "itinera/value";
import { Orders } from "./ch29_testing_the_flow_with_scripted_steps.js";

// ANCHOR: quiet
mock.method(console, "log", () => undefined);
// ANCHOR_END: quiet

// ANCHOR: order
/**
 * An instance of the shop's workflow for one of Ann's orders, of 30, with
 * this card and a parcel of this weight.
 */
function order(
  order: string,
  card: string,
  weight: number,
): WorkflowInstance<Orders> {
  return Orders.build("email").instance({
    order,
    customer: "Ann",
    price: 30,
    parcel: { weight, size: [20, 15, 5] },
    card,
    giftCard: false,
  });
}
// ANCHOR_END: order

// ANCHOR: rescued
test("a declined card is renewed and the charge tried again with it", async () => {
  const run = await scripted(order("A-1", "ann-2024", 1))
    .step("charge")
    .willFail("card-expired")
    .onAttempt(2)
    .willContribute("receipt", "R-A-1")
    .willSucceed()
    .andStep("charge_rescue")
    .willContribute("card", "ann-2029")
    .willSucceed()
    .andStep("ship")
    .willContribute("tracking", "T-A-1")
    .willSucceed()
    .run();
  assert.ok(run.result.isSucceeded());
  const retried = run
    .step("charge")
    .attempt(2)
    .expect("the charge's attempt 2");
  assert.deepEqual(retried.input("card", string).option(), Some("ann-2029"));
});
// ANCHOR_END: rescued

// ANCHOR: given_up
/** The reason a policy failed the journey with, if one did. */
function failedByAPolicy(result: JourneyResult): string | undefined {
  if (!result.isFailed()) return undefined;
  const { failure } = result.status;
  if (failure.cause !== "FailWorkflow") return undefined;
  return failure.reason.code;
}

/**
 * Runs an order of 60 kg whose shipping fails on attempt 1 with this reason,
 * and would succeed on attempt 2.
 */
function shippingThatFailsWith(reason: string): Promise<ScriptedRun> {
  return scripted(order("A-2", "ann-2028", 60))
    .step("charge")
    .willContribute("receipt", "R-A-2")
    .willSucceed()
    .andStep("ship")
    .willFail(reason)
    .onAttempt(2)
    .willContribute("tracking", "B-A-2")
    .willSucceed()
    .run();
}

test("a shipping that fails with too-heavy is tried again", async () => {
  const run = await shippingThatFailsWith("too-heavy");
  assert.equal(run.step("ship").attempts.length, 2);
  assert.ok(run.result.isSucceeded());
});

test("a shipping that fails with over-parcel-limit is given up after attempt 1", async () => {
  const run = await shippingThatFailsWith("over-parcel-limit");
  assert.equal(run.step("ship").attempts.length, 1);
  assert.equal(failedByAPolicy(run.result), "no-carrier-can-take-it");
});

test("a scripted shipping is still built with the weight the adapter supplies", async () => {
  const run = await shippingThatFailsWith("too-heavy");
  const first = run.step("ship").attempt(1).expect("the shipping's attempt 1");
  assert.deepEqual(first.input("weight", integer).option(), Some(60));
});
// ANCHOR_END: given_up

// ANCHOR: outage
test("a charge that ends in an error is given up and never rescued", async () => {
  const run = await scripted(order("A-3", "ann-2028", 1))
    .step("charge")
    .willTerminateAbnormally("the payment network is down")
    .onAttempt(2)
    .willContribute("receipt", "R-A-3")
    .willSucceed()
    .andStep("charge_rescue")
    .willSucceed()
    .run();
  assert.equal(run.step("charge_rescue").attempts.length, 0);
  assert.equal(failedByAPolicy(run.result), "payment-network-down");
});
// ANCHOR_END: outage

// ANCHOR: mistake
test("a script that leaves out the shipping's attempt 2 is refused", async () => {
  const script = scripted(order("A-4", "ann-2028", 1))
    .step("ship")
    .willFail("too-heavy")
    .run();
  await assert.rejects(
    script,
    (mistake) =>
      mistake instanceof TestMistake &&
      mistake.message === "the test is mistaken: attempt 2 of ship is left out",
  );
});
// ANCHOR_END: mistake
