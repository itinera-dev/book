# Doing something when the journey ends

## What you want to do

Every customer should hear how their order went: that the parcel is on its way, with its tracking number, or, for a gift card, its code, or, when the order couldn't be completed, an apology. You want that message written once the journey has ended, whichever step it ended on and whether it succeeded or failed, without any step having to know it is the last one.

## The code

The message is written by a **workflow policy**: a class extending `WorkflowPolicy`, marked `@workflowPolicy({ name: "notify" })`, both from `itinera/policy`. Its hook marked `@onWorkflowSuccess()` asks for the order, its tracking number and its gift card's code from the data bag, and contributes the customer's message under `message`. Its hook marked `@onWorkflowFailure()` asks only for the order, and contributes the apology under the same key. A workflow hook decides nothing, so each returns `void`.

```example
name: ch18_doing_something_when_the_journey_ends
region: policy
```

The workflow lists it in its own options, in `@workflow(...)`, with `policies: [Notify]`, rather than in a step's:

```example
name: ch18_doing_something_when_the_journey_ends
region: workflow
```

The charge and the audit are [chapter 15](15-reacting-to-what-a-step-did.md)'s, the rescue step `RenewCard` is [chapter 14](14-repairing-then-trying-once-more.md)'s, the shipping, `Outage` and `HeavyParcels` are [chapter 17](17-giving-up-a-step-early.md)'s and the desk is [chapter 11](11-naming-a-journey.md)'s. The providers are [chapter 14](14-repairing-then-trying-once-more.md)'s, with the tokens `PAYMENTS` and `CARRIER` of [chapter 7](07-giving-a-step-the-tools-it-needs.md).

The log is [chapter 17](17-giving-up-a-step-early.md)'s, and now also names a workflow hook by its policy and its hook, since it is about no step, and names the key of a request that found nothing:

```example
name: ch18_doing_something_when_the_journey_ends
region: workflow_hook
```

The output now holds the message:

```example
name: ch18_doing_something_when_the_journey_ends
region: output
```

The rest is [chapter 17](17-giving-up-a-step-early.md)'s shop, which now also reads back the message and prints it. It runs Ann's, Gil's, Jo's and Kim's orders again, then a new customer's: Lee's card expired in 2019, and the faked bank now has no replacement for a card that expired more than three years ago. It prints:

```output
name: ch18_doing_something_when_the_journey_ends
```

## How it works

1. **A workflow policy acts on the whole journey.** A step policy is attached to steps, and its hooks are about one step's outcome. A workflow policy is attached to the workflow, and its hooks are about the journey. **What a journey does at its end belongs to a workflow policy, never to the step that happens to run last.**
2. **`on workflow success` runs once, when the journey has succeeded.** It runs after the last step, once the journey's fate is decided, and before the event that says so: in every journey that succeeds, `hook_called notify: on workflow success` comes before `journey_succeeded`. Kim's journey ends on a skipped shipping, and Jo's needed a rescue, but each succeeds, so the hook runs for each.
3. **`on workflow failure` runs once, when the journey has failed.** Lee's charge fails on the expired card, and the rescue step fails too, with `not-replaced`, since the bank has no new card: the rescue step is given up, and the journey fails. `on workflow failure` runs after that decision and before `journey_failed`, and `on workflow success` never runs for Lee. Every journey that succeeds or fails runs exactly one of the two hooks, so each of these customers hears back.
4. **A workflow hook reads the data bag, and nothing about any step.** A workflow hook asks for data with `@fromWorkflow`, as the audit does in [chapter 15](15-reacting-to-what-a-step-did.md). Wrapping the type token in `optional(...)`, from `itinera/value`, as the output does in [chapter 6](06-passing-data-on.md), makes a request optional, and the hook gets an `Option`. On success, Kim's order has no tracking number, and Ann's, Gil's and Jo's have no code, so each gets `None` for what its journey never contributed, and `isSome()` says which it has, as in [chapter 17](17-giving-up-a-step-early.md). The executor records the absence as `optional_input_absent`, naming the key in `event.key`, just before the hook is called.
5. **A workflow hook decides nothing.** By the time it runs, the journey's fate is decided, so the hook can only do something about it. That is why each hook returns `void`, and the base class `WorkflowPolicy` has no `this.keepDefault()` to offer.
6. **A workflow hook's contributions are part of the result.** The message is committed when the hook returns, before the journey ends, so the shop reads it back with the rest of the output, Lee's apology included, though that journey failed.
7. **A workflow policy is built once for each journey.** One `Notify` serves the whole journey, made with a constructor that takes nothing before the journey starts, and is let go when it ends.

Each customer's message now exists, but only in the data bag, and only the program's own loop prints it. Sending it is something else: today the shop sends it by email, and the workflow policy shouldn't need to know how.

## Outcomes

We have shown how a workflow policy does something once a journey has ended, whichever step ended it, and whether it succeeded or failed.

| The usual way | With Itinera |
|---|---|
| A message at the end of a process is often sent by whichever function happens to finish it, so every path that can end the process, failures included, has to remember to send it. | One of the workflow policy's two hooks runs once the journey has ended, whether it succeeded or failed, and no step knows about it. |
