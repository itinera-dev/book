# What this version supports

What the release this book covers implements, how you can check it, and what comes next. The specification defines what each version means; your language's conformance report says which parts of it a release passes.

## The specification

This edition follows specification 0.1, the first version of the [Itinera specification](https://github.com/itinera-dev/spec/tree/main/spec). It defines tier 1: a single workflow, with everything the book teaches in parts 2 to 6 except services:

1. steps, their inputs, their outcomes and their abnormal terminations;
2. the data bag, contributions and values;
3. retry budgets, and whether an abnormal termination may be retried;
4. rescue steps;
5. step policies and workflow policies, their hooks and the lifecycles they return;
6. workflow roles and input adapters;
7. workflow instances, journey IDs and the result;
8. events, reporters, dispatchers and dispatcher factories;
9. admission, violations and refusals;
10. the local executor, and the two execution modes, synchronous and asynchronous.

Each part of the specification is written from accepted proposals, listed at the top of each of its chapters, and is checked by the [conformance cases](https://github.com/itinera-dev/conformance), a set of scenarios every implementation runs. Until a first implementation passes every case of tier 1, the cases are released as candidates.

## Services and rescue steps

[Rescue steps](https://github.com/itinera-dev/spec/blob/main/spec/07-rescue-steps.md), which [chapter 14](14-repairing-then-trying-once-more.md) teaches, are in the specification now. [Services](https://github.com/itinera-dev/spec/issues/95), which [chapter 12](12-giving-a-step-the-tools-it-needs.md) teaches, are being added to tier 1 before its first release: their design is agreed, and becomes part of the specification once accepted. Both arrive in each language before its first release candidate. Until a language has them, the examples that use them are written against the planned form and are not run, and the book says so wherever it shows one.

## Capabilities

A tier says what Itinera means; capabilities say what an executor accepts. In tier 1 the capabilities are the two execution modes, synchronous and asynchronous. A language claims the modes its executors accept, and its conformance runner runs the cases of those modes.

## The conformance report

Each language keeps a report of what its release passes: the version of the conformance cases it runs, the proposals whose cases all pass, the capabilities it claims, and the rules it makes impossible to write. A rule the specification lets a language make impossible, such as a policy attached to a workflow that does not provide its role, is not run as a case in that language. Instead, its own tests prove that the mistake does not compile, or cannot be expressed at all, as [chapter 27](27-mistakes-caught-before-a-journey.md) shows.

The Rust edition follows the crate `itinera` at the commit this book's `Cargo.toml` pins, on its way to the release `0.1.0-rc.1`. Rescue steps are in the specification now, and arrive in the crate together with services, before `0.1.0-rc.1`: the crate cannot be released until every tier 1 proposal is in its report, those two included.

Its report is [`conformance.json`](https://github.com/itinera-dev/itinera-rs/blob/main/conformance.json), in the crate's repository. At this commit it runs the conformance cases `v0.1.0-rc.8`, lists every proposal of specification 0.1 as passing except rescue steps (proposal 3), which it does not implement yet, and claims both capabilities: `LocalExecutor` runs synchronous workflows and, with the `async` feature, `AsyncLocalExecutor` runs asynchronous ones. Neither runs a workflow of the other mode: that is a compile error, not a refusal.

It lists these rules as impossible to write in Rust, each with the test that proves it: a lifecycle a hook may not return (`invalid-lifecycle`), a policy needing a role the workflow does not provide (`role-not-provided`), something that is not a value in the data bag, initial data, an input adapter's answer, a reason's details or an event (`non-value`), an asynchronous step on the synchronous executor (`mode-not-accepted`), a handle used after its attempt or call (`late-handle`), and a write through an input adapter's access to the data bag (`bag-write`).

## What later tiers add

Each tier includes everything before it. Later tiers will add, once their proposals are accepted:

1. **Tier 2:** moving a journey to another workflow, with or without its data.
2. **Tier 3:** running a workflow from inside another.
3. **Durable runs:** journeys that survive a crash and resume, with their state kept in a store.

Each will add a part to this book, in the same style, once your language implements it.
