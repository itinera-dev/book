# The TypeScript book

The TypeScript book: its [chapters](chapters/README.md), and the examples and compile errors they show, laid out as [FORMAT.md](../FORMAT.md) describes. They depend on `itinera` as [package.json](package.json) names it: until itinera's first release, on both `main` and `next`, a commit of [itinera-ts](https://github.com/itinera-dev/itinera-ts), the one [itinera.json](itinera.json) names, as [BOOK.md](../BOOK.md#how-the-examples-are-checked) explains. Until that first release, this book is published only at `/book/typescript/next/`.

## Getting itinera

npm cannot install one package out of a repository that holds several, as itinera-ts does, so `npm run itinera` prepares it first:

1. It fetches the commit that [itinera.json](itinera.json) names into `.itinera/source`, installs its dependencies, packs the `itinera` package as npm would publish it, and unpacks it into `.itinera/package`.
2. [package.json](package.json) depends on `file:.itinera/package`, and [.npmrc](.npmrc) has npm install it as a package of its own, with its dependencies beside the book's.
3. It does nothing when `.itinera/package` already comes from that commit, so run it once, and again whenever [itinera.json](itinera.json) changes, followed by `npm ci`, since npm copies the package into `node_modules`. When the new commit changes itinera's own dependencies, run `npm install` instead, which updates [package-lock.json](package-lock.json).

With `ITINERA_TS` set to a checkout of itinera-ts whose dependencies are installed, `npm run itinera` packs that checkout as it stands instead, so the book can run against code not pushed yet: `ITINERA_TS=../../itinera-ts npm run itinera`, then `npm install`.

## Running the examples

From this directory, after `npm run itinera` and `npm ci`:

- `npm run build` compiles every example, tested program and check into `build/`, with `tsc`, since the decorators need the metadata it records;
- `node build/examples/NAME.js` runs one example, and what it prints must match `expected/NAME.txt`;
- `npm test` builds everything, then runs every example and checks that it prints its recorded output, as CI does. An event's timestamp and a generated journey ID change from one run to the next, so they are compared as placeholders, and the recorded output keeps the values of the run that recorded it.
- `npm test` also builds every example and every program in `tested/` as it would ship, with [tsconfig.ship.json](tsconfig.ship.json): without the tests, and with `itinera/testing` resolving to nothing, so a program that uses test support outside its tests fails. A tested program `NAME.ts` keeps its tests in `NAME.test.ts`, run with Node's own test runner, and `npm test` checks that they print the report recorded in `tested/NAME.txt`, where the time each test took is a placeholder.
- `npm test` also compiles every program in `compile-errors/` on its own, with the book's compiler options, and checks that the compiler refuses it with the message recorded next to it. Compiler messages change between TypeScript releases, so [package.json](package.json) pins the compiler exactly; moving to a newer one means recording them again.

The examples are ES modules, compiled with the options in [tsconfig.json](tsconfig.json): `strict`, `experimentalDecorators` and `emitDecoratorMetadata`, which itinera's decorators need.
