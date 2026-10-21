# Declaring by hand

## What you want to do

The decorators have declared the shop's workflow since the first chapters, and checked it. You want to see what they stand for, and to write part of the shop without them: for a compiler that records no decorator metadata, for a declaration put together as the program starts, or to have its mistakes reported where you choose.

## The code

### The shop by hand

The shop is the one [chapter 36](36-orders-refused-before-they-start.md) runs, declared in the **long form**: with the builder rather than the decorators, for its pricing step, its audit and its workflow. The pricing is [chapter 20](20-an-optional-input.md)'s class without decorators. Its builder, `Price.builder("price")`, takes its requests in order: `needs` one for each parameter of the constructor, and `run` one for each of `run`'s, each made by the decorator's own function, such as `input("discount", optional(integer))` or `contributor()`:

```example
name: ch38_declaring_by_hand
region: price
```

The audit is [chapter 34](34-saying-which-attempt-took-the-money.md)'s class without decorators. Its builder names the policy, and `onStepSuccess` makes its method `charged` the hook, with a request for each parameter, in order:

```example
name: ch38_declaring_by_hand
region: audit
```

`Orders` keeps the role implementations of [chapter 26](26-running-async-journeys-while-others-wait.md), and its ID generator, input adapter and providers are plain methods now:

```example
name: ch38_declaring_by_hand
region: parts
```

`Parcel` is [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s, `OrderBook` is [chapter 22](22-finishing-a-journey-early.md)'s, `Carrier`, `Messages` and `Refunds` are [chapter 26](26-running-async-journeys-while-others-wait.md)'s, and `Payments`, `Cards`, `CARDS` and `cardRegistry` are [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s, with the tokens `PAYMENTS` and `CARRIER` of [chapter 7](07-giving-a-step-the-tools-it-needs.md) and `ORDER_BOOK` of [chapter 22](22-finishing-a-journey-early.md).

And `Orders.builder()` puts the workflow together, part by part. The other steps and policies keep their decorators: each goes in as its class, a step through its own builder, which reads its requests from them. A method of `Orders` is named, followed by its requests:

```example
name: ch38_declaring_by_hand
region: workflow
```

The rest is as earlier chapters show it: the lookup and `AlreadyCompleted` in [chapter 22](22-finishing-a-journey-early.md), the charge in [chapter 33](33-giving-every-reporter-what-it-needs.md), `RenewCard` in [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md), the shipping, `Notify` and `FailedShipping` in [chapter 26](26-running-async-journeys-while-others-wait.md), `Outage` in [chapter 34](34-saying-which-attempt-took-the-money.md), the log in [chapter 33](33-giving-every-reporter-what-it-needs.md), [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md) and [chapter 36](36-orders-refused-before-they-start.md), and the desk in [chapter 11](11-naming-a-journey.md).

`shop` replaces `Orders.build("email")`, and each order's instance is made from it as [chapter 33](33-giving-every-reporter-what-it-needs.md) shows, its result followed up by [chapter 34](34-saying-which-attempt-took-the-money.md)'s `followUp`. The program runs Ann's order and Jo's, and prints:

```output
name: ch38_declaring_by_hand
```

### A workflow put together wrongly

This small shop is declared by hand throughout, and `charge()` and `ship()` start a step's builder:

```example
name: ch38_violations
region: shop
```

It is put together with three mistakes: `ship` listed twice, two policies of the charge that both define `on step success`, and the parcels adapter attached to `shipping`, a step the workflow doesn't have:

```example
name: ch38_violations
region: wrong
```

The mistakes are found when the program calls `build()`, which throws a `Violations`, as the decorators do in [chapter 37](37-mistakes-caught-before-a-journey.md):

```example
name: ch38_violations
region: report
```

Fixed, the builder's `listing()` lists its steps:

```example
name: ch38_violations
region: fixed
```

It prints:

```output
name: ch38_violations
```

## How it works

1. **Both forms make the same declaration.** A workflow is a **workflow descriptor**: its name, a **step descriptor** for each step, in order, giving its name, how to build it, its retry budget, policies and rescue step, then the workflow's policies, reporters, ID generator, input adapters and providers. The decorators record it from your classes, and the builder takes it part by part. The executor runs either the same way, and the two forms mix, as here: write by hand only what needs it.
2. **Each decorator stands for a call of a builder.** A step's decorators stand for its builder's `needs` and `run`, and `@withStep` for the workflow builder's `step`, its name for the step builder's, and its options for `retryBudget`, `rescue` and `policy`. `@stepPolicy` and a hook's decorator stand for a policy's builder and its `onStepSuccess`, or its other hooks' methods. `@workflow` stands for `name`, `reporter` and `policy`, `@idGenerator()`, `@inputAdapter` and `@provider` for the methods of their names, and `Orders.build("email")` for `constructWith("email")` and `build()`.
3. **TypeScript checks each request against its parameter.** A request missing, or for a parameter of another type, doesn't compile, and a step whose `needs` or `run` is never given throws an `Error` naming the parameter when the workflow's builder takes it.
4. **`build()` checks the declaration, as admission.** It throws a `Violations` listing every violation, as `@workflow` does, but where the program calls it. `listing()` runs the same check without making the workflow object.
5. **When you need the long form.** With a compiler that records no decorator metadata, such as esbuild, tsx or Vite, declare every step, rescue step, policy and workflow with its `builder`; of [chapter 3](03-installing-itinera.md)'s compiler options, only `strict` is then needed. It also suits a declaration put together from configuration as the program starts. `itinera` has no options, as chapter 3 said, so nothing turns the decorators off: a program uses the form its compiler allows.

What the book hasn't said is how to decide, for a new rule of the shop, which part it belongs in.

## Outcomes

We have shown what the decorators stand for, how to write it by hand, and when a workflow declared by hand reports its mistakes.

| The usual way | With Itinera |
|---|---|
| Code wired by decorators often works only with the compilers that support them. | The builder takes the same declaration part by part, from classes without decorators. |
| Wiring checked at start-up usually runs wherever the framework decides, before the program has a say. | `build()` checks it at the point the program chooses, and throws every violation there. |
