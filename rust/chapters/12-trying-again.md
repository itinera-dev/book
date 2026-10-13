# Trying again

## What you want to do

A carrier that won't collect a parcel today may collect it on another try, so you want the shipping tried once more before an order is given up. A card that has expired won't be charged however often you try, so you want the charge given up at once, and the shop's log to say which of the two happened.

## The code

The charge now charges the order's card. The faked bank declines a card that has expired, and the charge then fails with the reason `card-expired`, as a plain failure: another attempt with the same card would be declined too.

```example
name: ch12_trying_again
region: charge
```

The shipping is [chapter 9](09-saying-what-only-a-step-knows.md)'s, with one change: when the carrier won't collect the parcel, it fails with `Self::fail_retriably("too-heavy")`, saying that another attempt might go better.

```example
name: ch12_trying_again
region: ship
```

The workflow gives the shipping a **retry budget** of 1, with `retry_budget = 1` on its `#[with_step(...)]` line:

```example
name: ch12_trying_again
region: workflow
```

The log is [chapter 11](11-naming-a-journey.md)'s, and now says more about some events: the number of each attempt, whether a failure was retriable, and why a step was given up.

```example
name: ch12_trying_again
region: detail
```

The shop runs three orders: Ann's and Gil's, each now with a card, and one from a new customer, Jo, whose card expired in 2024.

```example
name: ch12_trying_again
region: run
```

It prints:

```output
name: ch12_trying_again
```

## How it works

1. **An attempt is one execution of a step.** A step's first attempt is attempt 1, and every attempt after it is a retry, so in Rust the number is a `NonZeroU32`, which `step.attempt` holds in every event about a step. Each attempt builds the step afresh, from the data bag as it is when the attempt starts, and gets its own contributor. What Gil's first attempt contributed was never committed, as [chapter 6](06-passing-data-on.md) showed for any failed attempt, so his journey ends with no tracking number.
2. **A step says whether a failure is retriable.** `Self::fail_retriably` reports a failure that another attempt might mend, and `Self::fail` one it won't. The step knows which failures are worth another try, since only it knows what went wrong: a carrier may collect a parcel tomorrow, but an expired card stays expired.
3. **The workflow decides how many retries a step gets.** The retry budget is the number of retries the workflow allows a step, declared on the step's line, and is 0 unless the line says otherwise. With a budget of 1, the shipping runs at most twice. **A step says whether a failure is worth another attempt; the workflow decides whether it gets one.**
4. **Only a retriable failure is retried.** Gil's shipping failed with a retriable failure, and had a retry left, so the executor decided to retry it and recorded that as `step_retrying`, before attempt 2. Jo's charge failed with a failure that isn't retriable, so it was given up at once, whatever the budget, with the cause `failure`.
5. **When the budget is spent, the step is given up.** Gil's second attempt failed too, with no retry left, so the executor gave the shipping up with the cause `retries exhausted`, and the journey failed. Like every decision, each of these is an event, after the step's outcome it is about: the step reported each failure, and the executor decided what followed.
6. **Retries happen at once.** The executor never waits between attempts: attempt 2 starts as soon as attempt 1 has failed.

Now look at Gil's second attempt in the log. There is no `service_provided` before it: the main carrier was made once for the journey, the first time the shipping asked, as [chapter 7](07-giving-a-step-the-tools-it-needs.md) showed, so the second attempt was lent the same carrier, which refused the same parcel again. The shop has a backup carrier, which collects parcels up to 80 kg, but no attempt is ever offered it.

## Outcomes

We have shown how a step that fails can be tried again, and how the workflow decides how often.

| The usual way | With Itinera |
|---|---|
| A retry is often a loop around a call, written where the call is, so how often something is tried is decided in many places. | Every step's retry budget is declared on its line in the workflow, and a step only says whether its failure is worth another try. |
| Whether a failure was retried, and why it was finally given up, is usually known only to the loop that did it. | Every retry and every give-up is an event the executor records, with its cause, after the failure it answers. |
