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

The second is what the shop knows about an order when it comes in, here only its price. It is the order's **initial data**, and the shop gives it a piece at a time, each under a name, such as `data("price", 40L)`.

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

1. **A workflow lists its steps, and names them.** `@Workflow(name = "orders")`, from `org.itineradev.itinera.workflow`, declares the class `Orders` as a workflow named `orders`, and each `@WithStep(...)` written after it lists a step: its class, and its name in this workflow, here `charge`. With one step, the whole declaration fits on one line. The class implements `ItineraWorkflow` and holds nothing; the annotations are the whole declaration. **A step never knows its own name, nor which workflow runs it.**
2. **The initial data is where a journey's data starts.** `data("price", price)` puts the order's price there under the name `price`. That is the name the charge's input asks for, and that is how the charge gets its price: when the step is built, each of its inputs is found by its name in the journey's data, which starts as the initial data. The step still doesn't know where its price came from.
3. **A number is read as exactly the type it was given as.** The charge's input is a `Long`, and Kotlin writes a whole number as an `Int` unless it ends in `L`, so Ann's price is written `40L`. **Itinera never turns one kind of number into another:** an `Int` is not a `Long`, however small it is, so data a step reads as a `Long` must be given as one.
4. **One instance for each order.** `Orders()` makes the workflow object, an ordinary object of your class, and its `instance()`, which `ItineraWorkflow` gives it, starts an instance of the workflow. `data(...)` adds to its initial data, and `create()` creates it, with one order's data. The next order gets an instance of its own.
5. **The executor runs the instance as a journey.** A **journey** is one run of an instance, from its first step to its end. `LocalExecutor()`, from `org.itineradev.itinera.executor`, makes the **local executor** that comes with Itinera, which runs the journey right there, on the thread that calls its `run`, and returns once the journey has ended. It builds the charge with the order's price, calls the charge's `run`, and decides what follows from its outcome. **An instance runs once:** once it has run, it can never run again.
6. **The result says how the journey ended.** `run` returns a `JourneyResult`, whose `isSucceeded` and `isFailed` say whether the journey succeeded or failed. `run` throws a `Refusal` only for a journey that could not start at all, and neither order meets one; nothing here catches it, so it would end the program. **A journey that failed still returns its result: a failed order is a business outcome, not an error of your program.**
7. **What follows a step's outcome is not the step's decision.** Ann's charge succeeded, and since it was the workflow's last step, her journey succeeded. Hal's charge failed with `nothing-to-charge`, so the bank was never called, and his journey failed. The charge only reported the failure. Failing the journey because of it is what the executor does with a failed step by default, the same way for every step of every workflow.

So the shop charges its orders. But look at what Ann's journey gives back: only that it succeeded. The bank charged her 40, and the shop knows that only because the faked bank printed it. A business rule has to give a result the rest of the shop can use, and the charge has no way to give one yet.

## Outcomes

We have run a step for an order, as a journey, and read how it ended.

| The usual way | With Itinera |
|---|---|
| Code usually calls a business function directly, so each call site gathers what the function needs and passes it in. | The shop hands the executor an instance with the order's data, and the executor builds the step and runs it. |
| A failed order often comes back as a thrown exception, mixed with the program's own exceptions, so the caller has to tell them apart. | A failed journey is an ordinary result, and `run` throws a `Refusal` only for a journey that could not start. |
