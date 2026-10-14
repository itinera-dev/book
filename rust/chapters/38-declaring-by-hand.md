# Declaring by hand

## What you want to do

The attributes have written the shop's workflow for you since the first chapters, and checked it as they wrote it. You want to see what they write, and to write part of the shop that way yourself: for a declaration put together when the program starts, from the shop's configuration, or for a mistake you'd rather read as a value than as a compile error.

## The code

### The shop by hand

The shop is the one [chapter 36](36-orders-refused-before-they-start.md) runs, declared in the **long form**: without the attributes, for its pricing step, its audit and its workflow. The pricing is two things: the step, which runs once and implements `Step`, and what builds it for each attempt, which implements `StepFactory`, says what the step needs and builds it from what was resolved. Each input is named once, by a constant holding its key and its type, such as `Input<i64>` or `OptionalInput<i64>`, which both `needs` and `build` use, and `Resolved` hands over what was resolved for the attempt:

```example
name: ch38_declaring_by_hand
region: price
asynchronous: ch38_declaring_by_hand_async
```

The audit implements the trait of its one hook, `OnStepSuccess<Orders>`: its `needs` lists what the hook requests, as a `HookNeeds`, and the hook reads it from `Requested` when it is called, with the journey ID, the step's name and the attempt, which every step hook gets:

```example
name: ch38_declaring_by_hand
region: audit
asynchronous: ch38_declaring_by_hand_async
```

The ID generator, the input adapter and the providers are plain methods of `Orders`, each reading what it requested from its own `Requested`, such as `Provider<PerAttempt>`, which carries the attempt:

```example
name: ch38_declaring_by_hand
region: parts
asynchronous: ch38_declaring_by_hand_async
```

`Parcel` is [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s, `OrderBook` is [chapter 22](22-finishing-a-journey-early.md)'s, `Payments` and `Carrier` are [chapter 12](12-trying-again.md)'s, and `Cards` and `card_registry` are [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s.

And the workflow is put together by its builder, part by part, each service under a constant holding its key and its trait. The other steps and policies are still declared with the attributes, and go into the builder through their `descriptor`, and the rescue step through `rescued_by`:

```example
name: ch38_declaring_by_hand
region: workflow
asynchronous: ch38_declaring_by_hand_async
```

The rest is as earlier chapters show it: the lookup and `already_completed` in [chapter 22](22-finishing-a-journey-early.md), the charge in [chapter 33](33-giving-every-reporter-what-it-needs.md), `RenewCard` in [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md), the shipping in [chapter 17](17-giving-up-a-step-early.md), `outage` in [chapter 34](34-saying-which-attempt-took-the-money.md), `failed_shipping` in [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md), `notify` in [chapter 19](19-letting-policies-act-on-the-outside-world.md), the log in [chapter 33](33-giving-every-reporter-what-it-needs.md) and [chapter 36](36-orders-refused-before-they-start.md), and the desk in [chapter 11](11-naming-a-journey.md).

In the asynchronous form, the pricing implements `AsyncStep` and `AsyncStepFactory`, the audit `AsyncOnStepSuccess`, and every descriptor is made with its asynchronous constructor, such as `WorkflowDescriptor::async_builder`. `main` calls `orders()` once, before any journey, makes each order's instance with `orders.instance(Orders { channel: "email" })`, runs Ann's order and Jo's, and prints:

```output
name: ch38_declaring_by_hand
asynchronous: ch38_declaring_by_hand_async
```

### A workflow put together wrongly

This small shop is declared by hand with three mistakes: `ship` listed twice, two policies of the charge that both define `on step success`, and the parcels adapter attached to `shipping`, a step the workflow doesn't have:

```example
name: ch38_violations
region: wrong
```

The program compiles. `build` returns the violations instead of the workflow, and `main` reads them:

```example
name: ch38_violations
region: report
```

Fixed, it builds, and `main` prints its listing:

```example
name: ch38_violations
region: fixed
```

It prints:

```output
name: ch38_violations
```

## How it works

1. **Both forms make the same descriptors.** A workflow is a **workflow descriptor**: its name, a step descriptor for each step in order, its policies, reporters, ID generator, input adapters and providers. The attributes write them from your types, and the long form writes them by hand. The executor runs either the same way.
2. **Each attribute stands for a trait or a builder.** `#[step]` makes the step's `StepDescriptor`, through `descriptor(name)`, from what `#[derive(Step)]` says the step needs. `#[step_policy]` implements a trait for each hook and makes a `StepPolicyDescriptor`. `#[provider]` makes a `ServiceProviderDescriptor`, and `#[role]` implements `Provides<dyn Messages>` for the workflow. `#[workflow(...)]` builds the `WorkflowDescriptor` with `WorkflowDescriptor::builder`, once. And `local_executor_for!(Orders)` stands for `LocalExecutor::new()`, or `AsyncLocalExecutor::new()` for an asynchronous workflow, which this `main` uses.
3. **A step is built afresh for every attempt.** The factory builds `Price` from what was resolved for that attempt, which is why `Price<'a>` borrows its contributor: it lives for one attempt. Nothing carries over to the next.
4. **`build` checks the declaration, as admission.** It returns `Result<WorkflowDescriptor<Orders>, Violations>`: the workflow, or every violation found, each displayed as the specification names it. The attributes run the same checks while the program compiles, as [chapter 37](37-mistakes-caught-before-a-journey.md) showed; the long form runs them when the program starts.
5. **The two forms mix.** Most of the shop here is still declared with the attributes. Write by hand only the part that needs it.
6. **When you need the long form.** A declaration put together from configuration when the program starts, such as a gift-wrapping step only in the shops that offer it; violations read as values; a step or policy generic over a type, which the attributes refuse; or a program built without the attributes at all, with `default-features = false` on `itinera`, which turns off its `macros` option. Otherwise, the attributes say the same in fewer lines.

The shop can now declare every part of its workflow, in either form. What the book hasn't said is how to decide, for a new rule of the shop, which part it belongs in.

## Outcomes

We have shown what the attributes write, how to write it by hand, and how a workflow declared by hand reports its mistakes.

| The usual way | With Itinera |
|---|---|
| Wiring written by a framework's annotations is often hidden in code nobody reads. | The attributes write public descriptors, which you can write yourself, part by part. |
| A wiring mistake found at start-up often stops at the first one. | `build` returns every violation, as values. |
