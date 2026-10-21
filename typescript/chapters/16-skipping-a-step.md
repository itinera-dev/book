# Skipping a step

## What you want to do

Kim's gift card is sent by email, so her order has nothing to ship. You want the shipping to know that, and leave her order alone, rather than book a parcel of 0 kg.

## The code

The shipping now also asks whether the order is a gift card. When it is, it does nothing, and says so with `this.skipBecause("nothing-to-ship")`, before it books anything.

```example
name: ch16_skipping_a_step
region: ship
```

The interface `Carrier` it asks for is [chapter 12](12-trying-again.md)'s.

The log is [chapter 15](15-reacting-to-what-a-step-did.md)'s, and now also names the reason a step gave for skipping, and the step whose contributions were thrown away:

```example
name: ch16_skipping_a_step
region: skipped
```

The rest of the shop is [chapter 15](15-reacting-to-what-a-step-did.md)'s. It runs the same four orders, and one from a new customer, Ned, whose parcel weighs 90 kg. It prints:

```output
name: ch16_skipping_a_step
```

## How it works

1. **A skip is the third outcome.** A step that runs to the end says how it went in one of three ways: it succeeded, it failed, or it **skipped**, meaning it decided it had nothing to do for this journey. `this.skip()` says so, and `this.skipBecause(...)` adds a reason, as a failure has, which the event `step_skipped` carries as `event.reason`. `event.reason` is optional in the event's type, so the log checks that it is there.
2. **Skipping is the step's own decision.** Only the shipping knows that a gift card has nothing to ship, so it reads `giftCard` and skips. Ann's, Gil's, Jo's and Ned's orders are not gift cards, so their shipping runs as before. **A step skips only for a reason of the business it knows itself, never to hide a failure.**
3. **A skipped step leaves the flow.** Nothing it contributed is kept: the executor records `contributions_discarded`, even though the shipping skipped before contributing anything, and the journey moves on as if the step had not been there. The shipping was the last step, so Kim's journey succeeded, with her receipt and no tracking number.
4. **A skip is not a failure, and is never retried.** The shipping has a retry budget of 1, for failures that might be mended, and Kim's shipping never spends it: a step that skipped did what it meant to do, so the executor never tries it again.
5. **Its services were still provided.** The shipping asks for a carrier as a parameter of `run`, so the executor lends it one before `run` is called, as `service_provided` shows. The shipping never uses it.

So a gift card no longer gets a parcel booked. But look at Ned's order. The main carrier won't collect a parcel of 90 kg, so the shipping is tried again with the backup carrier, which collects parcels only up to 80 kg. That retry could never help, and the shop spends it all the same.

## Outcomes

We have shown how a step skips itself when it has nothing to do.

| The usual way | With Itinera |
|---|---|
| A function with nothing to do often returns early with a success or a special value, so callers cannot tell "done" from "nothing to do". | A skip is an outcome of its own, with a reason, recorded as an event, and the journey moves on. |
