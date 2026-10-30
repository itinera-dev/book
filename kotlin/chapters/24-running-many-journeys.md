# Running many journeys

## What you want to do

Orders come in all day, and the shop shouldn't make a new executor for each one. You want one executor that runs every order of the day, one journey after another, each starting from its own order and nothing else.

## The code

The shop is [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md)'s, except that its output no longer holds the refund, which `main` no longer prints:

```example
name: ch24_one_executor_for_every_order
region: output
```

`main` now makes one executor before the loop, and hands it each order's instance in turn:

```example
name: ch24_one_executor_for_every_order
region: run
```

`NewOrder` and `Parcel` are as [chapter 21](21-deciding-where-a-steps-data-comes-from.md) shows them, and `main` runs that chapter's five orders, none of which the order book has completed. It prints:

```output
name: ch24_one_executor_for_every_order
```

An instance runs one journey and no more, and an executor runs one journey at a time. Kotlin can't see either mistake when it compiles the program, so the executor checks both when the program runs. The next program has [chapter 5](05-a-first-journey.md)'s shop, which only charges, with [chapter 4](04-a-first-step.md)'s charge. It hands one executor Ann's instance twice, then Cat's while Ben's journey is still running, on a thread of its own, started with `thread { ... }` from `kotlin.concurrent`. For that, `main` tells the faked bank to hold the next charge, Ben's, until `main` lets it answer, and waits until that charge has started. `said` takes a function that runs an instance, calls it, and says how its journey ended, or why the run was refused:

```example
name: ch24_runs_refused
region: run
```

It prints:

```output
name: ch24_runs_refused
```

## How it works

1. **An executor keeps nothing between journeys.** The local executor runs a journey inside the call to its `run`, and saves nothing: the journey lives only as long as that call. When `run` returns, nothing of that journey remains in the executor, so the next journey starts from its own order alone. Gil's shipping needed the backup carrier and Jo's card was replaced, yet Kim's journey, run next by the same executor, starts like every other: `journey_started` with Kim's data, the lookup's first attempt, and the bank made anew for Kim's charge, as `service_provided` shows. Its log starts again at `#1`, since an event's sequence number counts the events of its own journey, never those of the journeys before it. **What one journey did never reaches another through the executor.**
2. **The loop still creates an instance for every order.** An instance is made for one journey, and runs once, as [chapter 5](05-a-first-journey.md) showed. Handing it to `run` a second time is refused: `run` throws a `Refusal`, from `org.itineradev.itinera.executor`, an `Exception` whose `refused` says why, here `InstanceRan`, and whose message says what to do instead. No journey starts, so the bank charges Ann only once. To run an order again, for example after the customer has corrected it, create a new instance with the corrected data.
3. **An executor runs one journey at a time.** `run` runs the journey on the thread that called it, and returns only once the journey has ended. Ben's journey was still running when Cat's instance was handed over from `main`'s thread, so that run was refused with `JourneyRunning`, and Ben's journey went on to succeed. That is why the shop's loop waits for each journey before it hands over the next order. **These two refusals are mistakes in the program, found when it runs:** neither is an answer of the business, so the shop's loop doesn't catch them, and one there would end the program.

The executor now runs every order. But for Lee's order, `main` learns only that the journey failed. The log shows why, line by line, but the code that runs the shop has only the result, and reads from it only how the journey ended and its output.

## Outcomes

We have shown how one executor runs every order of the day, each journey starting from its own data.

| The usual way | With Itinera |
|---|---|
| Code that handles many requests often keeps state between them, in a cache or a field, and a request can be affected by the ones before it. | The executor keeps nothing between journeys: each starts from its own order, with services made for it. |
| Running the same request twice by mistake often goes unnoticed, until a customer is charged twice. | The executor refuses an instance it has run before, with a `Refusal` saying why, so an instance never runs a second journey. |
