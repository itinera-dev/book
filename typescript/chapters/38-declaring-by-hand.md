# Declaring by hand

## What you want to do

The decorators have declared the shop's workflow since the first chapters, and checked it. You want to see what they stand for, and to write part of the shop that way yourself: for a compiler that records no decorator metadata, for a declaration put together when the program starts, or for a mistake you'd rather catch where you choose than as a module loads.

## The code

### The shop by hand

The shop is the one [chapter 36](36-orders-refused-before-they-start.md) runs, declared in the **long form**: with the builder rather than the decorators, for its pricing step, its audit and its workflow. The pricing is [chapter 20](20-an-optional-input.md)'s class without decorators. Its own builder, `Price.builder("price")`, takes its requests: `needs` one for each parameter of the constructor, in order, and `run` one for each parameter of `run`, each made by the function that made the decorator, such as `input("discount", optional(integer))` or `contributor()`:

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

`Parcel` is [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s, `OrderBook` is [chapter 22](22-finishing-a-journey-early.md)'s, `Carrier`, `Messages` and `Refunds` are [chapter 26](26-running-async-journeys-while-others-wait.md)'s, and `Payments`, `Cards` and `cardRegistry` are [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s.

And `Orders.builder()` puts the workflow together, part by part. The other steps and policies are still declared with the decorators: each goes in as its class, a step through its own builder, which reads its requests from its decorators. A method of `Orders` is named, followed by its requests:

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

1. **Both forms make the same declaration.** A workflow is a **workflow descriptor**: its name, a step descriptor for each step in order, its policies, reporters, ID generator, input adapters and providers. The decorators record it from your classes, and the builder takes it part by part. The executor runs either the same way: Ann's order is charged and shipped, and Jo's card renewed, as with the decorators.
2. **Each decorator stands for a call of a builder.** A step's decorators stand for its builder's `needs` and `run`, and the options of `@withStep` for its `retryBudget`, `rescue` and `policy`. `@stepPolicy` and a hook's decorator stand for a policy's builder and its `onStepSuccess`, or the method of its other hook. `@workflow` stands for `name`, `reporter` and `policy`, `@idGenerator()`, `@inputAdapter` and `@provider` for the methods of the same names, and `Orders.build("email")` for `constructWith("email")` and `build()`.
3. **TypeScript checks each request against its parameter.** A request for a parameter of another type doesn't compile, and a parameter left without one throws an `Error` from that call, saying which.
4. **`build()` checks the declaration, as admission.** It throws a `Violations` listing every violation, as `@workflow` does, but when the program calls it. `listing()` runs the same check without making the workflow object.
5. **The two forms mix.** Most of the shop is still declared with the decorators. Write by hand only the part that needs it.
6. **When you need the long form.** With a compiler that records no decorator metadata, such as esbuild, tsx or Vite, write every step, rescue step and policy, and the workflow, by hand, each class with a `builder` of its own; of [chapter 3](03-installing-itinera.md)'s compiler options, only `strict` is then needed. Or for a declaration put together from configuration when the program starts or violations caught where you choose. The package has no option to turn the decorators off, since it has none: a program uses the form its compiler allows. Otherwise, the decorators say the same in fewer lines.

The shop can now declare every part of its workflow, in either form. What the book hasn't said is how to decide, for a new rule of the shop, which part it belongs in.

## Outcomes

We have shown what the decorators stand for, how to write it by hand, and when a workflow declared by hand reports its mistakes.

| The usual way | With Itinera |
|---|---|
| Code wired by decorators often works only with the compilers that support them. | The builder takes the same declaration part by part, from plain classes, with any compiler. |
| A wiring mistake is often found wherever the code that wires it happens to run. | `build()` throws every violation where the program calls it. |
