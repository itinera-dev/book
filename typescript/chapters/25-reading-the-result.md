# Reading the result

## What you want to do

When an order doesn't go through, you want the code that runs the shop to read why from the journey's result. And you want it to tell a journey that failed, which the customer can be told about, from one that was stopped because the shop itself got something wrong.

## The code

The shop is [chapter 24](24-running-many-journeys.md)'s, with one more customer: Max, whose order arrives without a card, because the page that took it forgot to ask for one. His order's data leaves `card` out:

```example
name: ch25_reading_the_result
region: data
```

The loop now hands each journey's result to `followUp`, which looks at how the journey ended before it reads anything else. `JourneyResult`, the type of a result, is from `itinera/journey`, and `orderOutput`, the output it reads, is [chapter 24](24-running-many-journeys.md)'s:

```example
name: ch25_reading_the_result
region: follow_up
```

Two small functions put into words what explains a failure, and an abort. `Failure` and `Abort` are from `itinera/journey` too:

```example
name: ch25_reading_the_result
region: explain
```

It runs the six orders, and prints:

```output
name: ch25_reading_the_result
```

## How it works

1. **The status is one of three.** The result's `status` is a union, told apart by its `kind`, as an event is by its own: `succeeded`, `failed`, which also holds a `Failure`, and `aborted`, which holds an `Abort` instead. These are every way a journey ends. `result.isSucceeded()`, `result.isFailed()` and `result.isAborted()` say which it is, and also tell TypeScript, so after `isFailed()` the code can read `result.status.failure`, and after `isAborted()`, `result.status.abort`. What the journey gave back, the shop reads typed, with `result.output(...)`, as [chapter 6](06-passing-data-on.md) showed.
2. **A failure explains itself.** Lee's failure carries its cause, `failure`, and the reason the rescue step gave, `not-replaced`: a rescue step that fails is given up, as [chapter 18](18-doing-something-when-the-journey-ends.md) showed. A `Failure` holds its `cause`, and either the `reason` given or the error that ended the last attempt; the causes are the ones `step_given_up` names, as [chapter 12](12-trying-again.md) and [chapter 17](17-giving-up-a-step-early.md) showed. A failure with the cause `failure` always holds a reason, so once `why` has compared `failure.cause` with it, TypeScript lets it read `failure.reason`.
3. **The result says how a journey ended, not where.** It names no step. The desk said that Lee's order failed at `charge_rescue`, because it read the `journey_failed` event. Which step failed, and how many attempts each step made, are in the events.
4. **An abort means something that must not happen did.** Max's charge asks for a card, and the data bag has none, so the charge can't be built. The executor doesn't fail the journey: it aborts it, with the reason `required data missing`. A failure is an answer of the business, which the shop can give the customer. An abort is a fault in the shop's own code or data, to be fixed. An `Abort` holds its `reason`, and what that reason carries: once `whatWentWrong` has compared `abort.reason` with `required data missing`, TypeScript lets it read `abort.missing`, whose `kind` is `key` when a key was missing, and whose `key` names it.
5. **An aborted journey has no output.** Max's pricing succeeded, and its amount was committed, but the data bag of a stopped journey may stand halfway through a change, so the result doesn't offer it. `result.output(...)` throws an `OutputError`, from `itinera/journey`, for an aborted journey, which is why `followUp` reads the output only once it has dealt with an abort.
6. **An aborted journey runs no workflow hook.** Lee was told the order could not be completed, and Max was told nothing, since the hooks of a workflow policy run only for a journey that succeeded or failed. No `journey_failed` was recorded either, so the desk is silent: Max's log ends with `journey_aborted`. **An aborted journey is never resumed:** fix the cause, then run the order again with a new instance.

Every order now ends with an answer the shop can act on. But the shop's real bank and carriers answer over the network, and while a journey waits for them, the shop takes no other order forward.

## Outcomes

We have shown how the shop reads from the result how each journey ended and what explains it.

| The usual way | With Itinera |
|---|---|
| Why a request failed is often a message in a log, or an error type the caller has to guess at. | A failed journey's result carries its cause and the reason the step gave. |
| A bug in the shop's own code and a refusal of the business often end the same way, as an error. | A journey that failed and a journey that was aborted are different statuses, and an aborted journey has no output. |
