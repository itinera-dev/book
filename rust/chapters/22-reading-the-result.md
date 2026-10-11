# Reading the result

## What you want to do

Every order's journey ends in one of three ways. You want to tell them apart and do the right thing for each: confirm an order that went through, tell the customer when it could not, undo what was already done, and call the developers when something is wrong with the shop's own code.

## The code

The workflow reserves the stock, then charges the price. Three orders run through it: one with a price, one with nothing to charge, and one whose price was never set.

```example
name: ch22_reading_the_result
region: run
```

Two small types say what the shop reads from the journey's output: the amount charged when it succeeded, and whether stock was reserved when it failed.

```example
name: ch22_reading_the_result
region: output
```

Once a journey has ended, the shop looks at how it ended before anything else, and acts on it.

```example
name: ch22_reading_the_result
region: read
```

A failed journey says why, and so does an aborted one. These two functions put that into words.

```example
name: ch22_reading_the_result
region: explain
```

It prints:

```output
name: ch22_reading_the_result
```

`result.status` is a `JourneyStatus`, an enum with one variant per status: `Succeeded` and `Failed` hold the data bag, `Failed` also holds a `Failure`, and `Aborted` holds an `Abort`. `Failure` has one variant per cause, holding the `Reason` a step or hook gave, or the `Error` that ended the last attempt, and `Abort` has one variant per abort reason, with its details. These enums are `#[non_exhaustive]`, so a `match` on them ends with a `_` arm, ready for what later versions add. `result.output::<T>()` reads the output as a type that derives `Output`; for an aborted journey it returns `OutputError::Aborted`, since there is no data bag to read. A panic is not a status: it is not caught, and it reaches whoever called `run`, which then returns no result.

## How it works

1. **The result is a business outcome.** It carries the journey ID and the final status, with what explains it, and, unless the journey was aborted, the data bag as it stood at the end. The data bag is the journey's output: the initial data plus everything that was committed.
2. **Each of the three statuses means something different to your users.**

   | Status | What happened | What it carries | What the shop does |
   |---|---|---|---|
   | succeeded | every step succeeded or skipped itself, or a policy finished the journey early | the output | confirms the order |
   | failed | a step failed and was not tried again, or a policy failed the journey | the cause, the reason or error when there is one, and the output | tells the customer, and undoes what was done |
   | aborted | something illegal happened, such as required data missing, or a hook or reporter that failed | the abort reason, its details, and the error when code of yours failed; no output | calls the developers |

3. **A failure explains itself.** Its cause is one of four. `failure`: the step reported a failure it did not mark as worth retrying. `retries exhausted`: a retriable failure, or a retriable abnormal termination, with no retry left in the budget. `abnormal termination`: an error escaped the step. `FailWorkflow`: a policy decided the journey fails. With the cause comes the reason, a code such as `nothing-to-charge`, when a step or policy gave one, and the error, when an error ended the last attempt.
4. **A failed journey may already have done some work.** Order A-2 failed at the charge, but the stock was reserved before that, and the output still says so: the contributions of every step that succeeded were committed. That is how the shop knows to release the stock. Failing is a legitimate end, decided by the outcomes and the policies, and the output is there to help you clean up after it.
5. **An aborted journey has no output.** Order A-3 has no price, so the charge could not even be built, and the journey was aborted with `required data missing`, naming the key `price`. An abort means something illegal happened, typically code that did not handle a situation as it should, and the data bag may stand halfway through a change, so the result does not offer it. The stock of A-3 was reserved too, but only the events, which chapter 23 shows, can tell you that. An aborted journey is never resumed or continued: fix the cause, then start the order again with a new instance.
6. **Which step failed is not in the result.** The result says how the journey ended, not how it got there: the name of the step that failed, each step's status and how many attempts each made are in the event stream.

## Remember

1. Look at the status before reading the output: an aborted journey has none.
2. A failed journey is a business answer you can give the customer; an aborted journey is a fault in the shop's own code or setup, to be fixed, and never resumed.
3. A failed journey may have done work before it failed; its output tells you what to undo.
4. Which step failed, and how often each step ran, are in the events, not in the result.
