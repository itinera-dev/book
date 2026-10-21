// Prepares the itinera package the book's examples depend on, in .itinera/package, where
// package.json's dependency `file:.itinera/package` finds it.
//
// Until itinera's first release, the book follows a commit of itinera-ts, the one itinera.json
// names, as `next` always does. npm cannot install one package of a repository that holds several, so this fetches that
// commit into .itinera/source, installs its dependencies, packs the package as npm would publish
// it, and unpacks it. With ITINERA_TS set to a checkout of itinera-ts, it packs that checkout as it
// stands instead, which is how itinera-ts runs the book against its own code; that checkout needs
// its dependencies installed, since packing builds the package.
//
// Run it before `npm ci`, and again, followed by `npm ci`, whenever itinera.json changes; it does
// nothing when the package is already the pinned commit's.

import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const book = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const prepared = join(book, ".itinera");
const unpacked = join(prepared, "package");
const stamp = join(prepared, "commit");
const { repository, commit } = JSON.parse(
  readFileSync(join(book, "itinera.json"), "utf8"),
);

/**
 * Starts a command. npm is run through the Node running this script when npm itself started it,
 * and through a shell only on Windows otherwise, where npm is a script; nothing else needs a shell,
 * whose quoting would break paths with spaces.
 */
function spawn(command, args, options) {
  const npm = process.env.npm_execpath;
  if (command === "npm" && npm) {
    return spawnSync(process.execPath, [npm, ...args], options);
  }
  return spawnSync(command, args, {
    ...options,
    shell: command === "npm" && process.platform === "win32",
  });
}

/** Runs a command, printing what it prints, and stops everything when it fails. */
function run(command, args, cwd) {
  const result = spawn(command, args, {
    cwd,
    stdio: ["ignore", "inherit", "inherit"],
  });
  if (result.status !== 0)
    throw new Error(`${command} ${args.join(" ")} failed in ${cwd}.`);
}

/** Runs a command and returns what it printed. */
function output(command, args, cwd) {
  const result = spawn(command, args, { cwd, encoding: "utf8" });
  if (result.status !== 0)
    throw new Error(
      `${command} ${args.join(" ")} failed in ${cwd}:\n${result.stderr}`,
    );
  return result.stdout;
}

/** A checkout of itinera-ts at the pinned commit, with its dependencies installed. */
function fetched() {
  const source = join(prepared, "source");
  if (!existsSync(join(source, ".git"))) {
    mkdirSync(source, { recursive: true });
    run("git", ["init", "--quiet"], source);
  }
  run("git", ["fetch", "--quiet", "--depth", "1", repository, commit], source);
  run("git", ["checkout", "--quiet", "--force", "--detach", commit], source);
  run("npm", ["ci", "--no-audit", "--no-fund"], source);
  return source;
}

const local = process.env.ITINERA_TS;
if (
  !local &&
  existsSync(stamp) &&
  readFileSync(stamp, "utf8").trim() === commit &&
  existsSync(unpacked)
) {
  console.log(`itinera is already at ${commit}.`);
  process.exit(0);
}

const source = local ? resolve(local) : fetched();
mkdirSync(prepared, { recursive: true });
const destination = mkdtempSync(join(tmpdir(), "itinera-pack-"));
try {
  const [packed] = JSON.parse(
    output(
      "npm",
      [
        "pack",
        "--json",
        "--workspace",
        "packages/itinera",
        "--pack-destination",
        destination,
      ],
      source,
    ),
  );
  rmSync(unpacked, { recursive: true, force: true });
  rmSync(stamp, { force: true });
  // npm's tarballs hold the package under a directory named `package`.
  run(
    "tar",
    ["-xzf", join(destination, packed.filename), "-C", prepared],
    book,
  );
  if (!local) writeFileSync(stamp, `${commit}\n`);
  console.log(
    `itinera ${packed.version} from ${local ? source : `${repository} at ${commit}`} is in .itinera/package.`,
  );
} finally {
  rmSync(destination, { recursive: true, force: true });
}
