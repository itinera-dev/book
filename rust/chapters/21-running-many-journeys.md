# Running many journeys

## What you want to do

Orders come in all day. You want to run the shop's workflow once for every order, each journey with that order's own data, and know what to do on the rare occasion when `run` will not start a journey at all.

## The code

The workflow charges, then ships. Each journey is named after its order, as chapter 20 showed, so the output below is the same on every run.

```example
name: ch21_many_journeys
region: workflow
```

Each order is the data one journey starts with. The second order has nothing to charge.

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

`Orders::builder()` starts a new instance, `initial_data` gives it the order, and `create` makes it. `create` can fail before any journey, with an `InstanceError`, when the workflow's ID generator or a reporter fails while the instance is made; that is not a refusal, since the executor is not involved yet. `run` takes the instance by value, so an instance cannot run twice, and it borrows the executor mutably, so one executor cannot run two journeys at once: both mistakes are compile errors rather than refusals. To run journeys in parallel, give each thread its own `LocalExecutor`. On the asynchronous executor, `AsyncLocalExecutor`, the loop is the same, with the call to `run` awaited.

Sometimes a journey cannot even start. Here the shop's `notify` policy, which tells the customer how the order went, needs the texts the shop sends, and the file that holds them is missing. The policy is built before the journey starts, so building it fails first.

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

`run` returns `Result<JourneyResult, Refusal>`, and the refusal is the `Err`. Its variants name what refused the journey, and each carries the error: `Refusal::WorkflowPolicy` when a workflow policy's `#[factory]` returns an `Err`, and `Refusal::DispatcherFactory` or `Refusal::Dispatcher` when the executor's dispatcher factory, or its dispatcher while the reporters are added, fails. The other causes of a refusal never reach `run` in Rust: a workflow put together wrongly is caught when it is built, by the compiler when it is declared with the attributes, or by `build` when it is declared by hand; a step, hook or reporter in a mode the executor does not accept does not compile; and initial data can only hold values, since its types must implement `Value`.

An instance runs one journey and no more. Handing the same instance to the executor a second time is a mistake:

```compile-error
name: ch21_instance_run_twice
```

## How it works

1. **The workflow is declared once; its instances are many.** The workflow is the plan, the same for every order. A workflow instance is one order's copy of that plan: it is made for one journey, with that journey's initial data, its journey ID and its reporters, and it holds everything the journey changes. Its initial data is what the data bag starts with.
2. **The executor runs one journey at a time and remembers nothing.** The executor here is the local executor, which runs a journey in your process, without saving anything: the journey lives only as long as the call to `run`. When `run` returns, nothing of that journey remains in the executor, so the failure of order A-2 has no effect on order A-3, and the same executor runs every order of the day.
3. **A journey that fails still has a result.** Order A-2 has nothing to charge, so its journey failed, but `run` returned its result like any other. Whatever happens inside a journey, a failure or even an abort, is in its result, which chapter 22 reads in detail.
4. **An instance runs at most one journey.** Once an instance has run, it is spent. To run an order again, for example after the customer has corrected it, create a new instance with the corrected data.
5. **Side by side means several executors.** An executor never runs two journeys at once. To run journeys in parallel, give each line of work its own executor.
6. **A refusal means no journey started.** `run` refuses an instance, instead of running it, when the journey cannot start: the workflow is put together wrongly, the executor does not accept its execution mode, the initial data holds something that is not a value, or code of yours that runs before the journey starts fails. That code is the workflow policies, which are built once for each journey before it starts, and the dispatcher that will deliver the journey's events. A refusal has no journey, no event and no result: nothing ran, so order A-1 was not charged and there is nothing to undo. Fix the cause, then run the order again with a new instance.

## Remember

1. Create a new workflow instance for every journey, with that journey's own data, and never run an instance twice.
2. An executor runs one journey at a time and keeps nothing between journeys; to run journeys side by side, use one executor for each.
3. A journey that fails or is aborted still returns a result; `run` refuses only a journey that cannot start.
4. A refusal means nothing ran and no event was emitted: fix its cause, then run a new instance.
