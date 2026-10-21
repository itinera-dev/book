# Mistakes caught before a journey

## What you want to do

The shop's workflow is now put together from many parts: steps and their names, policies attached to them, an input adapter, services and their providers, a rescue step and roles. You want to see how it is put together without running an order, and to know that a mistake in putting it together is caught before any order runs.

## The code

### How the workflow is put together

The shop is the one [chapter 36](36-orders-refused-before-they-start.md) runs; its workflow class is [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md)'s, with the role implementations of [chapter 26](26-running-async-journeys-while-others-wait.md) and [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s provider for `cards`. The program prints its **listing**, which `Orders.listing()` reads from the workflow's declaration, one entry for each step:

```example
name: ch37_listing
region: listing
```

`entry` writes each step's line from what the listing says about it, and `services`, with `served` for each one, writes the services of a step or of its rescue step. The entries are `ListedStep`s, and their services `ListedService`s, both types from `itinera/workflow`:

```example
name: ch37_listing
region: entry
```

It prints:

```output
name: ch37_listing
```

### Mistakes found as the class is defined

This small shop has steps and policies of its own:

```example
name: ch37_violations
region: shop
```

Its workflow is put together with two mistakes: both policies are attached to the charge, though both define `on step success`, and the shipping is listed twice. The decorators check the class as it is defined, so a mistake stops a program as it loads, before its first order. To print what they found, this program defines the class inside a function, `defineOrders`:

```example
name: ch37_violations
region: wrong
```

It calls the function, and catches what the decorators throw:

```example
name: ch37_violations
region: refused
```

It prints:

```output
name: ch37_violations
```

### Mistakes that don't compile

Each program below is a small shop with one mistake, and none of them compiles. A policy that tells the customer through the `Messages` role, on a workflow that can't tell anyone anything:

```compile-error
name: ch37_role_not_provided
```

An `on step failure` hook that tries to finish the journey as succeeded, which only `on step success` may do:

```compile-error
name: ch37_invalid_lifecycle
```

A step that contributes its connection to the bank, which isn't a value:

```compile-error
name: ch37_not_a_value
```

A rescue step that tries to skip the charge rather than repair it:

```compile-error
name: ch37_rescue_step_skips
```

A provider that returns the bank as it is, rather than through `this.provide(...)`:

```compile-error
name: ch37_provider_without_provide
```

An input adapter that asks for the data bag, as the ID generator of [chapter 11](11-naming-a-journey.md) does, and tries to write the weight into it:

```compile-error
name: ch37_adapter_writes_the_data_bag
```

A journey run without awaiting its result:

```compile-error
name: ch37_result_not_awaited
```

## How it works

1. **The listing says how the workflow is put together.** It lists each step in order, with its position, the policies attached to it, its input adapter, each service it asks for with the provider that serves it, and its rescue step with that step's own services. It is read from the declaration: nothing runs, no provider is called, and no workflow object is made, so `listing` is called on the class itself. The listing alone shows, for example, that the card registry serves `charge_rescue` and nothing else.
2. **A mistake in putting a workflow together is a violation.** Checking how a workflow is put together, before any journey, is called **admission**, and the specification names each mistake it catches: `duplicate step name`, `hook defined twice`, `service not provided`, `rescued step retried` and a few more, all listed in [3.6, Configuration errors](https://github.com/itinera-dev/spec/blob/main/spec/03-workflow-definitions.md#36-configuration-errors). **Every violation found is reported at once, not only the first.**
3. **With the decorators, a violation is thrown as the class is defined.** `@workflow` checks the whole declaration as the class statement runs, and throws a `Violations`, from `itinera/workflow`, an `Error` whose message has one line for each violation, as the specification names it. A class defined at the top of a module, as the shop's is, is defined as the module loads, so the program stops there: no order can run on a workflow put together wrongly. The program checks that what it caught is a `Violations`, and throws anything else on.
4. **Some mistakes can't be written at all.** TypeScript's types rule out the seven that don't compile. A workflow policy that asks for a [role](19-letting-policies-act-on-the-outside-world.md) compiles only on a workflow class that has the role's methods. Each hook's return type lists only the [lifecycles](17-giving-up-a-step-early.md) that hook may return, so `OnFailure` holds no `FinishWorkflow`. A contribution must be a value, as [chapter 6](06-passing-data-on.md) said, and an object with a method isn't one. A `RescueOutcome` is never a skip, as [chapter 14](14-repairing-then-trying-once-more.md) said, so a rescue step has no `skipBecause`. A provider returns what [`this.provide(...)`](07-giving-a-step-the-tools-it-needs.md) makes, never the bare service. A `DataBag` has methods that read and none that write, since an adapter reads and never writes, as [chapter 21](21-deciding-where-a-steps-data-comes-from.md) said. And [`run`](05-a-first-journey.md) returns a promise, which has no [`isSucceeded`](25-reading-the-result.md) until it is awaited.

The violations are caught because the decorators write the workflow's declaration for you, and check it as they write it. But you haven't seen what they write.

## Outcomes

We have shown how the listing says how a workflow is put together, and how a mistake in putting it together is caught before any order runs.

| The usual way | With Itinera |
|---|---|
| Which hooks run after a step, or which client serves a repair, is often learned only by reading every module. | The listing names each step's policies, its adapter, and the services of its rescue step. |
| A wiring mistake, such as two handlers for one event, often shows only when a request takes that path in production. | The program doesn't compile, or stops as it loads, and the message names every mistake. |
