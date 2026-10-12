# A workflow of one step

## What you want to do

The shop takes an order and charges its price. You want that charge to be a step of a workflow, run it for one order, and read what it charged once the journey is over.

## The code

The step holds the business rule: an order with nothing to charge is a failure, and any other order is charged. It is a struct, `Charge`, with an `impl` block.

```example
name: ch06_one_step
region: step
asynchronous: ch06_one_step_async
```

The asynchronous step is the same, with `async fn run` in place of `fn run`.

The workflow lists the step under a name. For now, charging is all the shop does.

```example
name: ch06_one_step
region: workflow
asynchronous: ch06_one_step_async
```

The asynchronous workflow is declared `asynchronous`. A workflow and its steps share one mode: a synchronous step in an asynchronous workflow, or the reverse, does not compile.

The journey starts with what the shop knows about the order, and ends with what you read back from it. Both are plain structs.

```example
name: ch06_one_step
region: data
```

Then you create an instance of the workflow for this order, and run it.

```example
name: ch06_one_step
region: run
asynchronous: ch06_one_step_async
```

The asynchronous form runs it on `AsyncLocalExecutor` instead, and waits for the future its `run` returns with `futures::executor::block_on`.

It prints:

```output
name: ch06_one_step
```

## How it works

1. **The step asks for its data, and gets it.** The charge declares that it needs a price: deriving `Step` makes each field marked `#[input]` an input, named after the field, so `price: i64` asks for the key `price`. It does not say where the price comes from: when the step is built, the workflow finds the price in the journey's data and hands it over. Here it comes from the order you started the journey with.
2. **The step reports what happened.** The `#[step]` attribute on the `impl` block declares `run`, which returns `Result<Outcome, Error>`. The charge succeeds, with `Self::succeed()`, or fails with a reason, with `Self::fail("nothing-to-charge")`. It says nothing about what should happen next; that is never a step's decision. And `run` takes `self` by value: the step is built for its attempt, and used up by it.
3. **The step contributes what it learned.** It asks for a contributor with a parameter marked `#[contributor]`, and adds the amount it charged under the key `charged`. Because the attempt succeeded, the contribution is committed to the journey's data, the data bag.
4. **The workflow is the plan.** The `#[workflow(name = "orders")]` attribute declares it, and its `#[with_step(Charge, name = "charge")]` line names the step `charge` and lists it. A step never knows its own name, nor which workflow runs it.
5. **One instance, one journey.** You create a workflow instance for each order, with that order's data. Deriving `InitialData` on `NewOrder` makes each of its fields a key of the initial data, and `Orders::builder().initial_data(...).create()?` creates the instance with them. You hand it to an executor, `LocalExecutor`, which runs the journey to its end and returns the result.
6. **The result says how the journey ended, and carries its output.** The executor's `run` returns `Result<JourneyResult, Refusal>`. The `Err` side is only for a journey that could not start, so a journey that failed is still `Ok`, and `result.status` says how it ended. This one succeeded, so its data bag is its output. Deriving `Output` on `Charged` lets `result.output()` read the data bag into that struct, one field per key, and you read what was charged from it.

Try changing the price to 0: the step fails, and so does the journey, so its data bag never receives `charged`. The program prints that the journey failed, then stops with an error when it tries to read what was charged: `result.output()` returns an error when a key the struct needs is missing, and the `?` passes it up out of `main`.

## Remember

1. A step declares the data it needs; the workflow decides where that data comes from.
2. A step reports what happened, never what to do next.
3. A step's contributions are committed to the data bag only when its attempt succeeds.
4. Every journey starts from a new workflow instance, made with that journey's data.
