# A workflow of one step

## What you want to do

The shop takes an order and charges its price. You want that charge to be a step of a workflow, run it for one order, and read what it charged once the journey is over.

## The code

The step holds the business rule: an order with nothing to charge is a failure, and any other order is charged.

```example
name: ch06_one_step
region: step
asynchronous: ch06_one_step_async
```

The workflow lists the step under a name. For now, charging is all the shop does.

```example
name: ch06_one_step
region: workflow
asynchronous: ch06_one_step_async
```

The journey starts with what the shop knows about the order, and ends with what you read back from it.

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

It prints:

```output
name: ch06_one_step
```

```note
name: ch06_one_step
```

## How it works

1. **The step asks for its data, and gets it.** The charge declares that it needs a price. It does not say where the price comes from: when the step is built, the workflow finds the price in the journey's data and hands it over. Here it comes from the order you started the journey with.
2. **The step reports what happened.** It succeeds, or it fails with a reason, `nothing-to-charge`. It says nothing about what should happen next; that is never a step's decision.
3. **The step contributes what it learned.** It adds the amount it charged, under the key `charged`. Because the attempt succeeded, the contribution is committed to the journey's data, the data bag.
4. **The workflow is the plan.** It names the step `charge` and lists it. A step never knows its own name, nor which workflow runs it.
5. **One instance, one journey.** You create a workflow instance for each order, with that order's data, and hand it to an executor, which runs the journey to its end and returns the result.
6. **The result says how the journey ended, and carries its output.** The journey succeeded, so its data bag is its output, and you read what was charged from it.

Try changing the price to 0: the step fails, and so does the journey, so its data bag never receives `charged`. The program prints that the journey failed, then stops with an error when it tries to read what was charged.

## Remember

1. A step declares the data it needs; the workflow decides where that data comes from.
2. A step reports what happened, never what to do next.
3. A step's contributions are committed to the data bag only when its attempt succeeds.
4. Every journey starts from a new workflow instance, made with that journey's data.
