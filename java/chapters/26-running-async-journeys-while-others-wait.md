# Running async journeys while others wait

## What you want to do

The shop's bank and carriers answer over the network, and while one order waits for them, the program's thread waits too. You want the shop to take other orders forward in the meantime.

## The code

Itinera has two **execution modes**, synchronous and asynchronous, and Java writes workflows in the synchronous one only: every step, hook, provider and reporter is an ordinary method, and a journey runs on the thread that calls `run`. So the shop runs journeys side by side the way Java runs any work side by side: each on a thread of its own.

The shop is [chapter 25](25-reading-the-result.md)'s, unchanged but for its fakes. The faked bank and carriers now call `Network.roundTrip()` before each answer, which waits a moment, as a real call does while the other side answers, and holds its thread meanwhile. The order book answers from the shop's own records, without the network, so it stays as [chapter 22](22-finishing-a-journey-early.md) shows it:

```example
name: ch26_running_async_journeys_while_others_wait
region: network
```

The workflow class calls it too, in the methods of its roles, since telling a customer and giving money back go over the network as well:

```example
name: ch26_running_async_journeys_while_others_wait
region: roles
```

Journeys that run at the same time print at the same time, and their lines would mix in another order on every run. So the program gives its journeys a console of its own, which keeps what each thread prints apart until that thread takes it:

```example
name: ch26_running_async_journeys_while_others_wait
region: console
```

`main` runs three of the day's orders, Ann's, Gil's and Jo's, at the same time. It makes a pool of three threads with `Executors.newFixedThreadPool`, and starts each order's journey on it with `CompletableFuture.supplyAsync`, both from `java.util.concurrent`. `journey` makes each one's instance as [chapter 25](25-reading-the-result.md)'s `main` did, runs it with a `LocalExecutor` of its own, and gives back what the journey printed, with its result. Once every journey has ended, `main` prints each journey's lines together, in the order of the orders, so the output is the same on every run. `NewOrder` is [chapter 25](25-reading-the-result.md)'s, and `Parcel` [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s:

```example
name: ch26_running_async_journeys_while_others_wait
region: run
```

It prints:

```output
name: ch26_running_async_journeys_while_others_wait
```

## How it works

1. **A Java workflow is in the synchronous execution mode,** and so is every part of it. The asynchronous mode, where a part waits for an answer without holding its thread, belongs to languages whose code can suspend, such as Kotlin with its coroutines. Java's part waits by holding its thread, so a program that wants many journeys at once gives each one a thread.
2. **`LocalExecutor` runs a journey on the thread that calls its `run`,** here a thread of the pool. The journeys end as they did in chapter 25: Gil's parcel goes with the backup carrier, and Jo's card is renewed.
3. **A journey that waits lets the others go on.** While Ann's charge waits for the bank, her thread waits with it, and Gil's and Jo's journeys go on, on their own threads. The three journeys run at the same time, and each one's events are still numbered from `#1`, since every line names its journey and every journey counts its own.
4. **The executor chooses no thread.** It never starts one: the journey runs wherever `run` is called. The pool here has a thread for each order; a shop with many orders makes a pool of the size it can afford, and `supplyAsync` queues the rest until a thread is free.
5. **An executor runs one journey at a time,** as [chapter 24](24-running-many-journeys.md) showed, so each journey that runs at the same time as others gets an executor of its own.
6. **A refusal still ends the program.** A `Refusal` is checked, and the work `supplyAsync` runs can't throw a checked exception, so `journey` wraps it in a `CompletionException`, which `join()` throws in `main`, as the refusal itself did in [chapter 24](24-running-many-journeys.md).

Ann's, Gil's and Jo's journeys now go on together, each while the others wait. But the only way to know that the shop still gives the same answers after a change is to run every order again, against the bank and the carriers, and read the log.

## Outcomes

We have shown how the shop runs journeys at the same time, each on a thread of its own while others wait.

| The usual way | With Itinera |
|---|---|
| Running requests side by side often means sharing one client or one handler between threads, and guarding everything it keeps. | Each journey runs on its own thread with an executor of its own, and its steps and policies are built for it, so nothing in one journey is shared with another. |
