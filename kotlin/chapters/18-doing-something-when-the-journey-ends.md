# Doing something when the journey ends

## What you want to do

Every customer should hear how their order went: that the parcel is on its way, with its tracking number, or, for a gift card, its code, or, when the order couldn't be completed, an apology. You want that message written once the journey has ended, whichever step it ended on and whether it succeeded or failed, without any step having to know it is the last one.

## The code

The message is written by a **workflow policy**: a class marked `@WorkflowPolicy(name = "notify")` that implements `ItineraWorkflowPolicy`, both from `org.itineradev.itinera.policy`. Its hook marked `@OnWorkflowSuccess` asks for the order, its tracking number and its gift card's code from the data bag, and contributes the customer's message under `message`. Its hook marked `@OnWorkflowFailure` asks only for the order, and contributes the apology under the same key. A workflow hook decides nothing, so neither returns anything.

```example
name: ch18_doing_something_when_the_journey_ends
region: policy
```

The workflow lists it in its own annotation, with `policies = [Notify::class]` in `@Workflow(...)`, rather than in a step's `@WithStep`:

```example
name: ch18_doing_something_when_the_journey_ends
region: workflow
```

The charge and the audit are [chapter 15](15-reacting-to-what-a-step-did.md)'s, the rescue step `RenewCard` is [chapter 14](14-repairing-then-trying-once-more.md)'s, the shipping, `Outage` and `HeavyParcels` are [chapter 17](17-giving-up-a-step-early.md)'s and the desk is [chapter 11](11-naming-a-journey.md)'s. The bank's provider is [chapter 12](12-trying-again.md)'s, the interface `Payments` it lends [chapter 14](14-repairing-then-trying-once-more.md)'s, and the carriers' provider [chapter 13](13-a-carrier-for-each-attempt.md)'s, with [chapter 12](12-trying-again.md)'s `Carrier`.

The log is [chapter 11](11-naming-a-journey.md)'s, with [chapter 12](12-trying-again.md)'s `detail`, the branches chapters [14](14-repairing-then-trying-once-more.md) to [16](16-skipping-a-step.md) added and [chapter 17](17-giving-up-a-step-early.md)'s `givenUp`. Its `when` now also names a workflow hook by its policy and its hook, since it is about no step, and names the key of a request that found nothing:

```example
name: ch18_doing_something_when_the_journey_ends
region: workflow_hook
```

The output now holds the message:

```example
name: ch18_doing_something_when_the_journey_ends
region: output
```

The shop reads back the message and prints it. It runs Ann's, Gil's, Jo's and Kim's orders again, then a new customer's: Lee's card expired in 2019, and the faked bank now has no replacement for a card that expired more than three years ago. It prints:

```output
name: ch18_doing_something_when_the_journey_ends
```

## How it works

1. **A workflow policy acts on the whole journey.** A step policy is attached to steps, and its hooks are about one step's outcome. A workflow policy is attached to the workflow, and its hooks are about the journey. **What a journey does at its end belongs to a workflow policy, never to the step that happens to run last.**
2. **`on workflow success` runs once, when the journey has succeeded.** It runs after the last step, once the journey's fate is decided, and before the event that says so: in every journey that succeeds, `hook_called notify: on workflow success` comes before `journey_succeeded`. Kim's journey ends on a skipped shipping, and Jo's needed a rescue, but each succeeds, so the hook runs for each.
3. **`on workflow failure` runs once, when the journey has failed.** Lee's charge fails on the expired card, and the rescue step fails too, with `not-replaced`, since the bank has no new card: the rescue step is given up, and the journey fails. `on workflow failure` runs after that decision and before `journey_failed`, and `on workflow success` never runs for Lee. Every journey that succeeds or fails runs exactly one of the two hooks, so each of these customers hears back.
4. **A workflow hook reads the data bag, and nothing about any step.** A workflow hook asks for data with `@FromWorkflow`, as the audit does in [chapter 15](15-reacting-to-what-a-step-did.md). A parameter of a nullable type, such as `String?`, makes a request optional, as in [chapter 17](17-giving-up-a-step-early.md): on success, Kim's order has no tracking number, and Ann's, Gil's and Jo's have no code, so each gets `null` for what its journey never contributed. A parameter whose type is not nullable is required. The executor records the absence as `optional_input_absent`, an `OptionalInputAbsent` naming the key in `event.key`, just before the hook is called.
5. **A workflow hook decides nothing.** By the time it runs, the journey's fate is decided, so the hook can only do something about it. That is why each hook returns nothing, and `ItineraWorkflowPolicy` has no `keepDefault()` to offer.
6. **A workflow hook's contributions are part of the result.** The message is committed when the hook returns, before the journey ends, so the shop reads it back with the rest of the output, Lee's apology included, though that journey failed.
7. **A workflow policy is built once for each journey.** One `Notify` serves the whole journey, made with its constructor, which takes nothing, before the journey starts, and is let go when it ends.

Each customer's message now exists, but only in the data bag, and only `main` prints it. Sending it is something else: today the shop sends it by email, and the workflow policy shouldn't need to know how.

## Outcomes

We have shown how a workflow policy does something once a journey has ended, whichever step ended it, and whether it succeeded or failed.

| The usual way | With Itinera |
|---|---|
| A message at the end of a process is often sent by whichever function happens to finish it, so every path that can end the process, failures included, has to remember to send it. | One of the workflow policy's two hooks runs once the journey has ended, whether it succeeded or failed, and no step knows about it. |
