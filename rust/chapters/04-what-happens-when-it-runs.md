# What happens when it runs

## What you want to do

You have a workflow; now you want to run it for real orders, and to know the name of each thing involved: the workflow instance, the journey, the executor, the data bag, the result and the events. You also want to know where Itinera fits in your system, and where it does not.

## The code

This shop is smaller than the one in [chapter 3](03-read-the-workflow-know-what-happens.md): two steps, charge then ship, and nothing around them, so that its events stay short. The workflow lists `DebugReporter`, the ready-made reporter you met in the [quick start](quick-start.md), which prints every event of a journey, one per line.

```example
name: ch04_running
region: workflow
asynchronous: ch04_running_async
```

In the asynchronous form, the workflow is declared `asynchronous`, and each step's `run` is an `async fn`. `DebugReporter` stays synchronous: `AsyncLocalExecutor` also accepts synchronous reporters, and calls them in line.

A journey starts with what the shop knows about the order:

```example
name: ch04_running
region: data
```

The program runs two orders, which differ only in their data. Ann's costs 40, less a discount of 5. Ben's costs 1,200, more than the bank accepts, so the bank declines it. The asynchronous form runs them on `AsyncLocalExecutor`, whose `run` returns a future; here `futures::executor::block_on` waits for it.

```example
name: ch04_running
region: run
asynchronous: ch04_running_async
```

It prints:

```output
name: ch04_running
```

## How it works

### The words

1. **Workflow instance.** For every order, your code creates a new **workflow instance** from the workflow, with that order's initial data: `Orders::builder().initial_data(...).create()?`. The instance holds everything about that order's journey: its data, its journey ID and its reporters. An instance runs once; the next order gets a new one. Rust holds you to that: `run` takes the instance by value, so once it has run, it is gone, and running it again does not compile, as [chapter 21](21-running-many-journeys.md) shows.
2. **Journey.** A **journey** is one execution of an instance, from its first step to its result. The output shows two journeys, one per order.
3. **Executor.** The **executor** runs the instance. It builds each step afresh for every attempt, runs it, calls the hooks of the attached policies after each outcome, and carries out what they decide. The executor here, `LocalExecutor`, runs the journey in your process and keeps nothing once it returns: no data, no state, nothing saved to disk. It runs one journey at a time: its `run` takes it as `&mut self`, so the compiler itself prevents two journeys from running on one executor at the same time. To run journeys side by side, you create several executors.
4. **Data bag.** The **data bag** holds the journey's data, as values under string keys. It starts with the initial data: three keys for Ann's order, two for Ben's, which has no discount. It changes only when a contribution is committed: a step's contributions when its attempt succeeds, a hook's when the hook returns. That is why Ann's data bag ends with `charged` and `tracking`, and Ben's holds only what it started with. When the journey ends, the data bag is its output.
5. **Result.** What `run` returns is the **result**, a `JourneyResult`: the journey ID, how the journey ended, with what explains it, in `result.status`, and, unless it was aborted, the data bag. A journey **succeeded** when every step succeeded or skipped itself, or a hook finished it early. It **failed** when a step failed or a hook failed it, and the result carries why. This program prints how each journey ended and what its data bag holds, but the events show why Ben's failed: the charge reported a failure, `declined`. It was **aborted** when something illegal happened, such as a hook that failed or required data that was missing; an aborted journey has no output and is never resumed. All three are `Ok`: `run` returns `Result<JourneyResult, Refusal>`, and its `Err` side is only for a journey that cannot even start. Then `run` returns a **refusal** instead of a result: no journey, and no events. In Rust, most mistakes in how a workflow is put together never reach `run` at all, because the macros refuse them when the program compiles. A panic is outside the model altogether: Rust treats it as an unrecoverable failure, `itinera` never catches one, and it reaches whoever called `run`.
6. **Events.** Everything that happens in a journey is an **event**, numbered in order, and the workflow's reporters receive every one. `DebugReporter` prints each as Rust's `Debug` format writes it: its `sequence` number, its timestamp, the journey's ID, the workflow, and its `body`, which says what happened. The timestamp and the journey's ID change every time you run it. Some events are facts: `StepFailed` says the charge failed, with its reason. Others are decisions: `StepGivenUp` says the executor will not try the charge again, and why, and `JourneyFailed` says the journey failed. Read Ben's journey line by line and you see the order of things: the step's outcome, the decision about the step, then the decision about the journey. Every journey reports the same kinds of event in the same order, whoever wrote the steps, so one reporter can log every workflow you have; [chapter 23](23-seeing-what-happened.md) shows how to write your own.
7. **Execution modes.** A step, a hook or a reporter is either **synchronous** or **asynchronous**, and an executor accepts one or both modes. In Rust, the asynchronous form is an `async fn`, and it needs the `async` feature of `itinera`. In the asynchronous form of this program, every step is asynchronous and runs on `AsyncLocalExecutor`; it prints the same, apart from the timestamps and journey IDs. `LocalExecutor` accepts only synchronous workflows, and `AsyncLocalExecutor` only asynchronous ones, though it also accepts synchronous reporters, such as `DebugReporter`. Something in a mode the executor does not accept is refused before the journey starts, and in Rust that means before the program runs: a synchronous step in an asynchronous workflow, or the reverse, does not compile, and nor does an asynchronous workflow handed to `LocalExecutor`, as [chapter 27](27-mistakes-caught-before-a-journey.md) shows.

### Where Itinera fits

Itinera fits a piece of business work made of steps that run in a fixed order, with decisions around them: try again, repair and try once more, give up, finish early, tell someone. A checkout, a sign-up, an invoice run or a refund are good examples. What Itinera is not:

1. **Not a job queue.** It does not schedule work or hand it to other machines. `run` runs one journey, now, in your process, and returns its result. It never waits between attempts: a step that needs to wait does so itself.
2. **Not BPMN.** There is no diagram language and no engine server. A workflow is a declaration in your own Rust code, compiled and checked with the rest of it.
3. **No branching inside a workflow.** The steps always run in the order the workflow lists them. Nothing reorders them or jumps between them. A step can skip itself, and a policy can end the journey early, and that is all.
4. **Not durable, yet.** A journey lives only as long as the call that runs it. Nothing is saved, so a journey cut short by a crash is not resumed.

## Remember

1. Create a new workflow instance for every journey; an instance runs only once.
2. Only committed contributions change the data bag: a step's when its attempt succeeds, a hook's when it returns.
3. Read how a journey ended from its result, and what happened during it from its events.
4. The steps run in the order the workflow lists them; nothing branches, reorders or jumps.
5. An aborted journey has no output, and it is never resumed.
