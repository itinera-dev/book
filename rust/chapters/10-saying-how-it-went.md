# Saying how it went

## What you want to do

Not every order goes the same way. An order that costs nothing can't be charged; the carrier is sometimes down, and asking again works; a digital order has nothing to ship. You want each step to say which of these happened, and leave what follows to the workflow.

## The code

The charge fails when there is nothing to charge, with a reason: a code and a message.

```example
name: ch10_outcomes
region: charge
```

The shipping skips a digital order, fails in a way that might work next time when the carrier is down, and succeeds otherwise.

```example
name: ch10_outcomes
region: ship
```

The workflow lets the shipping be tried once more after a failure that might work next time, with `retry_budget = 1` on its line. [Chapter 13](13-trying-again.md) is about this retry budget.

```example
name: ch10_outcomes
region: workflow
```

The shop runs three orders: a parcel, a digital order and an order that costs nothing. The carrier is down the first time it is called. The `report` function prints how each journey ended; [chapter 22](22-reading-the-result.md) shows how to read a result.

```example
name: ch10_outcomes
region: run
```

It prints:

```output
name: ch10_outcomes
```

## How it works

A step that runs to the end reports exactly one outcome: success, failure or skipped. The outcome says what happened, never what to do next. In Rust, `run` returns it as the `Ok` side of its `Result<Outcome, Error>`, built with the shorthands that `#[step]` gives the step: `Self::succeed()`, `Self::fail(reason)`, `Self::fail_retriably(reason)`, `Self::skip()` and `Self::skip_because(reason)`.

1. **Success: the step did its work.** Its contributions are committed, and the journey moves to the next step.
2. **Failure: the step couldn't do its work.** A failure carries a reason: a code, such as `nothing-to-charge`, and optionally a message for people and details for programs. A plain text such as `"carrier-down"` becomes a reason's code; `Reason::new("nothing-to-charge").with_message(...)` adds a message, and `.with_details(...)` adds details, which must be a value. A failure from `Self::fail` says that trying again wouldn't help, so the step fails for good, and the journey fails with the cause `failure` and the step's reason, as for the order that costs nothing.
3. **A failure that might work next time.** With `Self::fail_retriably`, the step marks the failure retriable: it believes trying again might help. That is only a suggestion. Whether another attempt happens depends on the step's retry budget in the workflow, and on its policies: objects the workflow attaches to the step, which can decide what happens after an outcome, as [chapter 15](15-reacting-to-what-a-step-did.md) shows. The shipping has a budget of one retry, so when the carrier was down, the executor, the part of Itinera that runs journeys, built a fresh shipping step and ran it again at once, and the second attempt succeeded. Had the carrier still been down, the shipping would have failed for good, and the journey with the cause `retries exhausted`.
4. **Skipped: the step decided not to do its work.** A digital order has nothing to ship, and the shipping step knows that from the order itself. A skipped step leaves the flow: nothing it contributed is kept, nothing reacts to it, and the journey moves to the next step as if the step weren't there. The digital order's journey succeeded, with nothing shipped. A skip may carry a reason too, through `Self::skip_because`; it appears in the events.
5. **The result says which failure it was.** The result's `status` is a `JourneyStatus`, and a failed journey holds a `Failure` whose variant is its cause: `Failure::Failure(reason)` for a failure that isn't retriable, as `report` matches for the order that costs nothing, and `Failure::RetriesExhausted(...)` when the retry budget is spent. Both enums are `#[non_exhaustive]`, since they may grow, so a `match` on them ends with a wildcard arm.

When to use each:

| Outcome | Shorthand | Use it when | In the shop |
|---|---|---|---|
| Success | `Self::succeed()` | the step did its work | the order is charged, or shipped |
| Failure | `Self::fail(reason)` | the step couldn't do its work, and trying again would give the same answer | there is nothing to charge |
| Retriable failure | `Self::fail_retriably(reason)` | the step couldn't do its work, and another attempt might succeed | the carrier is down |
| Skipped | `Self::skip()` or `Self::skip_because(reason)` | the business says this step has nothing to do for this journey | a digital order has nothing to ship |

## Remember

1. Every attempt of a step reports exactly one outcome: success, failure or skipped.
2. Mark a failure retriable only when trying again might help; whether it is tried again is the workflow's decision, never the step's.
3. Skip only for a reason of the business that the step itself knows, never to hide a failure.
4. Give every failure a reason whose code says what happened.
