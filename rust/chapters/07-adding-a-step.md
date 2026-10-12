# Adding a step

## What you want to do

Once an order is charged, the shop ships it. You want shipping to be a second step of the same workflow, run after the charge, and only when the charge succeeded.

## The code

The charge is the step of the previous chapter. The new step ships the order through the carrier, faked here so that it prints the parcel it collects.

```example
name: ch07_two_steps
region: step
```

The workflow now lists two steps, one `#[with_step(...)]` line each. They run in the order they are written: the charge, then the shipping.

```example
name: ch07_two_steps
region: workflow
```

The order now carries its ID as well as its price.

```example
name: ch07_two_steps
region: data
```

You run the journeys as before, for two orders: A-1 of 40, and A-2, which costs nothing.

```example
name: ch07_two_steps
region: run
```

It prints:

```output
name: ch07_two_steps
```

## How it works

1. **The workflow's list is the order.** Each step of the workflow is one `#[with_step(...)]` line below `#[workflow(...)]`, with the step's type and its name, and the steps run in the order of those lines. Nothing reorders them. To read what the shop does with an order, you read that list, top to bottom.
2. **The steps do not know about each other.** The charge does not call the shipping, and the shipping does not check that the charge happened. Neither knows it is part of a list. The workflow alone puts one after the other.
3. **Each step asks only for what it needs.** The charge asks for the price, and the shipping asks for the order's ID, with its field `order: String` marked `#[input]`. Both come from the same journey data, which started with the order you gave, and each step receives only what it declared.
4. **The next step runs only when the previous one is finished.** The executor considers the first step that is not finished, runs it, and moves on only once it has succeeded, or skipped itself, as [chapter 10](10-saying-how-it-went.md) shows. A-2's charge fails with `nothing-to-charge`, so its journey fails there: the carrier is never called, and the shipping keeps the status the specification calls `NotExecuted`, the status of a step that has not run.

## Remember

1. Steps run in the order the workflow lists them; to change the order, change the list.
2. A step never calls another step, and never knows which step comes before or after it.
3. The steps after a failed step never run.
