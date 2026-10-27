# What this version supports

What the `itinera` this book covers implements, how you can check it, and what comes next. The specification defines what each version of Itinera means; the conformance report of `itinera` says which parts of it the package passes.

## The specification

This book follows specification 0.1, the first version of the [Itinera specification](https://github.com/itinera-dev/spec/tree/main/spec). It defines tier 1, a single workflow:

1. steps, their inputs, their outcomes and their abnormal terminations;
2. services, and the service providers that supply them to steps;
3. the data bag, contributions and values;
4. retry budgets, and whether an abnormal termination may be retried;
5. rescue steps;
6. step policies and workflow policies, their hooks and the lifecycles they return;
7. workflow roles and input adapters;
8. workflow instances, journey IDs and the result;
9. events, reporters, dispatchers and dispatcher factories, and global metadata;
10. admission, violations and refusals;
11. the local executor, and the execution modes.

Each part of the specification is written from accepted proposals, listed at the top of each of its chapters, and is checked by the [conformance cases](https://github.com/itinera-dev/conformance), a set of scenarios every implementation runs. Until a first implementation passes every case of tier 1, the cases are released as candidates.

## The `itinera` this book covers

This book covers `itinera` `0.1.0-beta.6`, the second release, a beta, published on [npm](https://www.npmjs.com/package/itinera) under the tag `next` and built from its [repository](https://github.com/itinera-dev/itinera-ts) at the tag `v0.1.0-beta.6`, and every example you read here compiles and runs against it. It passes every proposal of tier 1, with the capabilities test support and value conversions.

## Capabilities

A tier says what Itinera means; capabilities say what an executor accepts, and what else a language offers. `itinera` claims three:

1. **`async`:** its one executor, `LocalExecutor`, runs journeys asynchronously, so `run` always returns a promise. It accepts steps and hooks of either execution mode, and waits for whichever it gets, as [chapter 26](26-running-async-journeys-while-others-wait.md) shows, and reporters too, as [chapter 31](31-stopping-a-journey-when-its-log-fails.md) does. It doesn't claim `sync`, an executor that runs a journey without a promise, since JavaScript can't wait for a promise synchronously.
2. **`testing`:** test support, from `itinera/testing`, as chapters [27](27-testing-a-step-alone.md) to [30](30-testing-a-whole-journey.md) show.
3. **`value-conversions`:** an object of one of your own classes can be given wherever a value is, such as a contribution to the data bag. `itinera` calls the object's `toValue()` to turn it into a value, and the class's static `fromValue` to turn the value back into an object when it is read. A conversion that fails while a value is read makes it of the wrong type, and one that fails while a value is captured makes it not a value, as the specification says.

## The conformance report

The report of `itinera` is [`conformance.json`](https://github.com/itinera-dev/itinera-ts/blob/v0.1.0-beta.6/conformance.json), in its repository. It says which version of the conformance cases the package runs, the proposals whose cases all pass, the capabilities it claims, and the rules it makes impossible to write. In `0.1.0-beta.6`, it runs the conformance cases `v0.1.0-rc.18`, and lists all 40 proposals of tier 1 of specification 0.1 as passing, including those of test support and value conversions.

A rule the specification lets a language make impossible, such as a policy attached to a workflow that doesn't provide its role, isn't run as a case in TypeScript. Instead, a type test of the package proves that the mistake doesn't compile, as [chapter 37](37-mistakes-caught-before-a-journey.md) shows. The report lists these rules as impossible to write in TypeScript, each with the type tests that prove it:

1. a lifecycle a hook may not return (`invalid-lifecycle`);
2. a policy needing a role the workflow doesn't provide (`role-not-provided`);
3. a write through an input adapter's access to the data bag (`bag-write`);
4. a rescue step that skips (`rescue-skip`);
5. a service provider that returns no service (`no-instance`);
6. your code returning what its call can't return, such as a step's `run` that returns no outcome or a step hook that returns `undefined` (`unexpected-result`);
7. an ID generator that returns something that isn't a string (`non-string-journey-id`).

The specification lets a language make a few more rules impossible, which TypeScript's types can't rule out in every case: something stored that is not a value, for one, since a type can't refuse every value that isn't one. `itinera` checks those while the program runs, and their cases run like any other, except for an executor that refuses an execution mode, which doesn't arise, since `LocalExecutor` accepts both.

## What later tiers add

Each tier includes everything before it. Later tiers will add, once their proposals are accepted:

1. **Tier 2:** moving a journey to another workflow, with or without its data.
2. **Tier 3:** running a workflow from inside another.
3. **Durable runs:** journeys that survive a crash and resume, with their state kept in a store.

Each will add a part to this book, in the same style, once `itinera` implements it.
