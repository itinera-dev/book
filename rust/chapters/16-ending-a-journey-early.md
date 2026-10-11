# Ending a journey early

## What you want to do

Some orders are done before the last step. A digital order has nothing to ship: once it's paid, its journey is over. And when the payment network is down, the charge can't be repaired or retried in any useful way: you want the journey to fail at once, with a reason the shop can act on.

## The code

The charge step charges the order. For a digital order, the payment also unlocks the download, which the step contributes under `download`. It knows nothing of shipping. When the payment network is down, the `?` after `take_payment(...)` lets the error escape the step: that's an abnormal termination.

```example
name: ch16_ending_early
region: charge
```

The first policy reads what the charge contributed, asking for `download` as an optional `#[from_step]` parameter. If there is a download, it returns `Self::finish_workflow()`, which is `FinishWorkflow`: the journey succeeds at once, and the ship step never runs.

```example
name: ch16_ending_early
region: downloads
```

The second policy defines `on step abnormal termination`, and receives the error that escaped the charge with `#[error] error: &Error`. In this shop, the only error the charge lets escape is the payment network's, so the policy doesn't look at what went wrong: it returns `Self::fail_workflow(...)`, which is `FailWorkflow`, with a reason of the shop's own, `payment-network-down`, and the error's message. A policy attached to a step that can fail in other ways would read the error first.

```example
name: ch16_ending_early
region: outage
```

The workflow attaches both policies to the charge.

```example
name: ch16_ending_early
region: workflow
```

Three orders go through: a parcel for Ann, a digital order for Ben, and a parcel for Cat, whose bank the payment network can't reach today. A small reporter prints the attempts, the hooks and how each journey ended.

```example
name: ch16_ending_early
region: run
```

It prints:

```output
name: ch16_ending_early
```

## How it works

1. **A hook may return a lifecycle.** A lifecycle tells the executor what to do next instead of the default. There are two: `FinishWorkflow` ends the journey at once as succeeded, and `FailWorkflow` ends it at once as failed, with a reason. Returning `Self::carry_on()` keeps the default.
2. **Not every hook may return every lifecycle, and Rust's types say which.** Only `on step success` may finish the journey, so only it returns `Option<OnSuccess>`, which holds `FinishWorkflow` or `FailWorkflow`, and `Self::finish_workflow()`, whose type is `Result<Option<OnSuccess>, Error>`, fits only there. `on step failure`, `on step retry` and `on step abnormal termination` may only fail it, so they return `Option<FailWorkflow>`. Workflow hooks, which run at the very end, return no lifecycle at all: their `Ok` holds `()`. Returning a lifecycle a hook may not return therefore doesn't compile.
3. **A failure carries a reason.** `Self::fail_workflow(...)` takes a code, such as `"payment-network-down"`, or a `Reason` built with `Reason::new` and `with_message`, as the `outage` policy does. The result carries it as `Failure::FailWorkflow(reason)`, which is what the program's `ending` function prints.
4. **Finishing early keeps what the step did.** For Ben's order, the charge succeeded and its contributions were committed before the hook was called, so the download is in the output. The ship step didn't run; its status stays `NotExecuted`. The decision, `journey_succeeded`, names the policy that took it.
5. **Failing early replaces the default.** Without the `outage` policy, the abnormal termination would end the charge anyway, since it isn't retried by default, and the journey would fail with the cause `abnormal termination` and the error. With it, the step is given up with the cause `FailWorkflow`, decided by the policy and its hook, `on step failure` isn't called, and the result carries the shop's reason.
6. **This is how a rescue is refused.** In the shop of [chapter 14](14-repairing-then-trying-once-more.md), the charge has a rescue step, which would otherwise run after this failure. A step given up by its `on step abnormal termination` hook is never rescued, so a payment network that is down ends the journey at once instead of asking the bank for a new card.
7. **A panic is not an abnormal termination.** Only the `Err` that `run` returns is. A panic is outside the model: no hook sees it, and it reaches whoever called the executor's `run`, as [chapter 11](11-when-something-goes-wrong-inside-a-step.md) explains.
8. **The decision follows from what the step reported.** The charge reports what happened: it took a payment, it unlocked a download, or an error escaped it. The policy turns that into what happens next. The charge never decides whether the order ships.
9. **A skip is another way to leave out a step.** The ship step could instead skip itself when it is given a digital order: that would be shipping's own rule. Finishing early is a flow decision, taken from what the charge contributed, so it lives in a policy.

## Remember

1. A step reports what happened; a hook may end the journey early from it, with `FinishWorkflow` or `FailWorkflow`.
2. Only `on step success` may return `FinishWorkflow`, and workflow hooks return no lifecycle; in Rust, anything else doesn't compile.
3. A journey finished early has succeeded: the steps after it stay `NotExecuted`, and what was committed stays.
4. `FailWorkflow` carries a reason, which the result carries too.
5. A step given up by its `on step abnormal termination` hook is neither retried nor rescued.
