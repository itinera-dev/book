# Adding a step

## What you want to do

Once an order is charged, the shop ships it. You want shipping to be a second step of the same workflow, run after the charge, and to read the tracking number once the journey is over.

## The code

The charge is the step of the previous chapter, with one change: it now calls the shop's bank, faked here so that it prints what it charges. The new step ships the order through the carrier, and contributes the tracking number the carrier gives back.

```example
name: ch07_two_steps
region: step
```

The workflow now lists two steps. They run in the order they are written: the charge, then the shipping.

```example
name: ch07_two_steps
region: workflow
```

```note
name: ch07_two_steps
```

The order now carries its ID as well as its price, and the shop reads back both what was charged and where the parcel is.

```example
name: ch07_two_steps
region: data
```

You run the journey exactly as before.

```example
name: ch07_two_steps
region: run
```

It prints:

```output
name: ch07_two_steps
```

## How it works

1. **The workflow's list is the order.** Steps run in the order the workflow lists them, and nothing reorders them. To read what the shop does with an order, you read that list, top to bottom.
2. **The steps do not know about each other.** The charge does not call the shipping, and the shipping does not check that the charge happened. Neither knows it is part of a list. The workflow alone puts one after the other.
3. **Each step asks only for what it needs.** The charge asks for the price, and the shipping asks for the order's ID. Both come from the same journey data, which started with the order you gave, and each step receives only what it declared.
4. **The next step runs only when the previous one is finished.** The executor considers the first step that is not finished, runs it, and moves on only once it has succeeded, or skipped itself, as [chapter 10](10-saying-how-it-went.md) shows. If the charge fails, the journey fails there: the shipping never runs, and keeps the status `NotExecuted`, the status of a step that has not run.
5. **The output holds everything committed.** When the journey succeeds, its data bag holds the order you started with plus what each step contributed: `charged` from the charge and `tracking` from the shipping.

Try changing the price to 0: the charge fails with `nothing-to-charge`, the carrier is never called, and the journey fails, so its data bag never receives `charged` or `tracking`. The program prints that the journey failed, then stops with an error when it tries to read them.

## Remember

1. Steps run in the order the workflow lists them; to change the order, change the list.
2. A step never calls another step, and never knows which step comes before or after it.
3. The steps after a failed step never run.
