# A first journey

## What you want to do

An order comes in, and you want the shop to charge it: run the charge of [chapter 4](04-a-first-step.md) with the order's price, and know how it went.

## The code

The charge is chapter 4's step, unchanged. To run it, the shop needs three things around it.

The first is a **workflow**: the list of steps that run for every order. For now, the shop's workflow has one step.

```example
name: ch05_first_journey
region: workflow
```

The second is what the shop knows about an order when it comes in, here only its price. It is the order's **initial data**, and in TypeScript it is a plain object, such as `{ price: 40 }`.

The third is the running itself. For each order, the shop creates an **instance** of the workflow with that order's initial data, and hands it to an **executor**, which runs it. Two orders come in: Ann's, of 40, and Hal's, a free sample that costs nothing.

```example
name: ch05_first_journey
region: run
```

It prints:

```output
name: ch05_first_journey
```

## How it works

1. **A workflow lists its steps, and names them.** `@workflow({ name: "orders" })`, from `itinera/workflow`, declares the class `Orders` as a workflow named `orders`. It goes on the top line, and each `@withStep(...)` line under it lists a step: its class, and its name in this workflow, here `charge`. The class extends `Workflow` and holds nothing; the decorators are the whole declaration. **A step never knows its own name, nor which workflow runs it.**
2. **The initial data is where a journey's data starts.** Each property of `{ price }` is a piece of data under the property's name, so the price is there under the name `price`. That is the name the charge's input asks for, and that is how the charge gets its price: when the step is built, each of its inputs is found by its name in the journey's data, which starts as the initial data. The step still doesn't know where its price came from.
3. **One instance for each order.** `Orders.build()` makes the workflow object the shop runs, once, before any order; you never make one with `new`. Its `instance({ price })` creates an instance of the workflow with one order's data. The next order gets an instance of its own: **an instance runs once.**
4. **The executor runs the instance as a journey.** A **journey** is one run of an instance, from its first step to its end. `new LocalExecutor()`, from `itinera/executor`, makes the **local executor** that comes with `itinera`, which runs the journey right there, in your program. Its `run` returns a promise, which the shop awaits: the program is an ES module, so it can `await` at its top level. The executor builds the charge with the order's price, calls the charge's `run`, decides what follows from its outcome, and settles the promise once the journey has ended.
5. **The result says how the journey ended.** The promise resolves to a `JourneyResult`, and `result.status.kind` says how the journey ended, here `succeeded` or `failed`. The promise rejects only for a journey that could not start at all, with a refusal, and neither order meets one; a rejection would end the program at its `await`. **A journey that failed still resolves: a failed order is a business outcome, not an error of your program.**
6. **What follows a step's outcome is not the step's decision.** Ann's charge succeeded, and since it was the workflow's last step, her journey succeeded. Hal's charge failed with `nothing-to-charge`, so the bank was never called, and his journey failed. The charge only reported the failure. Failing the journey because of it is what the executor does with a failed step by default, the same way for every step of every workflow.

So the shop charges its orders. But look at what Ann's journey gives back: only that it succeeded. The bank charged her 40, and the shop knows that only because the faked bank printed it. A business rule has to give a result the rest of the shop can use, and the charge has no way to give one yet.

## Outcomes

We have run a step for an order, as a journey, and read how it ended.

| The usual way | With Itinera |
|---|---|
| Code usually calls a business function directly, so each call site gathers what the function needs and passes it in. | The shop hands the executor an instance with the order's data, and the executor builds the step and runs it. |
| A failed order often comes back as a thrown error, mixed with the program's own errors, so the caller has to tell them apart. | A failed journey is an ordinary result, and `run` rejects only for a journey that could not start. |
