# Reading the result

## What you want to do

When an order doesn't go through, you want the code that runs the shop to read why from the journey's result. And you want it to tell a journey that failed, which the customer can be told about, from one that was stopped because the shop itself got something wrong.

## The code

The shop is [chapter 22](22-running-many-journeys.md)'s, with one more customer: Max, whose order arrives without a card, because the page that took it forgot to ask for one. The order's card is now an `Option`, and Max's is `None`.

```example
name: ch23_reading_the_result
region: data
```

`main` hands each journey's result to `follow_up`, which looks at how the journey ended before it reads anything else, with a `match` on its status:

```example
name: ch23_reading_the_result
region: follow_up
```

Two small functions put into words what explains a failure, and an abort:

```example
name: ch23_reading_the_result
region: explain
```

It runs the six orders, and prints:

```output
name: ch23_reading_the_result
```

## How it works

1. **The status is an enum.** The result's `status` is a `JourneyStatus`, with one variant for each way a journey ends. `Succeeded` holds the data bag, `Failed` holds the data bag and a `Failure`, and `Aborted` holds an `Abort`. The `match` is how `follow_up` tells them apart, and each arm works with what its variant holds. These enums are `#[non_exhaustive]`, so a `match` on them always needs a `_` arm; in `why` and `what_went_wrong` it names the cause or reason alone.
2. **A failure explains itself.** Lee's failure carries its cause, `failure`, and the reason the rescue step gave, `not-replaced`: a rescue step that fails is given up, as [chapter 18](18-doing-something-when-the-journey-ends.md) showed. `Failure` has one variant for each cause, holding the reason given, or the error that ended the last attempt, and its `cause()` names the cause. The causes are the ones `step_given_up` names, as [chapter 12](12-trying-again.md) showed.
3. **The result says how a journey ended, not where.** It names no step. The desk said that Lee's order failed at `charge_rescue`, because it read the `journey_failed` event. Which step failed, and how many attempts each step made, are in the events.
4. **An abort means something that must not happen did.** Max's charge asks for a card, and the data bag has none, so the charge can't be built. The executor doesn't fail the journey: it aborts it, with the reason `required data missing`, naming the key. A failure is an answer of the business, which the shop can give the customer. An abort is a fault in the shop's own code or data, to be fixed. `Abort` has one variant for each reason, and `RequiredDataMissing` holds what was missing, which is how `what_went_wrong` finds the key.
5. **An aborted journey has no output.** Max's pricing succeeded, and its amount was committed, but the data bag of a stopped journey may stand halfway through a change, so the result doesn't offer it. `result.output()` returns an `Err` for an aborted journey, which is why only the other two arms read it.
6. **An aborted journey runs no workflow hook.** Lee was told the order could not be completed, and Max was told nothing, since the hooks of a workflow policy run only for a journey that succeeded or failed. No `journey_failed` was recorded either, so the desk is silent: Max's log ends with `journey_aborted`. **An aborted journey is never resumed:** fix the cause, then run the order again with a new instance.

Every order now ends with an answer the shop can act on. But the only way to know that the shop still gives those answers after a change is to run every order again, against the bank and the carriers, and read the log.

## Outcomes

We have shown how the shop reads from the result how each journey ended and what explains it.

| The usual way | With Itinera |
|---|---|
| Why a request failed is often a message in a log, or an error type the caller has to guess at. | A failed journey's result carries its cause and the reason the step gave. |
| A bug in the shop's own code and a refusal of the business often end the same way, as an error. | A journey that failed and a journey that was aborted are different statuses, and an aborted journey has no output. |
