# Reacting to what a step did

## What you want to do

The shop's accountants want an audit line for every charge that succeeded: what the order was charged, and the receipt. You want it written after every successful charge, without adding a word about auditing to the charge.

## The code

The audit is a **step policy**: a type deriving `Default`, with an `impl` block marked `#[step_policy(name = "audit")]`. Its method marked `#[on_step_success]` is a **hook**, which the executor calls after a step it is attached to succeeds. The hook asks for the step's name, for the order's price from the data bag, and for the receipt the step contributed, and contributes the audit line under `audit`. It returns `Self::keep_default()`, leaving what happens next as it was.

```example
name: ch15_reacting_to_what_a_step_did
region: policy
```

The workflow attaches the policy to the charge, with `policies = [Audit]` on its line:

```example
name: ch15_reacting_to_what_a_step_did
region: workflow
```

The rescue step `RenewCard` is [chapter 14](14-repairing-then-trying-once-more.md)'s, the shipping is [chapter 12](12-trying-again.md)'s and the desk is [chapter 11](11-naming-a-journey.md)'s.

The log is [chapter 14](14-repairing-then-trying-once-more.md)'s, and now also says which policy's hook was called, and after which step:

```example
name: ch15_reacting_to_what_a_step_did
region: hook_called
```

A new customer, Kim, buys a gift card, which the shop sends by email. The charge now asks whether the order is a gift card, and if it is, gives back the card's code under `code`:

```example
name: ch15_reacting_to_what_a_step_did
region: charge
```

The payment client it asks for is [chapter 14](14-repairing-then-trying-once-more.md)'s `Payments`.

The order's data now says whether it is a gift card, and the output holds the audit line:

```example
name: ch15_reacting_to_what_a_step_did
region: data
```

The shop runs Ann's, Gil's and Jo's orders, then Kim's, reads back the audit line, and prints it:

```example
name: ch15_reacting_to_what_a_step_did
region: run
```

It prints:

```output
name: ch15_reacting_to_what_a_step_did
```

## How it works

1. **A step policy holds what happens around a step.** It defines hooks, and the workflow attaches it to steps, so the charge never knows it is audited. **A step never knows which policies are attached to it; the workflow attaches them.** The policy names no step of its own, so it can be attached to any step of any workflow; a hook that needs the step's name asks for it, with a parameter of type `StepName`.
2. **The executor calls a hook after an outcome.** `#[on_step_success]` marks the hook called each time a step the policy is attached to succeeds, after its contributions are committed; the audit reads the receipt from what the charge contributed. The executor records the call as `hook_called`, naming the policy and the hook. Jo's charge fails on its first attempt, so the hook is called only after attempt 2, once the charge succeeded with the new card.
3. **A hook asks for what it needs, and gets only that.** Each parameter after `&self` is a request. `#[from_step]` asks for what the step contributed, by the parameter's name, here `receipt`; `#[from_workflow]` asks for data from the data bag, here `price`; a `&mut Contributor` asks for a contributor. A hook never gets the data bag itself.
4. **A hook's contributions are committed when it returns.** The audit line is committed right after the hook, as `contribution_committed audit`, and the shop reads it back with the journey's output.
5. **A hook decides nothing it isn't asked to.** Its `Ok` holds an `OnSuccess`, which could carry a decision about what happens next. `Self::keep_default()` is `Ok` of `OnSuccess::KeepDefault`: the hook keeps the default, what happens next without it, and after a success the next step runs. The audit only writes down what happened.
6. **A policy is built afresh for every attempt.** The executor builds it with `Default`, and nothing is kept from one call to the next, which is why a hook takes `&self`: everything it needs comes through what it asks for.

Now look at Kim's journey. Her charge succeeded, and gave back her gift card's code, so there is nothing left to do for her order. But the workflow still runs the shipping, and the main carrier books a parcel of 0 kg.

## Outcomes

We have shown how a step policy reacts to what a step did, without the step knowing.

| The usual way | With Itinera |
|---|---|
| Auditing a call is usually a line added after it, wherever it is made, so the code that charges also knows about the audit. | The audit is a policy attached to the charge in the workflow, and the charge has no line about it. |
| What runs after a call succeeds is often spread across the callers. | Every hook call is an event, naming the policy and the hook, after the outcome it reacts to. |
