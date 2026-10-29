# Trying again

## What you want to do

A carrier that won't collect a parcel today may collect it on another try, a booking system that failed may work a moment later, and a bank that declines a card may accept it on a second try, so you want the shipping and the charge tried once more before an order is given up. But when the bank's network fails while charging, the bank may already have taken the money, so you want that charge never tried again.

## The code

The bank and the carrier now say when something went wrong on their side: each throws an exception of the program's own, `NetworkDown` for the bank and `BookingFailed` for the carrier. Kotlin has no checked exceptions, so only their comments say so.

```example
name: ch12_trying_again
region: services
```

Each exception is a class of its own, whose message says what failed:

```example
name: ch12_trying_again
region: errors
```

The faked bank's network fails on any amount over 100, and the faked carrier's booking system on any parcel over 100 kg. The charge charges the order's card, and fails with `card-expired` when the bank declines it by giving back `null`. Nothing in `run` catches the bank's exception, so it escapes the step.

```example
name: ch12_trying_again
region: charge
```

The shipping is [chapter 9](09-saying-what-only-a-step-knows.md)'s, now giving the carrier the parcel's weight, and letting the carrier's exception escape too:

```example
name: ch12_trying_again
region: ship
```

The workflow gives both steps a **retry budget** of 1, with `retryBudget = 1` in their `@WithStep` annotations, and turns off retries for the charge's errors with `noRetryOnAbnormalTermination = true`. The bank's provider now gives the bank the current year; the ID generator and the carrier's provider are [chapter 11](11-naming-a-journey.md)'s.

```example
name: ch12_trying_again
region: workflow
```

The log and the desk are [chapter 11](11-naming-a-journey.md)'s, and the log's `detail` now also names each attempt, the error that ended one, and why a step was given up:

```example
name: ch12_trying_again
region: detail
```

Each order's data now holds its card, and the receipt read back may be missing:

```example
name: ch12_trying_again
region: data
```

The shop runs Ann's and Gil's orders, and three from new customers: Jo, whose card expired in 2024, Cat, who spends 120, and Dan, whose parcel weighs 150 kg.

```example
name: ch12_trying_again
region: run
```

It prints:

```output
name: ch12_trying_again
```

## How it works

1. **An attempt is one execution of a step.** Attempts count from 1, and every attempt after the first is a retry. Every event about a step carries its `Attempt`, from `org.itineradev.itinera.step`, as `event.attempt`. Each attempt builds the step afresh with its constructor, from the data bag, and what a failed attempt contributed is never committed, as [chapter 6](06-passing-data-on.md) showed, so Gil's journey ends with no tracking number.
2. **An exception that escapes is an abnormal termination.** A failure, such as `card-expired`, is an outcome the step chose. An exception thrown out of `run` is an error the step didn't handle. It ends the attempt as a failure of another kind, an **abnormal termination**, recorded as `step_abnormal_termination` with the exception's message. It never aborts the journey.
3. **The retry budget alone decides whether a step is tried again.** It is the number of retries the workflow allows a step, declared in its `@WithStep` annotation, 0 by default. While a retry is left, a step that failed is tried again, recorded as `step_retrying`: Gil's shipping and Jo's charge each get attempt 2. **A step reports what happened; the workflow decides whether it is tried again.**
4. **An abnormal termination spends the same budget.** Dan's booking failed with an exception, so the shipping was retried just as Gil's was, and failed again with the same parcel.
5. **The workflow can turn that off for a step.** `noRetryOnAbnormalTermination = true` on the charge says an error must never be retried there, since the bank may already have taken the money. Cat's charge ended in an exception, and was given up at once with the cause `abnormal termination`, though a retry was left. Jo's failure, which is not an error, was retried.
6. **When the budget is spent, the step is given up.** Each of Gil's, Jo's and Dan's steps failed again with no retry left, and was given up with the cause `retries exhausted`, the event's `cause.cause`. Each decision is an event, after the outcome it answers, and the executor never waits between attempts.
7. **A `java.lang.Error` is outside the model.** Itinera never catches an `Error`, such as a `StackOverflowError` or the `NotImplementedError` of Kotlin's `TODO()`: one thrown in a step stops the journey with no more events and no result, and reaches whoever called `run`. Throw an exception or return an outcome for anything an order can bring.

Look at Gil's second attempt in the log. There is no `service_provided` before it: the main carrier was made once for the journey, as [chapter 7](07-giving-a-step-the-tools-it-needs.md) showed, so the second attempt was lent the same carrier, which refused the same parcel again. The shop has a backup carrier, which collects parcels up to 80 kg, but no attempt is ever offered it.

## Outcomes

We have shown how the workflow decides, step by step, whether a step that failed is tried again.

| The usual way | With Itinera |
|---|---|
| A retry is often a loop around a call, written where the call is, so how often something is tried, and which errors are safe to try again, is decided in many places. | Every step's retry budget, and whether its errors are retried, is declared on its line in the workflow. |
| Whether a failure was retried, and why it was finally given up, is usually known only to the loop that did it. | Every retry and every give-up is an event the executor records, with its cause, after the failure it answers. |
