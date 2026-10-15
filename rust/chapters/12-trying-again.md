# Trying again

## What you want to do

A carrier that won't collect a parcel today may collect it on another try, a booking system that failed may work a moment later, and a bank that declines a card may accept it on a second try, so you want the shipping and the charge tried once more before an order is given up. But when the bank's network fails while charging, the bank may already have taken the money, so you want that charge never tried again.

## The code

The bank and the carrier now say when something went wrong on their side: each returns a `Result`, whose error is a type of the program's own, `NetworkDown` for the bank and `BookingFailed` for the carrier.

```example
name: ch12_trying_again
region: services
```

Each error is a struct of its own, whose message says what failed:

```example
name: ch12_trying_again
region: errors
```

The faked bank's network fails on any amount over 100, and the faked carrier's booking system on any parcel over 100 kg. The charge charges the order's card, and fails with `card-expired` when the bank declines it. The `?` after `payments.charge(...)` lets the bank's error escape the step.

```example
name: ch12_trying_again
region: charge
```

The shipping is [chapter 9](09-saying-what-only-a-step-knows.md)'s, with the same `?` after `carrier.book(...)`, and a failure, `too-heavy`, when the carrier won't collect the parcel.

```example
name: ch12_trying_again
region: ship
```

The workflow gives both steps a **retry budget** of 1, with `retry_budget = 1` on their `#[with_step(...)]` lines, and turns off retries for the charge's errors with `no_retry_on_abnormal_termination`:

```example
name: ch12_trying_again
region: workflow
```

The bank's provider now gives the bank the current year; the ID generator and the carrier's provider are [chapter 11](11-naming-a-journey.md)'s:

```example
name: ch12_trying_again
region: bank
```

The log and the desk are [chapter 11](11-naming-a-journey.md)'s, and the log now also names the error that ended an attempt, and why a step was given up:

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

1. **An attempt is one execution of a step.** Attempts count from 1, and every attempt after the first is a retry, so in Rust an attempt's number is an `Attempt`, which is never 0, held by `step.attempt` in every event about a step. Each attempt builds the step afresh from the data bag, and what a failed attempt contributed is never committed, as [chapter 6](06-passing-data-on.md) showed, so Gil's journey ends with no tracking number.
2. **An error that escapes is an abnormal termination.** A step's `run` returns `Result<Outcome>`, `itinera`'s own `Result`, whose error is `itinera::error::Error` unless another is named, as the bank's `Result<Option<String>, NetworkDown>` names one. A failure, such as `card-expired`, is an outcome the step chose. An `Err` is an error the step didn't handle, and `itinera::error::Error` converts from any error type that implements `std::error::Error`, `Send` and `Sync`, so `?` is all it takes. It ends the attempt as a failure of another kind, an **abnormal termination**, recorded as `step_abnormal_termination` with the error's message. It never aborts the journey.
3. **The retry budget alone decides whether a step is tried again.** It is the number of retries the workflow allows a step, declared on the step's line, and 0 unless the line says otherwise. While a retry is left, a step that failed is tried again, recorded as `step_retrying`: Gil's shipping and Jo's charge each get attempt 2. **A step reports what happened; the workflow decides whether it is tried again.**
4. **An abnormal termination spends the same budget.** Dan's booking failed with an error, so the shipping was retried just as Gil's was, and failed again with the same parcel.
5. **The workflow can turn that off for a step.** `no_retry_on_abnormal_termination` on the charge's line says an error must never be retried there, since the bank may have taken the money before its network failed. Cat's charge ended in an error, and was given up at once with the cause `abnormal termination`, though a retry was left. Jo's failure, which is not an error, was retried.
6. **When the budget is spent, the step is given up.** Each of Gil's, Jo's and Dan's steps failed again with no retry left, and was given up with the cause `retries exhausted`. Each decision is an event, after the outcome it answers, and the executor never waits between attempts.
7. **A panic is outside the model.** Itinera never catches one: a panic in a step stops the journey with no more events and no result. Return an error or an outcome for anything an order can bring.

Now look at Gil's second attempt in the log. There is no `service_provided` before it: the main carrier was made once for the journey, as [chapter 7](07-giving-a-step-the-tools-it-needs.md) showed, so the second attempt was lent the same carrier, which refused the same parcel again. The shop has a backup carrier, which collects parcels up to 80 kg, but no attempt is ever offered it.

## Outcomes

We have shown how the workflow decides, step by step, whether a step that failed is tried again.

| The usual way | With Itinera |
|---|---|
| A retry is often a loop around a call, written where the call is, so how often something is tried, and which errors are safe to try again, is decided in many places. | Every step's retry budget, and whether its errors are retried, is declared on its line in the workflow. |
| Whether a failure was retried, and why it was finally given up, is usually known only to the loop that did it. | Every retry and every give-up is an event the executor records, with its cause, after the failure it answers. |
