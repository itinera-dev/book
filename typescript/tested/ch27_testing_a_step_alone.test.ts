// The tests of chapter 26's shop for chapter 27: each runs the charge, its
// rescue step or the shipping alone, with a bank or a carrier faked for it.

// ANCHOR: imports
import assert from "node:assert/strict";
import { test } from "node:test";
import { None, Some } from "itinera/option";
import { Contributions, StepEventReporter } from "itinera/testing";
import { string } from "itinera/value";
import {
  BookingFailed,
  type Carrier,
  Charge,
  NetworkDown,
  type Payments,
  RenewCard,
  Ship,
} from "./ch27_testing_a_step_alone.js";
// ANCHOR_END: imports

// ANCHOR: fakes
/**
 * A bank for tests: it answers every charge as the test chose, and has
 * replaced every card with one named after its holder that expires in 2029.
 */
class TestBank implements Payments {
  constructor(private readonly answer: "charges" | "declines" | "is down") {}

  async charge(order: string): Promise<string | undefined> {
    if (this.answer === "is down") throw new NetworkDown();
    if (this.answer === "declines") return undefined;
    return `R-${order}`;
  }

  async replacement(card: string): Promise<string | undefined> {
    return `${card.slice(0, card.lastIndexOf("-"))}-2029`;
  }
}

/**
 * A carrier for tests: it books and collects every parcel, or its booking
 * system is down.
 */
class TestCarrier implements Carrier {
  constructor(private readonly answer: "works" | "is down") {}

  async book(order: string): Promise<string> {
    if (this.answer === "is down") throw new BookingFailed();
    return `T-${order}`;
  }

  async collects(): Promise<boolean> {
    return true;
  }
}
// ANCHOR_END: fakes

// ANCHOR: charge
/** Ann's charge of 40 to her card. */
function annsCharge(): Charge {
  return new Charge("A-1", 40, "ann-2028", false);
}

test("a charge the bank takes gives back the bank's receipt", async () => {
  const contributions = new Contributions();
  const outcome = await annsCharge().run(
    new TestBank("charges"),
    contributions.contributor,
  );
  assert.equal(outcome.kind, "success");
  assert.deepEqual(
    contributions.read("receipt", string).option(),
    Some("R-A-1"),
  );
});

test("a charge the bank declines fails with card-expired and no receipt", async () => {
  const contributions = new Contributions();
  const outcome = await annsCharge().run(
    new TestBank("declines"),
    contributions.contributor,
  );
  assert.deepEqual(outcome, {
    kind: "failure",
    reason: { code: "card-expired" },
  });
  assert.equal(contributions.read("receipt", string).option(), None);
});

test("a charge lets the error of a bank that is down escape", async () => {
  const contributions = new Contributions();
  await assert.rejects(
    annsCharge().run(new TestBank("is down"), contributions.contributor),
    NetworkDown,
  );
});
// ANCHOR_END: charge

// ANCHOR: rescue
test("renewing an expired card gives back the card that replaced it", async () => {
  const contributions = new Contributions();
  const renew = new RenewCard("jo-2024", { code: "card-expired" });
  const outcome = await renew.run(
    new TestBank("charges"),
    contributions.contributor,
  );
  assert.equal(outcome.kind, "success");
  assert.deepEqual(
    contributions.read("card", string).option(),
    Some("jo-2029"),
  );
});

test("a failure that is not an expired card is not repairable", async () => {
  const contributions = new Contributions();
  const renew = new RenewCard("max-2027", { code: "nothing-to-charge" });
  const outcome = await renew.run(
    new TestBank("charges"),
    contributions.contributor,
  );
  assert.deepEqual(outcome, {
    kind: "failure",
    reason: { code: "not-repairable" },
  });
});
// ANCHOR_END: rescue

// ANCHOR: ship
/** An order's shipping, for a parcel of this weight. */
function shipping(weight: number): Ship {
  return new Ship("A-1", weight, "R-A-1", false);
}

test("a parcel of 2 kg leaves with its receipt", async () => {
  const contributions = new Contributions();
  const events = new StepEventReporter();
  const outcome = await shipping(2).run(
    new TestCarrier("works"),
    contributions.contributor,
    events.reporter,
  );
  assert.equal(outcome.kind, "success");
  assert.equal(
    events.emitted[0]?.message,
    "the parcel leaves with receipt R-A-1",
  );
});

test("a parcel of 90 kg fails with over-parcel-limit before the carrier is called", async () => {
  const contributions = new Contributions();
  const events = new StepEventReporter();
  const outcome = await shipping(90).run(
    new TestCarrier("is down"),
    contributions.contributor,
    events.reporter,
  );
  assert.deepEqual(outcome, {
    kind: "failure",
    reason: { code: "over-parcel-limit" },
  });
});
// ANCHOR_END: ship
