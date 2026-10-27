// Every example and every program in `tested/` builds as it would ship, without test support, and
// the tests of each program in `tested/` print the report the book shows: the one recorded next to
// it.
//
// A tested program `NAME.ts` keeps its tests beside it, in `NAME.test.ts`, as TypeScript projects
// do, and only they may import `itinera/testing`. tsconfig.ship.json builds the examples and the
// tested programs without their tests and with `itinera/testing` resolving to nothing, so a program
// that uses test support outside its tests does not compile. The tests run with Node's own test runner, one file at a
// time, in the order they are written, and how long each took is the one placeholder of the report.

import assert from "node:assert/strict";
import { join } from "node:path";
import { test } from "node:test";

import { names, node, read, reportWithoutChanges, tsc } from "./book.js";

/** The names of the tested programs: every file of `tested/` that is neither a test nor a report. */
function programs(): string[] {
  return names("tested", ".ts").filter((name) => !name.endsWith(".test"));
}

test("every example and tested program builds without test support", (t) => {
  if (names("examples", ".ts").length === 0 && programs().length === 0) {
    t.skip("the book has no examples or tested programs yet");
    return;
  }
  const built = tsc("--project", "tsconfig.ship.json", "--pretty", "false");
  assert.equal(
    built.status,
    0,
    `a program uses test support outside its tests:\n${built.stdout}`,
  );
});

test("the tests of every tested program print their recorded report", async (t) => {
  const tested = programs();
  if (tested.length === 0) {
    t.skip("the book has no tested programs yet");
    return;
  }
  assert.deepEqual(
    names("tested", ".test.ts"),
    tested,
    "every tested program has its tests in NAME.test.ts, and every test file its program",
  );
  assert.deepEqual(
    names("tested", ".txt"),
    tested,
    "every tested program has its report in NAME.txt, and every report its program",
  );
  for (const name of tested) {
    await t.test(name, () => {
      const ran = node(
        "--test",
        "--test-reporter=spec",
        join("build", "tested", `${name}.test.js`),
      );
      assert.equal(
        ran.status,
        0,
        `the tests of ${name} failed:\n${ran.stdout}${ran.stderr}`,
      );
      assert.equal(
        reportWithoutChanges(ran.stdout),
        reportWithoutChanges(read(join("tested", `${name}.txt`))),
      );
    });
  }
});
