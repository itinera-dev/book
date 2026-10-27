# What this version supports

What the version of `itinera` this book covers implements, how you can check it, and what comes next. The specification defines what each version of Itinera means; the conformance report of `itinera` says which parts of it the crate passes.

## The specification

This book follows specification 0.1, the first version of the [Itinera specification](https://github.com/itinera-dev/spec/tree/main/spec). It defines tier 1: a single workflow, with everything the book teaches in parts 2 to 5:

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
11. the local executor, and the two execution modes, synchronous and asynchronous.

Each part of the specification is written from accepted proposals, listed at the top of each of its chapters, and is checked by the [conformance cases](https://github.com/itinera-dev/conformance), a set of scenarios every implementation runs. Until a first implementation passes every case of tier 1, the cases are released as candidates.

## The version of `itinera` this book covers

This book covers `itinera` `0.1.0-beta.4`, a beta, published on [crates.io](https://crates.io/crates/itinera) and built from its [repository](https://github.com/itinera-dev/itinera-rs) at the tag `v0.1.0-beta.4`, and every example you read here compiles and runs against it.

## Services, rescue steps and global metadata

[Services](07-giving-a-step-the-tools-it-needs.md), [rescue steps](14-repairing-then-trying-once-more.md) and global metadata are in `itinera`, together with the input adapters and service providers a rescue step may have of its own. Every program this book shows is compiled and run against that release.

## Capabilities

A tier says what Itinera means; capabilities say what an executor accepts. In tier 1 the capabilities are the two execution modes, synchronous and asynchronous, test support (`testing`) and value conversions (`value-conversions`). `itinera` claims both modes: `LocalExecutor` runs synchronous workflows and, with the `async` feature, `AsyncLocalExecutor` runs asynchronous ones. Neither runs a workflow of the other mode: that's a compile error, not a refusal. With the `testing` feature, it claims test support, which [chapters 27 to 30](27-testing-a-step-alone.md) use. It doesn't claim `value-conversions`, which a language claims when it calls your own code to turn your objects into values, or values back into your types: Rust keeps each value as it is and reads it back by its exact type, so it calls no conversion.

## The conformance report

The report of `itinera` is [`conformance.json`](https://github.com/itinera-dev/itinera-rs/blob/v0.1.0-beta.4/conformance.json), in its repository. It says which version of the conformance cases the crate runs, the proposals whose cases all pass, the capabilities it claims, and the rules it makes impossible to write. In `0.1.0-beta.4`, it runs the conformance cases `v0.1.0-rc.16`, and lists every tier 1 proposal of specification 0.1 as passing except 0113, on value conversions, whose every case needs the capability `value-conversions`.

A rule the specification lets a language make impossible, such as a policy attached to a workflow that doesn't provide its role, isn't run as a case in Rust. Instead, a test of the crate proves that the mistake doesn't compile, as [chapter 37](37-mistakes-caught-before-a-journey.md) shows. The report lists these rules as impossible to write in Rust, each with the test that proves it:

1. a lifecycle a hook may not return (`invalid-lifecycle`);
2. a policy needing a role the workflow doesn't provide (`role-not-provided`);
3. something that is not a value in the data bag, initial data, an input adapter's answer, a reason's details, an event or global metadata (`non-value`);
4. an asynchronous step or service provider on the synchronous executor (`mode-not-accepted`);
5. a handle used after its attempt or call (`late-handle`);
6. a write through an input adapter's access to the data bag (`bag-write`);
7. a rescue step that skips (`rescue-skip`);
8. a service provider of journey scope requesting the name of the step being built or the attempt number (`journey-scope-request`);
9. a service provider that returns no service (`no-instance`);
10. a test's script naming a negative attempt (`attempt-negative`);
11. a test's script naming a fraction of an attempt (`attempt-fraction`).

## What later tiers add

Each tier includes everything before it. Later tiers will add, once their proposals are accepted:

1. **Tier 2:** moving a journey to another workflow, with or without its data.
2. **Tier 3:** running a workflow from inside another.
3. **Durable runs:** journeys that survive a crash and resume, with their state kept in a store.

Each will add a part to this book, in the same style, once `itinera` implements it.
