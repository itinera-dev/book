# What this version supports

What the Itinera for Java this book covers implements, how you can check it, and what comes next. The specification defines what each version of Itinera means; the conformance report of Itinera for Java says which parts of it the library passes.

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

Each part of the specification is written from accepted proposals, listed at the top of each of its chapters, and is checked by the [conformance cases](https://github.com/itinera-dev/conformance), a set of scenarios every implementation runs. The cases of tier 1 were released as candidates until an implementation first passed every one of them, and are now tagged `v0.1.0`, like the specification.

## The Itinera for Java this book covers

This book covers Itinera for Java `0.1.0`, the second release and the first that is not a pre-release. Itinera for Java is the Java layer, `itinera-java`, over the same core as Itinera for Kotlin, published with it on [Maven Central](https://central.sonatype.com/namespace/org.itinera-dev) and built from their repository, [itinera-j](https://github.com/itinera-dev/itinera-j), at the tag `v0.1.0`, and every example you read here compiles and runs against it. It implements all of tier 1, and test support.

## Capabilities

A tier says what Itinera means; capabilities say what an executor accepts, and what else a language offers. Itinera for Java claims two:

1. **`sync`:** `LocalExecutor` runs a synchronous workflow's journey, and its `run` returns the result, as every chapter shows. Java writes synchronous workflows only, so it doesn't claim `async`; journeys that run at the same time each run on a thread of their own, as [chapter 26](26-running-async-journeys-while-others-wait.md) shows.
2. **`testing`:** test support, from the artifact `itinera-testing`, as chapters [27](27-testing-a-step-alone.md) to [30](30-testing-a-whole-journey.md) show.

It doesn't claim `value-conversions`, which a language claims when it calls your own code to turn your objects into values, or values back into your types. Itinera for Java keeps your own objects as values: it checks and copies each one, and reads it back by its type, so the only code of yours it calls is what copies an object, such as a record's constructor, and the specification says that copying is not a conversion.

## The conformance report

The report of Itinera for Java is [`conformance-java.json`](https://github.com/itinera-dev/itinera-j/blob/v0.1.0/conformance-java.json), in its repository. It says which version of the conformance cases the library runs, the proposals whose cases all pass, the capabilities it claims, and the rules it makes impossible to write. In `0.1.0`, it runs the conformance cases `v0.1.0`, and lists 39 proposals as passing: every proposal of tier 1 of specification 0.1, including test support, except 0113, on value conversions, whose every case needs the capability `value-conversions`.

A rule the specification lets a language make impossible, such as a hook that returns a lifecycle its kind may not, isn't run as a case in Java. Instead, a test of the library proves that the mistake doesn't compile, and where an annotated class can still write it, that itinera's checker reports it as the program is built, as [chapter 37](37-mistakes-caught-before-a-journey.md) shows, or, without the checker, that the workflow is refused before any instance. The report lists these rules as impossible to write in Java, each with the tests that prove it:

1. a lifecycle a hook may not return (`invalid-lifecycle`);
2. a write through an input adapter's access to the data bag (`bag-write`);
3. a rescue step that skips (`rescue-skip`);
4. a service provider of journey scope requesting the name of the step being built or the attempt number (`journey-scope-request`);
5. an asynchronous step, input adapter or service provider in a synchronous workflow (`mode-not-accepted`), since Java has no way to write one;
6. a test's script naming a fraction of an attempt (`attempt-fraction`).

The specification lets a language make a few more rules impossible, which Java's types can't rule out in every case: your code returning what its call can't return, for one, and an ID generator returning something that isn't a string, since any method that returns an object can return `null`, which Kotlin's types rule out; something stored that is not a value, since a value's type is `Object`; and a policy needing a role the workflow doesn't provide, since an annotation can't tie its policy classes to the workflow's type. Itinera for Java checks those when the workflow's declaration is read, when an instance is created, or while the journey runs, and their cases run like any other. The checker reports some of them before that, as the program is built, as chapter 37 shows.

## What later tiers add

Each tier includes everything before it. Later tiers will add, once their proposals are accepted:

1. **Tier 2:** moving a journey to another workflow, with or without its data.
2. **Tier 3:** running a workflow from inside another.
3. **Durable runs:** journeys that survive a crash and resume, with their state kept in a store.

Each will add a part to this book, in the same style, once Itinera for Java implements it.
