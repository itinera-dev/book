# Running async journeys while others wait

## What you want to do

The shop's bank and carriers answer over the network, and while one order waits for them, the program's thread waits too. You want the shop to take other orders forward in the meantime.

## The code

The shop is [chapter 25](25-reading-the-result.md)'s, written in Itinera's other **execution mode**: asynchronous, on Kotlin's coroutines, where a `suspend` function waits for an answer without holding its thread. `itinera-kotlin` brings this mode. To start coroutines itself, as `main` does, the shop adds kotlinx.coroutines, Kotlin's library of coroutines, to its build, as `org.jetbrains.kotlinx:kotlinx-coroutines-core`.

The payment client and the carrier now answer over the network, so their methods are `suspend` functions, still throwing [chapter 12](12-trying-again.md)'s errors. The order book answers from the shop's own records without waiting, so it stays as [chapter 22](22-finishing-a-journey-early.md) shows it:

```example
name: ch26_running_async_journeys_while_others_wait
region: services
```

The faked bank and carriers call `network` before each answer. It calls `yield()`, from kotlinx.coroutines, which hands the thread back once, as a real call does while it waits for the other side:

```example
name: ch26_running_async_journeys_while_others_wait
region: network
```

Each step's `run` becomes a `suspend` function, even the pricing's, which has nothing to wait for:

```example
name: ch26_running_async_journeys_while_others_wait
region: price
```

The shipping's step reporter is now an `AsyncStepReporter`, from `org.itineradev.itinera.step.async`, whose `info` is a `suspend` function too:

```example
name: ch26_running_async_journeys_while_others_wait
region: ship
```

Telling a customer and giving money back go over the network too, so the shop's roles become `suspend` the same way. A role whose methods don't wait may stay as it is, as a service may, like the order book:

```example
name: ch26_running_async_journeys_while_others_wait
region: roles
```

Each hook becomes a `suspend` function as well, and so does `refund`, which calls a role:

```example
name: ch26_running_async_journeys_while_others_wait
region: failed_shipping
```

The log and the desk now implement `AsyncReporter`, from `org.itineradev.itinera.report.async`, whose `report` is a `suspend` function:

```example
name: ch26_running_async_journeys_while_others_wait
region: reporters
```

The workflow class implements `ItineraAsyncWorkflow`, from `org.itineradev.itinera.workflow.async`, in place of `ItineraWorkflow`. Its providers and its input adapter become `suspend` functions, and its roles' methods call `network` before they answer:

```example
name: ch26_running_async_journeys_while_others_wait
region: workflow
```

Every other step and hook it names only adds `suspend`: the lookup and `AlreadyCompleted` are otherwise [chapter 22](22-finishing-a-journey-early.md)'s, the charge and the audit [chapter 20](20-an-optional-input.md)'s, `RenewCard` [chapter 14](14-repairing-then-trying-once-more.md)'s, `Outage` [chapter 17](17-giving-up-a-step-early.md)'s and `Notify` [chapter 19](19-letting-policies-act-on-the-outside-world.md)'s. The log's `detail` is [chapter 22](22-finishing-a-journey-early.md)'s, with the parts that page points to.

`main` runs three of the day's orders, Ann's, Gil's and Jo's, at the same time. `journey` makes each one's instance as [chapter 25](25-reading-the-result.md)'s `main` did, and runs it with an executor of its own, an `AsyncLocalExecutor`, from `org.itineradev.itinera.executor.async`. `runBlocking`, from kotlinx.coroutines, runs a coroutine on `main`'s own thread until it ends; inside it, `async` starts a coroutine for each journey, and `awaitAll` gives their results in order. The journeys take turns on that one thread, so the output is the same on every run. `NewOrder` is [chapter 25](25-reading-the-result.md)'s, and `Parcel` [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s:

```example
name: ch26_running_async_journeys_while_others_wait
region: run
```

It prints:

```output
name: ch26_running_async_journeys_while_others_wait
```

## How it works

1. **A workflow is in one execution mode, the one of the interface its class implements,** and so is every part of it: with `ItineraAsyncWorkflow`, every step, hook, provider, input adapter and reporter is `suspend`, even the pricing, which never waits. Only the ID generator is an ordinary function in both modes, since `create()`, an ordinary function, calls it.
2. **`AsyncLocalExecutor` runs asynchronous journeys,** as `LocalExecutor` runs synchronous ones, in the coroutine that calls its `run`. The journeys end as they did in chapter 25: Gil's parcel goes with the backup carrier, and Jo's card is renewed.
3. **A journey that waits lets the others go on.** Ann's journey runs until its charge waits for the bank. Then Gil's starts, and runs until it waits for the bank too, then Jo's. As each answer comes, its journey goes on from where it waited, so the log interleaves the three journeys, in the order the answers came. Each line names its journey, and each journey's events are still numbered from `#1`.
4. **The executor chooses no thread.** A journey runs on the dispatcher of the coroutine that calls `run`, here `runBlocking`'s, on `main`'s thread. On `Dispatchers.Default`, which has several threads, the shop runs just the same, though the lines of different journeys may come in another order. A `suspend` function must not block its thread, so a part that calls a blocking library calls it inside `withContext(Dispatchers.IO)`.
5. **An executor runs one journey at a time,** as [chapter 24](24-running-many-journeys.md) showed, so each journey that runs at the same time as others gets an executor of its own.

Ann's, Gil's and Jo's journeys now go on together, each while the others wait. But the only way to know that the shop still gives the same answers after a change is to run every order again, against the bank and the carriers, and read the log.

## Outcomes

We have shown how the shop runs in the asynchronous execution mode, with journeys going on at the same time while others wait.

| The usual way | With Itinera |
|---|---|
| Making code asynchronous often means changing every function between the request and the call that waits. | The parts of the workflow become `suspend` functions, and the workflow class implements `ItineraAsyncWorkflow`; the annotations that declare it, and what each part does, don't change. |
