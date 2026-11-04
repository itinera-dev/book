# Mistakes caught before a journey

## What you want to do

The shop's workflow is now put together from many parts: steps and their names, policies attached to them, an input adapter, services and their providers, a rescue step and roles. You want to see how it is put together without running an order, and to know that a mistake in putting it together is caught before any order runs.

## The code

### How the workflow is put together

The shop is the one [chapter 36](36-orders-refused-before-they-start.md) runs: `Orders` is the workflow [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) declares, with [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s provider for `cards`. `main` prints its **listing**, one entry for each step. `WorkflowDescriptor.of(Orders::class.java)`, from `org.itineradev.itinera.workflow`, reads the workflow's declaration from its annotations, and `listing()` lists it:

```example
name: ch37_listing
region: listing
```

`entry` writes each step's line from what the listing says about it, and `services`, with `service` for each one, writes the services of a step or of its rescue step. The entries are `ListedStep`s, and their services `ListedService`s, both from the same package:

```example
name: ch37_listing
region: entry
```

It prints:

```output
name: ch37_listing
```

### Mistakes that don't compile

Every program in this book is built with itinera's checker, `itinera-checker`, which checks each annotated workflow while the program is built. An application adds it through KSP: in Gradle's Kotlin DSL, the plugin `id("com.google.devtools.ksp") version "2.3.12"`, the release the checker is built against, then `ksp(platform("org.itinera-dev:itinera-bom:VERSION"))` and `ksp("org.itinera-dev:itinera-checker")` under `dependencies`.

Each program below is a small shop with one mistake, and none of them compiles. The shipping listed twice:

```compile-error
name: ch37_duplicate_step_name
```

Two policies of the charge that both define `on step success`:

```compile-error
name: ch37_hook_defined_twice
```

A policy that tells the customer through the `Messages` [role](19-letting-policies-act-on-the-outside-world.md), on a workflow that doesn't implement it:

```compile-error
name: ch37_role_not_provided
```

An `on step failure` hook that tries to finish the journey as succeeded, which only `on step success` may do:

```compile-error
name: ch37_invalid_lifecycle
```

The same hook, declared to return an `OnSuccess` so that `finishWorkflow()` fits:

```compile-error
name: ch37_hook_of_the_wrong_kind
```

A rescue step that tries to skip the charge rather than repair it:

```compile-error
name: ch37_rescue_step_skips
```

An input adapter that asks for its read access to the data bag, a `DataBagAccess` from `org.itineradev.itinera.journey`, and tries to write the weight into it:

```compile-error
name: ch37_adapter_writes_the_data_bag
```

An asynchronous workflow's instance given to a `LocalExecutor`:

```compile-error
name: ch37_asynchronous_instance_run_locally
```

## How it works

1. **The listing says how the workflow is put together.** It lists each step in order, with its position, the policies attached to it, its input adapter, each service it asks for with the provider that serves it, and its rescue step with that step's own services. It is read from the declaration: nothing runs, and no provider is called. The listing alone shows, for example, that the card registry serves `charge_rescue` and nothing else.
2. **A mistake in putting a workflow together is a violation.** Checking how a workflow is put together, before any journey, is called **admission**, and the specification names each mistake it catches: `duplicate step name`, `hook defined twice`, `service not provided`, `rescued step retried` and a few more, all listed in [3.6, Configuration errors](https://github.com/itinera-dev/spec/blob/main/spec/03-workflow-definitions.md#36-configuration-errors). Every violation found is reported at once, not only the first.
3. **With the checker, a violation is a compile error.** The checker reads the annotations as KSP runs, and reports each violation as an error marked `[ksp]`, on the line of the class or method it concerns, so a workflow put together wrongly never runs.
4. **Some mistakes can't be written at all.** Each hook's return type lists only the [lifecycles](17-giving-up-a-step-early.md) its kind may return, and `finishWorkflow()` is an `OnSuccess`, as [chapter 22](22-finishing-a-journey-early.md) said, never an `OnFailure`. A hook declared with another kind's type still compiles as Kotlin, so the checker reports it instead. A [`RescueOutcome`](14-repairing-then-trying-once-more.md) is a success or a failure, never a skip, so a rescue step has no `skipBecause`. A `DataBagAccess` has methods that read and none that write, since an adapter reads and never writes, as [chapter 21](21-deciding-where-a-steps-data-comes-from.md) said. And an instance's type says its mode: `LocalExecutor` runs only a [`WorkflowInstance<Sync>`](29-testing-the-flow-with-scripted-steps.md), while an asynchronous workflow's instance goes to an `AsyncLocalExecutor`, as in [chapter 26](26-running-async-journeys-while-others-wait.md), whose `run` is a `suspend` function, so its result is always awaited.

Each of these is caught because the annotations declare the workflow for you, and the checker reads that declaration as the program is built. But you haven't seen what the annotations stand for.

## Outcomes

We have shown how the listing says how a workflow is put together, and how a mistake in putting it together is caught before any order runs.

| The usual way | With Itinera |
|---|---|
| Which hooks run after a step, or which client serves a repair, is often learned only by reading every module. | The listing names each step's policies, its adapter, and the services of its rescue step. |
| A wiring mistake, such as two handlers for one event, often shows only when a request takes that path in production. | The program doesn't build, and the message names the mistake. |
