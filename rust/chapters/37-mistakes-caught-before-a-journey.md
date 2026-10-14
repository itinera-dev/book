# Mistakes caught before a journey

## What you want to do

The shop's workflow is now put together from many parts: steps and their names, policies attached to them, an input adapter, services and their providers, a rescue step and roles. You want to see how it is put together without running an order, and to know that a mistake in putting it together is caught before any order runs.

## The code

### How the workflow is put together

The shop is the one [chapter 36](36-orders-refused-before-they-start.md) runs; its workflow is the one [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) declares, with [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s provider for `cards`. `main` prints its **listing**, read from the workflow's declaration, one line for each step, with `Workflow` from `itinera::workflow` in scope:

```example
name: ch37_listing
region: listing
```

`entry` writes each step's line from what the listing says about it, and `services`, with `service` for each one, writes the services of a step or of its rescue step:

```example
name: ch37_listing
region: entry
```

It prints:

```output
name: ch37_listing
```

### Mistakes that don't compile

Each program below is a small shop with one mistake, and none of them compiles. The shipping listed twice:

```compile-error
name: ch37_duplicate_step_name
```

Two policies of the charge that both define `on step success`:

```compile-error
name: ch37_hook_defined_twice
```

A policy that tells the customer through the `Messages` role, on a workflow that doesn't provide it:

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

An asynchronous workflow whose journey is run without being awaited:

```compile-error
name: ch37_result_not_awaited
```

## How it works

1. **The listing says how the workflow is put together.** It lists each step in order, with its position, the policies attached to it, its input adapter, each service it asks for with the provider that serves it, and its rescue step with that step's own services. It is read from the declaration: nothing runs, and no provider is called. The listing alone shows, for example, that the card registry serves `charge_rescue` and nothing else.
2. **A mistake in putting a workflow together is a violation.** Checking how a workflow is put together, before any journey, is called **admission**, and the specification names each mistake it catches: `duplicate step name`, `hook defined twice`, `service not provided`, `rescued step retried` and a few more, all listed in [3.6, Configuration errors](https://github.com/itinera-dev/spec/blob/main/spec/03-workflow-definitions.md#36-configuration-errors). Every violation found is reported at once, not only the first.
3. **With the attributes, a violation is a compile error.** `#[workflow]` checks the declaration while the program compiles, so the first two programs never run, and their messages name the violation. Some messages say "evaluation panicked": the check runs in a constant the compiler evaluates, so it is still a compile error, and nothing panics when the program runs.
4. **Some mistakes can't be written at all.** Rust's types rule out the last four. A policy that asks for a role needs a workflow implementing `Provides<dyn Messages>`, which `#[role]` writes. Each hook's return type lists only the lifecycles that hook may return, so `on step failure` can't return `OnSuccess`. Everything that enters the data bag must implement `Value`, so it must be `Clone`, and serialise and deserialise with serde. And the `run` of an asynchronous workflow's executor returns a future, on which `?` doesn't compile until it is awaited.

Each of these is a compile error because the attributes write the workflow's declaration for you, and check it as they write it. But you haven't seen what they write.

## Outcomes

We have shown how the listing says how a workflow is put together, and how a mistake in putting it together is caught before any order runs.

| The usual way | With Itinera |
|---|---|
| Which hooks run after a step, or which client serves a repair, is often learned only by reading every module. | The listing names each step's policies, its adapter, and the services of its rescue step. |
| A wiring mistake, such as two handlers for one event, often shows only when a request takes that path in production. | The program doesn't compile, and the message names the mistake. |
