# Giving up a step early

## What you want to do

Some failures can't be mended by trying again: Ned's parcel is heavier than any carrier of the shop collects, and when the bank's network fails during a charge, the rescue step, which mends only an expired card, can't help either. You want the shop to give such a step up at once, before the retry or the rescue the workflow would otherwise run.

## The code

The shipping first learns a rule of the shop's business: the shop ships no parcel over 80 kg, the most its backup carrier collects. It fails such an order with the reason `over-parcel-limit`, before it books anything:

```example
name: ch17_giving_up_a_step_early
region: ship
```

The interface `Carrier` it asks for is [chapter 12](12-trying-again.md)'s.

Then the shop gets two new step policies. The first, `heavyParcels`, has a hook marked `@onStepRetry()`. It asks for the reason the shipping's attempt failed with, with `@failureReason`, as the rescue step of [chapter 14](14-repairing-then-trying-once-more.md) does, and gives the shipping up when that reason is `over-parcel-limit`, since the limit is the same on every attempt.

```example
name: ch17_giving_up_a_step_early
region: heavy_parcels
```

The second, `outage`, has a hook marked `@onStepAbnormalTermination()`. It asks for nothing, and gives up the step it is attached to whenever an error escapes it.

```example
name: ch17_giving_up_a_step_early
region: outage
```

The workflow attaches `Outage` to the charge, beside the audit, and `HeavyParcels` to the shipping, in the `policies` of each step's options:

```example
name: ch17_giving_up_a_step_early
region: workflow
```

The charge and the audit are [chapter 15](15-reacting-to-what-a-step-did.md)'s, the rescue step `RenewCard` is [chapter 14](14-repairing-then-trying-once-more.md)'s and the desk is [chapter 11](11-naming-a-journey.md)'s.

The log is [chapter 16](16-skipping-a-step.md)'s, and now also says which policy gave a step up, and why:

```example
name: ch17_giving_up_a_step_early
region: given_up
```

The rest is [chapter 16](16-skipping-a-step.md)'s shop. It runs the same five orders, and Cat's again, whose 120 is more than the bank's network can take, as in [chapter 12](12-trying-again.md). It prints:

```output
name: ch17_giving_up_a_step_early
```

## How it works

1. **A hook can end the journey.** What a hook returns can hold a **lifecycle**: a decision that ends the journey there. `FailWorkflow` fails it, with a reason, as a step's failure has one. `this.failWorkflow(...)` returns it, given a code as `this.fail(...)` is, and `this.keepDefault()`, as in [chapter 15](15-reacting-to-what-a-step-did.md), leaves what happens next unchanged.
2. **`on step retry` runs before a retry.** When a step that failed still has a retry to spend, the executor calls it first. Gil's parcel of 60 kg is within the limit, but the main carrier won't collect it, so the shipping fails with `too-heavy`, the hook keeps the default, and the backup carrier takes the parcel on attempt 2. Ned's parcel of 90 kg is over the limit, so the shipping fails with `over-parcel-limit`, and the hook fails the journey: the shipping is given up with the cause `FailWorkflow`, decided by `heavyParcels`, and never meets the backup carrier. **The retry budget says how many retries a step may have; a policy can still decide that none of them can help.**
3. **The step says what happened, and the policy decides what to do about it.** The limit is a rule of the business, so it lives in the shipping, which reports it as a reason. The hook only branches on it. It asks for the reason with `{ optional: true }`, getting an `Option<Reason>`, the `Option` of [chapter 6](06-passing-data-on.md), because an attempt that ended in an error reported no reason: `reason.isSome()` says there is one, and `reason.value` is it.
4. **`on step abnormal termination` runs first after an error.** It is called as soon as an error escaping `run` ends an attempt, before the executor decides between a retry, a rescue and giving up. Cat's charge ended in the bank's error, so `outage` failed the journey with the reason `payment-network-down`, and the charge was given up at once. Its rescue step never ran.
5. **A step given up by a hook says who decided.** `step_given_up` carries the cause `FailWorkflow`, the policy and the hook that returned it, as `event.decidedBy`, and its reason, as `event.reason`; an event of another cause was decided by `"default"` and has no reason, so the log checks `event.cause` first. Then `journey_failed` follows, naming the step. Neither step decided to give up: the shipping only reported that the parcel was over the limit, and the charge only let the bank's error escape.
6. **Each hook may return only some lifecycles, and its type says which.** A method marked `@onStepRetry()` or `@onStepAbnormalTermination()` compiles only when it returns an `OnFailure`, from `itinera/policy`: what `this.keepDefault()` or `this.failWorkflow(...)` returns, since failing the journey is the only thing either hook may decide.

So no order spends a retry or a rescue that can't help it. But none of the shop's journeys tells the customer how it went: the lines that say whether a journey succeeded are printed by the program's loop, after each journey has ended, and no customer ever reads them.

## Outcomes

We have shown how a step policy gives a step up early, when no retry or rescue can help it.

| The usual way | With Itinera |
|---|---|
| Deciding that a retry is pointless usually means more conditions inside the retry loop, beside the call it retries. | A policy's `on step retry` hook decides it from the reason the step reported, attached to the step in the workflow's declaration. |
| What happens after an unexpected error is often decided wherever the error is caught. | `on step abnormal termination` decides it for the step, before any retry or rescue, and the event says which policy did. |
