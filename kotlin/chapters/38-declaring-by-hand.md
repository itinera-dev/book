# Declaring by hand

## What you want to do

The annotations have declared the shop's workflow, and the checker has checked it as the program was built. You want to see what the annotations stand for, and to write part of the shop that way yourself: for a declaration put together when the program starts, from the shop's configuration, or for a mistake you'd rather read as a value than as an error of the build.

## The code

### The shop by hand

The shop is the one [chapter 36](36-orders-refused-before-they-start.md) runs, declared in the **long form**: without the annotations, for its pricing step, its audit and its workflow. Each request is made once, by `Requests` from `org.itineradev.itinera.request`, such as `Requests.input("price", typeToken<Long>())`. The pricing is a plain class implementing `SyncStepRun`, and `pricing()` declares it: `StepDescriptor.builder("price")` takes what builds it for each attempt from `got`, the typed answers to the requests `request` lists:

```example
name: ch38_declaring_by_hand
region: price
asynchronous: ch38_declaring_by_hand_async
```

The audit is a plain class too. `StepPolicyDescriptor.builder` names it and makes it, and `onStepSuccess` takes its hook's requests, in order, and calls `charged` with their answers:

```example
name: ch38_declaring_by_hand
region: audit
asynchronous: ch38_declaring_by_hand_async
```

`Orders` implements both roles but no longer `ItineraWorkflow`, so its ID generator, input adapter and providers are plain methods now, and build the `Supplied` and `Provided.Instance` that `supply`, `fromDataBag` and `provide` made before:

```example
name: ch38_declaring_by_hand
region: parts
asynchronous: ch38_declaring_by_hand_async
```

`Parcel` is [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s, `OrderBook` is [chapter 22](22-finishing-a-journey-early.md)'s, `Carrier` is [chapter 12](12-trying-again.md)'s, `Messages` [chapter 19](19-letting-policies-act-on-the-outside-world.md)'s, `Refunds` [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md)'s, and `Payments`, `Cards` and `cardRegistry` are [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s.

And `WorkflowDescriptor.builder` puts the workflow together, part by part. The other steps and policies keep their annotations, and go in through `StepDescriptor.builder("lookup", Lookup::class.java)`, `StepPolicyDescriptor.of`, `RescueStepDescriptor.of` and `WorkflowPolicyDescriptor.of`, which read each class alone:

```example
name: ch38_declaring_by_hand
region: workflow
asynchronous: ch38_declaring_by_hand_async
```

The rest is as earlier chapters show it: the lookup and `AlreadyCompleted` in [chapter 22](22-finishing-a-journey-early.md), the charge in [chapter 33](33-giving-every-reporter-what-it-needs.md), `RenewCard` in [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md), the shipping in [chapter 17](17-giving-up-a-step-early.md), `Outage` in [chapter 34](34-saying-which-attempt-took-the-money.md), `FailedShipping` in [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md), `Notify` in [chapter 19](19-letting-policies-act-on-the-outside-world.md), the log in [chapter 33](33-giving-every-reporter-what-it-needs.md), [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md) and [chapter 36](36-orders-refused-before-they-start.md), and the desk in [chapter 11](11-naming-a-journey.md).

In the asynchronous form, the pricing implements `AsyncStepRun`, and every `suspend` part is declared with a function of `itinera-kotlin`, such as `asyncStep`, `asyncStepPolicy`, `asyncBuilder<Orders>("orders")` or `asyncStepPolicyOf(Outage::class)`.

`main` calls `orders()` once, and makes each order's instance with `instance`, from `org.itineradev.itinera.instance`:

```example
name: ch38_declaring_by_hand
region: instance
asynchronous: ch38_declaring_by_hand_async
```

`followUp` is [chapter 34](34-saying-which-attempt-took-the-money.md)'s. In the asynchronous form, `instance` is from `org.itineradev.itinera.instance.async`, and each journey runs on an `AsyncLocalExecutor` inside `runBlocking`. `main` runs Ann's order and Jo's, and prints:

```output
name: ch38_declaring_by_hand
asynchronous: ch38_declaring_by_hand_async
```

### A workflow put together wrongly

This small shop has steps and policies of its own, `parcelsFor` declares its adapter, and `detail` says what a violation is:

```example
name: ch38_violations
region: shop
```

It is declared by hand with three mistakes: `ship` listed twice, two policies of the charge that both define `on step success`, and the parcels adapter attached to `shipping`, a step the workflow doesn't have:

```example
name: ch38_violations
region: wrong
```

The program builds: the checker reads a workflow only in a class that implements `ItineraWorkflow`, which `Shop` doesn't, and reads each step and policy alone, and each is right on its own. `build()` throws a `Violations`, from `org.itineradev.itinera.workflow`, and `main` reads it:

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

1. **Both forms make the same descriptors, and they mix.** A workflow is a **workflow descriptor**: its name, a step descriptor for each step in order, its policies, roles, reporters, ID generator, input adapters and providers. The annotations declare them, the long form writes them, and the executor runs either the same way, so write by hand only what needs it.
2. **Each annotation stands for a builder's call.** `@WithStep` stands for `StepDescriptor.builder` and its `retryBudget`, `rescue` and `policy`, and each `@Input` of a step for a `request`. `@StepPolicy` and a hook's annotation stand for `StepPolicyDescriptor.builder` and its `onStepSuccess`, or another hook's method. `@Workflow` stands for `WorkflowDescriptor.builder`, `reporter` and `policy`, a role the workflow implements for `role`, and `@IdGenerator`, `@InputAdapter` and `@Provider` for the methods of their names. `ReporterFactory { Log() }` makes each journey's log, and `PolicyFactory(::Audit)` the audit for each attempt.
3. **A step is built afresh for every attempt,** by the function `StepDescriptor.builder` takes, as in [chapter 12](12-trying-again.md).
4. **`build()` checks the declaration, as admission.** It throws a `Violations` listing every violation, each a `Violation` named as the specification names it. The checker finds them in an annotated workflow as the program is built, as [chapter 37](37-mistakes-caught-before-a-journey.md) showed; it never sees a workflow declared by hand, and checks alone each annotated part that no annotated workflow names, so how the parts are put together is left to `build()`.
5. **When you need the long form.** A declaration put together from configuration when the program starts, such as a gift-wrapping step only in the shops that offer it; violations read as values; or a class you can't annotate, such as one from a library. No option of itinera turns the annotations off: a program uses whichever form it needs.

What the book hasn't said is how to decide, for a new rule of the shop, which part it belongs in.

## Outcomes

We have shown what the annotations stand for, how to write it by hand, and how a workflow declared by hand reports its mistakes.

| The usual way | With Itinera |
|---|---|
| Wiring declared by a framework's annotations is often read by code nobody sees. | The annotations stand for public builders, which you can call yourself, part by part. |
| A wiring mistake found at start-up often stops at the first one. | `build()` throws every violation at once, each a value. |
