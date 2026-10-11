# What this version supports

What the release this book covers implements, how you can check it, and what comes next. The specification defines what each version means; your language's conformance report says which parts of it a release passes.

## The specification

This edition follows specification 0.1, the first version of the [Itinera specification](https://github.com/itinera-dev/spec/tree/main/spec). It defines tier 1: a single workflow, with everything the book teaches in parts 2 to 6:

1. steps, their inputs, their outcomes and their abnormal terminations;
2. the data bag, contributions and values;
3. retry budgets, and whether an abnormal termination may be retried;
4. step policies and workflow policies, their hooks and the lifecycles they return;
5. workflow roles and input adapters;
6. workflow instances, journey IDs and the result;
7. events, reporters, dispatchers and dispatcher factories;
8. admission, violations and refusals;
9. the local executor, and the two execution modes, synchronous and asynchronous.

Each part of the specification is written from accepted proposals, listed at the top of each of its chapters, and is checked by the [conformance cases](https://github.com/itinera-dev/conformance), a set of scenarios every implementation runs. Until a first implementation passes every case of tier 1, the cases are released as candidates.

## Services and rescue steps

Two more features are being added to tier 1 before its first release: [services](https://github.com/itinera-dev/spec/issues/95), which [chapter 12](12-giving-a-step-the-tools-it-needs.md) teaches, and [rescue steps](https://github.com/itinera-dev/spec/issues/3), which [chapter 14](14-repairing-then-trying-once-more.md) teaches. Their designs are agreed; they become part of the specification once accepted, and arrive in each language before its first release candidate. Until then, the examples that use them are written against the planned form and are not run, and the book says so wherever it shows one.

## Capabilities

A tier says what Itinera means; capabilities say what an executor accepts. In tier 1 the capabilities are the two execution modes, synchronous and asynchronous. A language claims the modes its executors accept, and its conformance runner runs the cases of those modes.

## The conformance report

Each language keeps a report of what its release passes: the version of the conformance cases it runs, the proposals whose cases all pass, the capabilities it claims, and the rules it makes impossible to write. A rule the specification lets a language make impossible, such as a policy attached to a workflow that does not provide its role, is not run as a case in that language. Instead, its own tests prove that the mistake does not compile, or cannot be expressed at all, as [chapter 27](27-mistakes-caught-before-a-journey.md) shows.

```note
name: appendix3_what_this_version_supports
```

## What later tiers add

Each tier includes everything before it. Later tiers will add, once their proposals are accepted:

1. **Tier 2:** moving a journey to another workflow, with or without its data.
2. **Tier 3:** running a workflow from inside another.
3. **Durable runs:** journeys that survive a crash and resume, with their state kept in a store.

Each will add a part to this book, in the same style, once your language implements it.
