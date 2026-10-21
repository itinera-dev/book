// Every example prints what the book shows: the output recorded for it in `expected/`.
//
// `npm test` builds the examples first, so they are checked as they are now. What changes from one
// run to the next is a placeholder on both sides before they are compared (see `withoutChanges`);
// everything else is compared exactly.

import assert from "node:assert/strict";
import { join } from "node:path";
import { test } from "node:test";

import { names, node, read, withoutChanges } from "./book.js";

test("every example prints its recorded output", async (t) => {
  const programs = names("examples", ".ts");
  const recorded = names("expected", ".txt");
  if (programs.length === 0 && recorded.length === 0) {
    t.skip("the book has no examples yet");
    return;
  }
  assert.deepEqual(
    recorded,
    programs,
    "every example has its output in expected/, and every output in expected/ its example",
  );
  for (const name of programs) {
    await t.test(name, () => {
      const ran = node(join("build", "examples", `${name}.js`));
      assert.equal(ran.status, 0, `${name} failed:\n${ran.stderr}`);
      assert.equal(
        withoutChanges(ran.stdout),
        withoutChanges(read(join("expected", `${name}.txt`))),
      );
    });
  }
});
