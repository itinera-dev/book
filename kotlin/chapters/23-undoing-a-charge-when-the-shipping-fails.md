# Undoing a charge when the shipping fails

## What you want to do

When the shop gives up an order's shipping, the customer has already paid: Ned's parcel of 90 kg is charged, then given up, as in [chapter 17](17-giving-up-a-step-early.md), and the shop keeps his money. You want the shop to give the charge back whenever the shipping is given up, whatever gave it up.

## The code

Giving money back is something the shop does for its policies, so it is a role, marked `@Role`, as telling a customer is in [chapter 19](19-letting-policies-act-on-the-outside-world.md):

```example
name: ch23_undoing_a_charge_when_the_shipping_fails
region: refunds
```

`HeavyParcels` now does more than turn heavy parcels away, so it becomes `FailedShipping`, named `failedShipping`. Its `on step retry` still gives up a parcel over the limit, and refunds the charge first. A second hook, marked `@OnStepFailure`, from `org.itineradev.itinera.policy`, refunds the charge of a shipping that the executor gave up. Its parameter of type `FailureCause`, from `org.itineradev.itinera.journey`, asks for why the step was given up. Both hooks ask for the receipt and the amount from the data bag, the role, and a contributor, and both call `refund`, a method of the policy's own:

```example
name: ch23_undoing_a_charge_when_the_shipping_fails
region: failed_shipping
```

The workflow attaches it to the shipping, in place of `HeavyParcels`, and implements the role, with `Refunds` beside `Messages`. Here the bank is faked, and prints what it gives back:

```example
name: ch23_undoing_a_charge_when_the_shipping_fails
region: workflow
```

The other steps, policies and reporters it names are as earlier chapters show them: the lookup and `AlreadyCompleted` in [chapter 22](22-finishing-a-journey-early.md), the pricing, the charge and the audit in [chapter 20](20-an-optional-input.md), `RenewCard` in [chapter 14](14-repairing-then-trying-once-more.md), the shipping and `Outage` in [chapter 17](17-giving-up-a-step-early.md), `Notify` and the role `Messages` in [chapter 19](19-letting-policies-act-on-the-outside-world.md), the log in [chapter 22](22-finishing-a-journey-early.md), with the parts that page points to, and the desk in [chapter 11](11-naming-a-journey.md). Its providers lend [chapter 14](14-repairing-then-trying-once-more.md)'s `Payments`, [chapter 12](12-trying-again.md)'s `Carrier` and [chapter 22](22-finishing-a-journey-early.md)'s `OrderBook`, and its input adapter reads [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s `Parcel`.

`main` runs three orders: Gil's, whose parcel the backup carrier takes, Ned's, and the order of a new customer, Oli. The warehouse weighed Oli's parcel as 0 kg by mistake, and the faked carriers' booking systems fail on a parcel that weighs nothing. `main` also prints the refund the journey recorded:

```example
name: ch23_undoing_a_charge_when_the_shipping_fails
region: run
```

`NewOrder` and `Parcel` are as [chapter 21](21-deciding-where-a-steps-data-comes-from.md) shows them, and the output now holds the refund:

```example
name: ch23_undoing_a_charge_when_the_shipping_fails
region: output
```

It prints:

```output
name: ch23_undoing_a_charge_when_the_shipping_fails
```

## How it works

1. **`on step failure` runs once the executor gives a step up by its own rule.** Both carriers threw `BookingFailed` for Oli's parcel, so the shipping ended in an abnormal termination on both attempts, so it was given up with the cause `retries exhausted`. Then `on step failure` was called, and asked for that cause, a `FailureCause`, which prints as the specification writes it. It returns an `OnFailure`, as `on step retry` does; keeping the default, as here, lets the journey fail as it would have.
2. **A step given up by a hook never reaches `on step failure`.** Ned's shipping was given up by `on step retry`, with `FailWorkflow`, so the hook that gave it up has already decided how the journey ends, and no other hook of the shipping is called. That is why `beforeRetrying` refunds too. `on step retry` is called only when a retry is left: for Gil, it keeps the default, and his order is never refunded.
3. **A policy is a class of its own, and may hold code its hooks share.** `refund` is a private method with no hook annotation, so the executor never calls it. Both hooks call it: `on step retry` when it gives the shipping up, and `on step failure` once the executor has given it up. The giving up and the undoing of the shipping live together in one policy: the shipping and the charge know nothing about refunds, and the workflow only carries out the refund the policy asks for, through its role.
4. **A hook's contributions are committed whatever the step's outcome.** The shipping failed, but `refund` is in Ned's and Oli's data, because a hook's contributor commits as soon as the hook returns. Only a failed attempt's own contributions are thrown away, as [chapter 6](06-passing-data-on.md) showed.
5. **The undo sits with the step it undoes.** `Notify`'s `on workflow failure` runs for every failed journey, so a refund there would first have to find out whether a charge went through. Lee's journey, for one, fails without any charge, as [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s run shows.

The shop now gives back what it can't deliver. But look at how `main` runs the orders: it makes a new executor for each one, and waits for each journey to end before it starts the next.

## Outcomes

We have shown how one step policy undoes a charge, however the shipping after it is given up.

| The usual way | With Itinera |
|---|---|
| Undoing work after a failure is often written in each place that can give up, and one of them is easily forgotten. | Whichever gave the shipping up, a hook or the executor, the hook that sees it calls the policy's own `refund`, so every shipping given up is refunded. |
| The code that undoes an order's payment often ends up inside the shipping code, which then has to know about payments. | The shipping only reports what happened, and the policy attached to it decides what to undo. |
