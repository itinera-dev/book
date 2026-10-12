# Trying again

## What you want to do

Carriers have limits. When the main carrier cannot take a parcel, you want shipping to be tried once more, through the backup carrier, before the order is given up. When an order can't be shipped at all, because it has no address, trying again wouldn't help, and you want it given up at once.

## The code

The ship step ships to the order's address, through the carrier it is lent, as in [chapter 12](12-giving-a-step-the-tools-it-needs.md). An order without an address is a failure that trying again wouldn't fix, so the step returns `Self::fail("no-address")`. A carrier that cannot take the parcel is a failure that another attempt might mend, so the step marks it retriable with `Self::fail_retriably("parcel-refused")`.

```example
name: ch13_backup_carrier
region: step
```

The workflow gives the ship step a retry budget of 1, with `retry_budget = 1` on its `#[with_step(...)]` line. Its carrier provider, marked `#[provider(per_attempt)]`, makes a fresh carrier for every attempt. It asks for the attempt number with `#[attempt] attempt: NonZeroU32`, and picks the carrier from it: the main carrier on attempt 1, which takes parcels up to 20 kg, and the backup carrier after, which takes up to 50 kg. The ship step never knows there are two.

```example
name: ch13_backup_carrier
region: workflow
```

Four orders go through, and each one's data takes it down a different path: a parcel of 2 kg, an order with no address, a parcel of 30 kg and one of 60 kg. The faked carriers print what they do, and the `report` function prints how each journey ended; [chapter 22](22-reading-the-result.md) shows how to read a result.

```example
name: ch13_backup_carrier
region: run
```

It prints:

```output
name: ch13_backup_carrier
```

## How it works

1. **An attempt is one execution of a step.** The first is attempt 1, which is why the attempt number is a `NonZeroU32`, and every attempt after it is a retry, except the attempt that follows a rescue step, which [chapter 14](14-repairing-then-trying-once-more.md) shows. Each attempt builds a fresh step from the data of the moment, so nothing carries over from a failed attempt but what was committed to the data bag.
2. **The retry budget is how many retries a step allows.** It is declared for each step in the workflow, with `retry_budget = N` on the step's line, and is 0 unless you say otherwise. With a budget of 1, the ship step runs at most twice.
3. **Only a retriable failure is retried.** The step says whether trying again might help, and the workflow decides whether it will. The order without an address fails through `Self::fail`, without the retriable flag: the step is given up at once, with the cause `failure`, even though its budget is untouched. The 30 kg parcel's failure is retriable, so it gets its second attempt, with the backup carrier, which takes it.
4. **An abnormal termination isn't retried, unless the step's line allows it.** An error the step didn't expect may already have done something, so by default an `Err` from `run` ends the step at once. Adding `abnormal_termination_retriable` to the step's `#[with_step(...)]` line declares that it may be retried, and it then uses the same budget as a retriable failure.
5. **When the budget is spent, the step is given up.** The 60 kg parcel is too heavy for both carriers. Its second attempt fails too, no retry is left, and the journey fails with the cause `retries exhausted` and the last attempt's reason.
6. **The events tell the whole story.** Every failed attempt is a fact, `StepFailed`, with its reason and its retriable flag. Then comes the decision: `StepRetrying` before another attempt, or `StepGivenUp` with its cause. [Chapter 23](23-seeing-what-happened.md) shows how to read them.
7. **Retries happen at once.** The executor never waits between attempts. A step that needs to wait for a carrier waits before it reports its failure.
8. **The choice of carrier belongs to the workflow.** The provider is part of the workflow, like the step's retry budget, and is called for each attempt. Here it reads the attempt number; the closing section of [Part 4](part-4-without-itinera.md#another-carrier-on-the-next-attempt) shows a step policy's `on step retry` hook choosing the carrier instead. Either way, the ship step only ships.

## Remember

1. A step says whether a failure is retriable; the workflow's retry budget decides whether it is retried.
2. A step with a retry budget of N runs at most N + 1 times; the budget is 0 unless the workflow declares it.
3. A failure that isn't retriable, from `Self::fail`, is never retried, whatever the budget.
4. An abnormal termination is retried only when the step's line says `abnormal_termination_retriable`, and it then uses the same budget.
5. Every attempt builds a fresh step: a failed attempt's contributions never reach the next one.
