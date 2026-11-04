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

Then the shop gets two new step policies. The first, `heavyParcels`, has a hook marked `@OnStepRetry`, from `org.itineradev.itinera.policy`. It asks for the reason the shipping's attempt failed with, and gives the shipping up when that reason is `over-parcel-limit`, since the limit is the same on every attempt.

```example
name: ch17_giving_up_a_step_early
region: heavy_parcels
```

The second, `outage`, has a hook marked `@OnStepAbnormalTermination`, from the same package. It asks for nothing, and gives up the step it is attached to whenever an exception escapes it.

```example
name: ch17_giving_up_a_step_early
region: outage
```

The workflow attaches `Outage` to the charge, beside the audit, and `HeavyParcels` to the shipping, in the `policies` of each step's `@WithStep`:

```example
name: ch17_giving_up_a_step_early
region: workflow
```

The charge and the audit are [chapter 15](15-reacting-to-what-a-step-did.md)'s, `RenewCard` [chapter 14](14-repairing-then-trying-once-more.md)'s and the desk [chapter 11](11-naming-a-journey.md)'s. The bank's provider is [chapter 12](12-trying-again.md)'s, the interface `Payments` it lends [chapter 14](14-repairing-then-trying-once-more.md)'s, and the carriers' provider [chapter 13](13-a-carrier-for-each-attempt.md)'s.

The log is [chapter 11](11-naming-a-journey.md)'s, with [chapter 12](12-trying-again.md)'s `detail` and the branches chapters [14](14-repairing-then-trying-once-more.md) to [16](16-skipping-a-step.md) added. Its branch for `StepGivenUp` now calls a function of its own, which says which policy gave a step up, and why:

```example
name: ch17_giving_up_a_step_early
region: given_up
```

It runs the same five orders as [chapter 16](16-skipping-a-step.md), and Cat's again, whose 120 is more than the bank's network can take, as in [chapter 12](12-trying-again.md). It prints:

```output
name: ch17_giving_up_a_step_early
```

## How it works

1. **A hook can end the journey.** What a hook returns can hold a **lifecycle**: a decision that ends the journey there. `FailWorkflow` fails it, with a reason, as a step's failure has one. `failWorkflow(...)`, which `ItineraStepPolicy` gives the class, returns it, and `keepDefault()`, as in [chapter 15](15-reacting-to-what-a-step-did.md), leaves what happens next unchanged.
2. **`on step retry` runs before a retry.** Gil's parcel of 60 kg is within the limit, but the main carrier won't collect it, so the shipping fails with `too-heavy`, the hook keeps the default, and the backup carrier takes the parcel on attempt 2. Ned's parcel of 90 kg is over the limit, so the shipping fails with `over-parcel-limit`, and the hook fails the journey: the shipping is given up with the cause `FailWorkflow`, decided by `heavyParcels`. **The retry budget says how many retries a step may have; a policy can still decide that none of them can help.**
3. **The step says what happened, and the policy decides what to do about it.** The limit is a rule of the business, so it lives in the shipping; the hook only branches on the reason. It asks for the reason by its type, as the rescue step does in [chapter 14](14-repairing-then-trying-once-more.md), but as a `Reason?`, since an attempt that ended in an exception reports none.
4. **`on step abnormal termination` runs first after an exception.** It is called as soon as an exception escaping `run` ends an attempt, before any retry, rescue or giving up. Cat's charge ended in the bank's `NetworkDown`, [chapter 12](12-trying-again.md)'s, so `outage` failed the journey with the reason `payment-network-down`, and the charge was given up at once. Its rescue step never ran.
5. **A step given up by a hook says who decided.** `StepGivenUp`'s `cause` is a `GiveUpCause`, from `org.itineradev.itinera.event`, here a `GiveUpCause.FailWorkflow`, which carries the `reason` and, as `decidedBy`, the policy and the hook that returned it. `givenUp` matches it with `when`, and writes any other cause as [chapter 12](12-trying-again.md) does. Then `journey_failed` follows, naming the step. Neither step decided to give up: the shipping only reported that the parcel was over the limit, and the charge only let the bank's exception escape.
6. **Each hook may return only some lifecycles, and its type says which.** A method marked `@OnStepRetry` or `@OnStepAbnormalTermination` returns an `OnFailure`, from `org.itineradev.itinera.policy`, whose only values are what `keepDefault()` and `failWorkflow(...)` return, since failing the journey is the only thing either hook may decide; the compiler refuses anything else.

So no order spends a retry or a rescue that can't help it. But none of the shop's journeys tells the customer how it went: the lines that say whether a journey succeeded are printed by `main`, and no customer ever reads them.

## Outcomes

We have shown how a step policy gives a step up early, when no retry or rescue can help it.

| The usual way | With Itinera |
|---|---|
| Deciding that a retry is pointless usually means more conditions inside the retry loop, beside the call it retries. | A policy's `on step retry` hook decides it from the reason the step reported, attached to the step in the workflow's declaration. |
| What happens after an unexpected error is often decided wherever the error is caught. | `on step abnormal termination` decides it for the step, before any retry or rescue, and the event says which policy did. |
