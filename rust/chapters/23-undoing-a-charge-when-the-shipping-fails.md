# Undoing a charge when the shipping fails

## What you want to do

When the shop gives up an order's shipping, the customer has already paid: Ned's parcel of 90 kg is charged, then given up, as in [chapter 17](17-giving-up-a-step-early.md), and the shop keeps his money. You want the shop to give the charge back whenever the shipping is given up, whatever gave it up.

## The code

Giving money back is something the shop does for its policies, so it is a role, as telling a customer is in [chapter 19](19-letting-policies-act-on-the-outside-world.md):

```example
name: ch23_undoing_a_charge_when_the_shipping_fails
region: refunds
```

The workflow implements it. Here the bank is faked, and prints what it gives back:

```example
name: ch23_undoing_a_charge_when_the_shipping_fails
region: refunds_role
```

`heavy_parcels` now does more than turn heavy parcels away, so it becomes `failed_shipping`. Its `on step retry` still gives up a parcel over the limit, and refunds the charge first. A second hook, marked `#[on_step_failure]`, refunds the charge of a shipping that the executor gave up. It asks for a `StepFailureCause`, which says why the step was given up. Both hooks ask for the receipt and the amount from the data bag, the role, and a contributor, and both call `refund`, a method of the policy's own:

```example
name: ch23_undoing_a_charge_when_the_shipping_fails
region: failed_shipping
```

The workflow attaches it to the shipping, in place of `heavy_parcels`:

```example
name: ch23_undoing_a_charge_when_the_shipping_fails
region: workflow
```

The other steps, policies and reporters it names are as earlier chapters show them: the lookup, `already_completed` and the log in [chapter 22](22-finishing-a-journey-early.md), the pricing, the charge and the audit in [chapter 20](20-an-optional-input.md), `RenewCard` in [chapter 14](14-repairing-then-trying-once-more.md), the shipping and `outage` in [chapter 17](17-giving-up-a-step-early.md), `notify` in [chapter 19](19-letting-policies-act-on-the-outside-world.md) and the desk in [chapter 11](11-naming-a-journey.md).

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

1. **`on step failure` runs once the executor gives a step up by its own rule.** Oli's shipping ended in an error on both attempts, so it was given up with the cause `retries exhausted`. Then `on step failure` was called, and asked for that cause. It returns an `OnFailure`, as `on step retry` does; keeping the default, as here, lets the journey fail as it would have.
2. **A step given up by a hook never reaches `on step failure`.** Ned's shipping was given up by `on step retry`, with `FailWorkflow`, so the hook that gave it up has already decided how the journey ends, and no other hook of the shipping is called. That is why `before_retrying` refunds too. `on step retry` is called only when a retry is left: for Gil, it keeps the default, and his order is never refunded.
3. **A policy is a type of its own, and may hold code its hooks share.** `refund` has no hook mark, so the executor never calls it. Both hooks call it: `on step retry` when it gives the shipping up, and `on step failure` once the executor has given it up. The giving up and the undoing of the shipping live together in one policy, and the shipping, the charge and the workflow's declaration know nothing about refunds.
4. **A hook's contributions are committed whatever the step's outcome.** The shipping failed, but `refund` is in Ned's and Oli's data, because a hook's contributor commits as soon as the hook returns. Only a failed attempt's own contributions are thrown away, as [chapter 6](06-passing-data-on.md) showed.
5. **The undo sits with the step it undoes.** `notify`'s `on workflow failure` runs for every failed journey, so a refund there would first have to find out whether a charge went through. Lee's journey, for one, fails without any charge. Undoing a journey's work in a standard way isn't part of Itinera yet, so for now a policy like this one does it.

The shop now gives back what it can't deliver. But look at how `main` runs the orders: it makes a new executor for each one, and waits for each journey to end before it starts the next.

## Outcomes

We have shown how one step policy undoes a charge, however the shipping after it is given up.

| The usual way | With Itinera |
|---|---|
| Undoing work after a failure is often written in each place that can give up, and one of them is easily forgotten. | Whichever gave the shipping up, a hook or the executor, the hook that sees it calls the policy's own `refund`, so every shipping given up is refunded. |
| The code that undoes an order's payment often ends up inside the shipping code, which then has to know about payments. | The shipping only reports what happened, and the policy attached to it decides what to undo. |
