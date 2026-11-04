# Reading the result

## What you want to do

When an order doesn't go through, you want the code that runs the shop to read why from the journey's result. And you want it to tell a journey that failed, which the customer can be told about, from one that was stopped because the shop itself got something wrong.

## The code

The shop is [chapter 24](24-running-many-journeys.md)'s, whose workflow, `Orders`, [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) shows, with one more customer: Max, whose order arrives without a card, because the page that took it forgot to ask for one. The order's card is now a `String?` that is `null` unless the order gives one, and Max's gives none, so his initial data has no `card`, as a `null` property adds no key in [chapter 20](20-an-optional-input.md):

```example
name: ch25_reading_the_result
region: data
```

Max's order is the sixth that `main` runs:

```example
name: ch25_reading_the_result
region: max
```

`Parcel` is [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s, and `Order`, the output that `followUp` reads, [chapter 24](24-running-many-journeys.md)'s.

`main` runs the six orders with one executor, as [chapter 24](24-running-many-journeys.md)'s does, and hands each journey's result to `followUp`:

```example
name: ch25_reading_the_result
region: run
```

`followUp` looks at how the journey ended before it reads anything else, with a `when` on its status:

```example
name: ch25_reading_the_result
region: follow_up
```

Two small functions put into words what explains a failure, and an abort:

```example
name: ch25_reading_the_result
region: explain
```

It prints:

```output
name: ch25_reading_the_result
```

## How it works

1. **The status is a sealed interface.** The result's `status` is a `JourneyStatus`, from `org.itineradev.itinera.journey`, with one class for each way a journey ends: `Succeeded`, `Failed`, which holds a `Failure`, and `Aborted`, which holds an `Abort`, both from the same package. Those are every way, so the `when` in `followUp` needs no `else`, and in each branch Kotlin knows which class the status is, so `status.failure` and `status.abort` can be read. What the journey gave back, the shop reads typed, with `result.output<Order>()`, as [chapter 6](06-passing-data-on.md) showed.
2. **A failure explains itself.** Lee's failure carries its cause, `failure`, and the reason the rescue step gave, `not-replaced`: a rescue step that fails is given up, as [chapter 18](18-doing-something-when-the-journey-ends.md) showed. A `Failure` holds its `cause`, a `FailureCause`, and the `reason` given, when there was one; the causes are the ones `step_given_up` names, as [chapter 12](12-trying-again.md) showed.
3. **A failure says why a journey ended, not where.** A `Failure` names no step. The desk said that Lee's order failed at `charge_rescue`, because it read the `journey_failed` event. Which step failed, and how many attempts each step made, are in the events.
4. **An abort means something that must not happen did.** Max's charge asks for a card, and the data bag has none, so the charge can't be built. The executor doesn't fail the journey: it aborts it, with the reason `required data missing`, naming the key. A failure is an answer of the business, which the shop can give the customer. An abort is a fault in the shop's own code or data, to be fixed. An `Abort` holds its `reason`, and its `details`, a class for each reason: for this one, `RequiredDataMissing`, from `org.itineradev.itinera.event`, whose `missing` is a `MissingData.Key` naming the key, which is how `whatWentWrong` finds it.
5. **An aborted journey has no output.** Max's pricing succeeded, and its amount was committed, but the data bag of a stopped journey may stand halfway through a change, so the result doesn't offer it. `result.output<Order>()` throws an `OutputException` for an aborted journey, which is why only the other two branches read it.
6. **An aborted journey runs no workflow hook.** Lee was told the order could not be completed, and Max was told nothing, since the hooks of a workflow policy run only for a journey that succeeded or failed. No `journey_failed` was recorded either, so the desk is silent: Max's log ends with `journey_aborted`. **An aborted journey is never resumed:** fix the cause, then run the order again with a new instance.

Every order now ends with an answer the shop can act on. But the shop's real bank and carriers answer over the network, and while a step waits for them, its journey holds the program's thread.

## Outcomes

We have shown how the shop reads from the result how each journey ended and what explains it.

| The usual way | With Itinera |
|---|---|
| Why a request failed is often a message in a log, or an error type the caller has to guess at. | A failed journey's result carries its cause and the reason the step gave. |
| When a request stops on a bug, its caller often still reads whatever state it left behind, as if it were a result. | An aborted journey is a status of its own, apart from a failure, and its result offers no output, since its data bag may stand halfway through a change. |
