# Trying again

## What you want to do

A carrier that won't collect a parcel may collect it on another try, a booking system that failed may work a moment later, and a declined card may go through on a second try, so you want the shipping and the charge tried once more before an order is given up. But when the bank's network fails while charging, the bank may already have taken the money, so you want that charge never tried again.

## The code

The bank and the carrier now throw an error of the program's own when something goes wrong on their side, `NetworkDown` for the bank and `BookingFailed` for the carrier; their tokens, `PAYMENTS` and `CARRIER`, are [chapter 7](07-giving-a-step-the-tools-it-needs.md)'s. The bank also charges a card now, and gives back `undefined` when it declines one.

```example
name: ch12_trying_again
region: services
```

Each error extends `Error`, with a message that says what failed:

```example
name: ch12_trying_again
region: errors
```

The faked bank's network fails on any amount over 100, and the carrier's booking system on any parcel over 100 kg. The charge charges the order's card, and fails with `card-expired` when the bank declines it. Nothing in it catches what `payments.charge(...)` throws, so the bank's error escapes `run`.

```example
name: ch12_trying_again
region: charge
```

The shipping is [chapter 9](09-saying-what-only-a-step-knows.md)'s, now giving the carrier the parcel's weight, and it catches nothing either.

```example
name: ch12_trying_again
region: ship
```

The workflow gives both steps a **retry budget** of 1, with `retryBudget: 1` in their `@withStep` options, and turns off retries for the charge's errors with `noRetryOnAbnormalTermination: true`. The bank's provider now gives the bank the year, 2026; the ID generator and the carrier's provider are [chapter 11](11-naming-a-journey.md)'s.

```example
name: ch12_trying_again
region: workflow
```

The log and the desk are [chapter 11](11-naming-a-journey.md)'s; the log now also names each attempt, the error that ended one, and why a step was given up:

```example
name: ch12_trying_again
region: detail
```

Each order's data now holds its card, and the receipt read back may be missing. The shop runs Ann's and Gil's orders, and three from new customers: Jo, whose card expired in 2024, Cat, who spends 120, and Dan, whose parcel weighs 150 kg.

```example
name: ch12_trying_again
region: run
```

It prints:

```output
name: ch12_trying_again
```

A step can't count its attempts on its class, so this shipping step doesn't compile:

```compile-error
name: ch12_step_keeps_static_data
```

## How it works

1. **Attempts count from 1.** An attempt is one run of a step, and every attempt after the first is a retry. Every event of an attempt, such as `step_retrying`, carries it as `event.attempt`, an `Attempt` from `itinera/step`. Each attempt builds the step afresh from the data bag, and what a failed attempt contributed is never committed, as [chapter 6](06-passing-data-on.md) showed, so Gil's journey ends with no tracking number.
2. **An error that escapes is an abnormal termination.** A failure, such as `card-expired`, is an outcome the step chose. A throw out of `run`, or a promise from `run` that rejects, is an error the step didn't handle. It ends the attempt as a failure of another kind, an **abnormal termination**, recorded as `step_abnormal_termination` with the error's message, and never aborts the journey.
3. **The retry budget alone decides whether a step is tried again.** It is the number of retries the workflow allows a step, declared on its `@withStep` line, 0 by default. While a retry is left, a step that failed is tried again, recorded as `step_retrying`: Gil's shipping and Jo's charge each get attempt 2. **A step reports what happened; the workflow decides whether it is tried again.**
4. **An abnormal termination spends the same budget.** Dan's booking failed with an error, so the shipping was retried just as Gil's was, and failed again with the same parcel.
5. **The workflow can turn that off for a step.** `noRetryOnAbnormalTermination: true` on the charge's line says an error must never be retried there, since the bank may already have taken the money. Cat's charge ended in an error, and was given up at once with the cause `abnormal termination`, though a retry was left. Jo's failure, which is not an error, was retried.
6. **When the budget is spent, the step is given up.** Gil's, Jo's and Dan's steps each failed again with no retry left, and were given up with the cause `retries exhausted`. The executor never waits between attempts.
7. **Every throw is caught.** JavaScript has no failure code can't catch, so Itinera catches them all: a mistake in `run`, such as reading a property of `undefined`, is an abnormal termination just as the bank's error is. Return an outcome for anything an order can bring.
8. **A step keeps nothing on its class.** A static field holding data would be shared by every attempt of every journey, so Itinera refuses one: the step above doesn't compile, and a `private static` field is refused as the program loads. A static method is fine.

Look at Gil's second attempt. There is no `service_provided` before it: the main carrier was made once for the journey, as [chapter 7](07-giving-a-step-the-tools-it-needs.md) showed, so the second attempt got the same carrier, which refused the same parcel again. The shop has a backup carrier, which collects parcels up to 80 kg, but no attempt is ever offered it.

## Outcomes

We have shown how the workflow decides, step by step, whether a step that failed is tried again.

| The usual way | With Itinera |
|---|---|
| A retry is often a loop around a call, written where the call is, so how often something is tried, and which errors are safe to try again, is decided in many places. | Every step's retry budget, and whether its errors are retried, is declared on its line in the workflow. |
| Whether a failure was retried, and why it was finally given up, is usually known only to the loop that did it. | Every retry and every give-up is an event the executor records, with its cause, after the failure it answers. |
