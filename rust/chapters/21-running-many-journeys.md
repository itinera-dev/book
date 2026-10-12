# Running many journeys

## What you want to do

Orders come in all day. You want to run the shop's workflow once for every order, each journey with that order's own data, and know what to do on the rare occasion when the executor's `run` won't start a journey at all.

## The code

The workflow charges, then ships. Each journey is named after its order, as [chapter 20](20-naming-your-journeys.md) showed, so the output below is the same on every run.

```example
name: ch21_many_journeys
region: workflow
```

Each order is the data one journey starts with: a struct that derives `InitialData`, whose fields become the keys of the data bag. The second order has nothing to charge.

```example
name: ch21_many_journeys
region: data
```

One executor runs them all, one after another. For each order you create a new workflow instance with that order's data, and hand it to the executor.

```example
name: ch21_many_journeys
region: run
```

It prints:

```output
name: ch21_many_journeys
```

Sometimes a journey can't even start. Here the shop's `notify` policy, which tells the customer how the order went, needs the texts the shop sends, and the file that holds them is missing. The policy is built by its `#[factory]` function before the journey starts, so building it fails first.

```example
name: ch21_refused
region: policy
```

This time the call to `run` gives back a refusal instead of a result.

```example
name: ch21_refused
region: run
```

It prints:

```output
name: ch21_refused
```

An instance runs one journey and no more. Handing the same instance to the executor a second time is a mistake, and Rust won't compile it:

```compile-error
name: ch21_instance_run_twice
```

## How it works

1. **The workflow is declared once; its instances are many.** The workflow is the plan, the same for every order. A workflow instance is one order's copy of that plan: it's made for one journey, with that journey's initial data, its journey ID and its reporters, and it holds everything the journey changes. Its initial data is what the data bag starts with.
2. **Creating an instance is three calls.** `Orders::builder()` starts a new instance, `initial_data` gives it the order, and `create` makes it. Creating it can fail before any journey, with an `InstanceError`, when the workflow's ID generator or a reporter fails while the instance is made. That isn't a refusal, since the executor isn't involved yet; the `?` in the loop simply hands the error to `main`.
3. **The executor runs one journey at a time and remembers nothing.** The executor here is `LocalExecutor`, the local executor, which runs a journey inside your program, without saving anything: the journey lives only as long as the call to `run`. When `run` returns, nothing of that journey remains in the executor, so the failure of order A-2 has no effect on order A-3, and the same executor runs every order of the day. On the asynchronous executor, `AsyncLocalExecutor`, the loop is the same, with the call to `run` awaited.
4. **A journey that fails still has a result.** Order A-2 has nothing to charge, so its journey failed, but `run` returned its result like any other. Whatever happens inside a journey, a failure or even an abort, is in its result, which [chapter 22](22-reading-the-result.md) reads in detail.
5. **An instance runs at most one journey, and the compiler checks it.** The executor's `run` takes the instance by value, so once an instance has run it's gone: using it again is the "use of moved value" error above, not something that waits to be found while the shop is open. To run an order again, for example after the customer has corrected it, create a new instance with the corrected data.
6. **Side by side means several executors.** An executor never runs two journeys at once. Its `run` borrows it mutably, so starting a second journey on the same executor while one is running doesn't compile either. To run journeys in parallel, give each thread its own `LocalExecutor`.
7. **A refusal means no journey started.** The executor's `run` returns `Result<JourneyResult, Refusal>`, and a refusal is the `Err`: the journey couldn't start, so there's no journey, no event and no result. Nothing ran, so order A-1 was not charged and there is nothing to undo. Fix the cause, then run the order again with a new instance.
8. **In Rust, only your own code that runs before the journey can be refused.** The specification lists four causes of a refusal: the workflow is put together wrongly, the executor doesn't accept its execution mode, the initial data holds something that is not a value, or code of yours that runs before the journey starts fails. The first three never reach `run` in Rust. A workflow put together wrongly is caught when it is built: by the compiler when it is declared with the attributes, or by `build` when it is declared by hand. A step, hook or reporter in a mode the executor doesn't accept doesn't compile. And initial data can only hold values, since its types must implement `Value`. What's left is your code that runs before the journey starts, and each variant of `Refusal` names it and carries its error: `Refusal::WorkflowPolicy` when a workflow policy's `#[factory]` returns an `Err`, since workflow policies are built once for each journey before it starts, and two more variants when the code that delivers events to the reporters, which [chapter 23](23-seeing-what-happened.md) shows, fails as the journey is set up.

## Remember

1. Create a new workflow instance for every journey, with that journey's own data; an instance moves into `run`, so it can never run twice.
2. An executor runs one journey at a time and keeps nothing between journeys; to run journeys side by side, use one executor for each.
3. A journey that fails or is aborted still returns a result; `run` refuses only a journey that can't start.
4. A refusal means nothing ran and no event was emitted: fix its cause, then run a new instance.
