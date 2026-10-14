# Running async journeys while others wait

## What you want to do

The shop's bank and carriers answer over the network, and while one order waits for them, the program's thread waits too. You want the shop to take other orders forward in the meantime.

## The code

The shop is [chapter 24](24-reading-the-result.md)'s, written in Itinera's other execution mode: asynchronous. Until now, everything ran in the synchronous one. The asynchronous mode needs the `async` feature of `itinera`, which you turn on in `Cargo.toml` with `features = ["async"]`, and an async runtime to run the program's futures. The shop uses tokio, a crate you add to `Cargo.toml` too; Itinera works with any runtime.

The payment client and the carrier now answer asynchronously, so their methods are `async fn`. The order book answers from the shop's own records without waiting, so it stays as it was. In Rust, a trait with `async fn` methods can't be used as `&dyn Payments`, the way a step asks for a service, unless it is marked `#[async_trait]`, from `itinera::mode`, and so is each implementation of it:

```example
name: ch29_running_async_journeys_while_others_wait
region: services
```

The faked bank and carriers await `network` before each answer. It hands the thread back once, as a real call does while it waits for the other side. The bank's implementation is marked `#[async_trait]` too:

```example
name: ch29_running_async_journeys_while_others_wait
region: network
```

Each step's `run` becomes an `async fn`, even the pricing's, which has nothing to wait for:

```example
name: ch29_running_async_journeys_while_others_wait
region: price
```

A step awaits the services it calls. The shipping's step reporter is now an `AsyncStepReporter`, whose `info` is awaited too:

```example
name: ch29_running_async_journeys_while_others_wait
region: ship
```

Each hook becomes an `async fn` as well:

```example
name: ch29_running_async_journeys_while_others_wait
region: heavy_parcels
```

`main` runs three of the day's orders, Ann's, Gil's and Jo's, at the same time. `order`, not shown, makes each one's instance as chapter 24's `main` did, and `journey` runs it with an `AsyncLocalExecutor` of its own. `#[tokio::main]` starts tokio's runtime and runs `main` on it, and `tokio::join!` awaits the three journeys together in `main`'s own task, so they take turns on the one thread that runs it, and the output is the same on every run; it returns once all of them have ended. The runtime has several threads by default; with one, as `#[tokio::main(flavor = "current_thread")]` asks, the shop runs just the same.

```example
name: ch29_running_async_journeys_while_others_wait
region: run
```

It prints:

```output
name: ch29_running_async_journeys_while_others_wait
```

## How it works

1. **`async fn` and `#[async_trait]` are all the shop needs.** Its steps and hooks become `async fn`, awaiting what they call, and the traits of the services that wait take `#[async_trait]`. The rest follows from those: the shipping awaits its step reporter, the role is `Sync`, and an asynchronous executor runs the journeys. The workflow's declaration, its input adapter, its ID generator and its providers don't change, nor do the log and the desk, which the asynchronous executor calls as they are.
2. **A workflow is in one execution mode, taken from its steps.** A step's `run` is synchronous or asynchronous, and the workflow takes its mode from its steps, so every step of the shop is asynchronous, even the pricing, whose `run` never awaits anything: a step's mode is the workflow's, not what its work needs. Its hooks are in the same mode.
3. **Each executor runs the workflows of its mode.** `AsyncLocalExecutor` runs asynchronous instances, and `LocalExecutor` synchronous ones. Its `run` returns a future, which runs the journey as it is awaited, and the journeys end as they did in chapter 24: Gil's parcel goes with the backup carrier, and Jo's card is renewed.
4. **A journey that waits lets the others go on.** Ann's journey runs until its charge waits for the bank. Then Gil's starts, and runs until it waits for the bank too, then Jo's. As each answer comes, its journey goes on from where it waited, so the log interleaves the three journeys, in the order the answers came. Each line names its journey, and each journey's events are still numbered from `#1`.
5. **An executor runs one journey at a time,** as [chapter 23](23-running-many-journeys.md) showed, so each journey that runs at the same time as others gets an executor of its own. An executor keeps nothing between journeys, so the three share nothing.
6. **A role an asynchronous hook asks for is `Sync`.** An asynchronous hook's future may move to another thread between two waits, and it holds the role while it runs. Rust lets it move only if the role can be used from several threads, which is what `Sync` says, so the shop's trait is now `trait Messages: Sync`.

Ann's, Gil's and Jo's journeys now go on together, each while the others wait. But the shop's log will soon write to a logging service, over the network too, where a write can fail.

## Outcomes

We have shown how the shop runs in the asynchronous execution mode, with journeys going on at the same time while others wait.

| The usual way | With Itinera |
|---|---|
| Making code asynchronous often means changing every function between the request and the call that waits. | The steps, the hooks and the services they await change, and the workflow takes its mode from its steps. |
| The log lines of requests handled at the same time are often hard to tell apart. | Every event names its journey and counts within it, however the journeys interleave. |
