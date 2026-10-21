// What the checks of the book share: where its files are, which programs it holds, how to run a
// command, and what changes from one run of a program to the next.

import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";

/** The book's `typescript` directory, which the build mirrors in `build/`. */
export const book = resolve(import.meta.dirname, "..", "..");

/** The names of the files in one of the book's directories ending with `suffix`, without it, in order. */
export function names(directory: string, suffix: string): string[] {
  const path = join(book, directory);
  if (!existsSync(path)) return [];
  return readdirSync(path)
    .filter((file) => file.endsWith(suffix))
    .map((file) => file.slice(0, -suffix.length))
    .sort();
}

/** A file of the book, as text. */
export function read(path: string): string {
  return readFileSync(join(book, path), "utf8");
}

export interface Ran {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

/**
 * Runs Node with `args` in the book's directory. Node's test runner tells the processes it starts
 * that they run under it, so that variable is left out, and so are those that turn colours on or
 * off: a program's tests run as they would from a terminal, and print the same whatever runs them.
 */
export function node(...args: string[]): Ran {
  const {
    NODE_TEST_CONTEXT: _context,
    FORCE_COLOR: _force,
    NO_COLOR: _none,
    ...environment
  } = process.env;
  return spawnSync(process.execPath, args, {
    cwd: book,
    encoding: "utf8",
    env: environment,
  });
}

/** Runs the TypeScript compiler the book pins, in the book's directory. */
export function tsc(...args: string[]): Ran {
  return node(join(book, "node_modules", "typescript", "bin", "tsc"), ...args);
}

type Changing = readonly (readonly [RegExp, string])[];

/**
 * What changes from one run of a program to the next: an event's timestamp, an instant in ISO 8601
 * in UTC, and a generated journey ID, a random UUID. The book's own data holds neither.
 */
const CHANGING: Changing = [
  [/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z/g, "[TIMESTAMP]"],
  [/[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}/g, "[JOURNEY_ID]"],
];

/** What also changes from one run of a program's tests to the next: how long each took. */
const CHANGING_IN_A_REPORT: Changing = [
  ...CHANGING,
  [/\(\d+(?:\.\d+)?ms\)/g, "([DURATION])"],
  [/duration_ms \d+(?:\.\d+)?/g, "duration_ms [DURATION]"],
];

function replaced(printed: string, changing: Changing): string {
  return changing.reduce(
    (text, [pattern, placeholder]) => text.replace(pattern, placeholder),
    printed,
  );
}

/**
 * What a program printed, with what changes from one run to the next replaced by a placeholder.
 * The recorded output keeps the values of the run that recorded it, so the book shows a real run.
 */
export function withoutChanges(printed: string): string {
  return replaced(printed, CHANGING);
}

/** What a program's tests printed, with how long each took a placeholder too. */
export function reportWithoutChanges(printed: string): string {
  return replaced(printed, CHANGING_IN_A_REPORT);
}
