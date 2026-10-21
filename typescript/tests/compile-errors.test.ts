// Every program in `compile-errors/` does not compile, and the compiler says what the book shows:
// the message recorded next to it.
//
// Each program is compiled on its own, with the book's compiler options, by the compiler that
// package.json pins exactly, since its messages change between releases. The message is what the
// compiler prints without colours, one line per error. Paths in it are written as a reader of the
// book sees them: the program's from the book's `typescript` directory, and itinera's as a path
// inside itinera-ts, never as a path on the machine that compiled it.

import assert from "node:assert/strict";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import { book, names, read, tsc } from "./book.js";

/** The compiler's message, with every path written as the book shows it. */
function asShown(message: string): string {
  return message
    .replaceAll(`${book}/`, "")
    .replaceAll("node_modules/itinera/", "itinera-ts/packages/itinera/");
}

/** A project holding only the program `name`, with every option of the book's own. */
function projectOf(name: string): string {
  const directory = join(book, "build", "compile-errors");
  mkdirSync(directory, { recursive: true });
  const project = join(directory, `${name}.json`);
  const options = {
    extends: "../../tsconfig.json",
    compilerOptions: { noEmit: true },
    include: [],
    files: [`../../compile-errors/${name}.ts`],
  };
  writeFileSync(project, JSON.stringify(options));
  return project;
}

test("every compile error gives its recorded message", async (t) => {
  const programs = names("compile-errors", ".ts");
  const messages = names("compile-errors", ".txt");
  if (programs.length === 0 && messages.length === 0) {
    t.skip("the book has no compile errors yet");
    return;
  }
  assert.deepEqual(
    messages,
    programs,
    "every program in compile-errors/ has its message, and every message its program",
  );
  for (const name of programs) {
    await t.test(name, () => {
      const compiled = tsc("--project", projectOf(name), "--pretty", "false");
      assert.notEqual(compiled.status, 0, `${name} compiles`);
      assert.equal(
        asShown(compiled.stdout).trimEnd(),
        read(join("compile-errors", `${name}.txt`)).trimEnd(),
      );
    });
  }
});
