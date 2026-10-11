# Ending a journey early

## What you want to do

Some orders are done before the last step. A digital order has nothing to ship: once it is paid, its journey is over. And when the payment network is down, the charge cannot be repaired or retried in any useful way: you want the journey to fail at once, with a reason the shop can act on.

## The code

The charge step charges the order. For a digital order, the payment also unlocks the download, which the step contributes under `download`. It knows nothing of shipping. When the payment network is down, the error it gets back escapes the step: that is an abnormal termination.

```example
name: ch16_ending_early
region: charge
```

The first policy reads what the charge contributed. If there is a download, it returns `FinishWorkflow`: the journey succeeds at once, and the ship step never runs.

```example
name: ch16_ending_early
region: downloads
```

The second policy defines `on step abnormal termination`. In this shop, the only error the charge lets escape is the payment network's, so the policy does not look at what went wrong: it returns `FailWorkflow` with a reason of the shop's own, `payment-network-down`, and the error's message. A policy attached to a step that can fail in other ways would read the error first.

```example
name: ch16_ending_early
region: outage
```

The workflow attaches both policies to the charge.

```example
name: ch16_ending_early
region: workflow
```

Three orders go through: a parcel for Ann, a digital order for Ben, and a parcel for Cat, whose bank the payment network cannot reach today. A small reporter prints the attempts, the hooks and how each journey ended.

```example
name: ch16_ending_early
region: run
```

It prints:

```output
name: ch16_ending_early
```

The type a hook returns says which lifecycles it may return. `on step success` returns `Option<OnSuccess>`, which holds `FinishWorkflow` or `FailWorkflow`; every other step hook returns `Option<FailWorkflow>`; workflow hooks return `()`. Returning a lifecycle a hook may not return therefore does not compile, and `Self::finish_workflow()` exists only for `on step success`. `Self::fail_workflow(...)` takes a code, or a `Reason` built with `Reason::new` and `with_message`.

The charge's `?` turns the payment network's error into the `Error` that `run` returns, and that `Err` is the abnormal termination. The hook receives the same error with `#[error] error: &Error`. A panic is not an abnormal termination: it is outside the model, no hook sees it, and it reaches whoever called `run`.

## How it works

1. **A hook may return a lifecycle.** A lifecycle tells the executor what to do next instead of the default. There are two: `FinishWorkflow` ends the journey at once as succeeded, and `FailWorkflow` ends it at once as failed, with a reason. Returning nothing keeps the default.
2. **Not every hook may return every lifecycle.** Only `on step success` may finish the journey. `on step success`, `on step failure`, `on step retry` and `on step abnormal termination` may fail it. Workflow hooks, which run at the very end, return no lifecycle.
3. **Finishing early keeps what the step did.** For Ben's order, the charge succeeded and its contributions were committed before the hook was called, so the download is in the output. The ship step did not run; its status stays `NotExecuted`. The decision, `journey_succeeded`, names the policy that took it.
4. **Failing early replaces the default.** Without the `outage` policy, the abnormal termination would end the charge anyway, since it is not retried by default, and the journey would fail with the cause `abnormal termination` and the error. With it, the step is given up with the cause `FailWorkflow`, decided by the policy and its hook, `on step failure` is not called, and the result carries the shop's reason.
5. **This is how a rescue is refused.** In the shop of chapter 14, the charge has a rescue step, which would otherwise run after this failure. A step given up by its `on step abnormal termination` hook is never rescued, so a payment network that is down ends the journey at once instead of asking the bank for a new card.
6. **The decision follows from what the step reported.** The charge reports what happened: it took a payment, it unlocked a download, or an error escaped it. The policy turns that into what happens next. The charge never decides whether the order ships.
7. **A skip is another way to leave out a step.** The ship step could instead skip itself when it is given a digital order: that would be shipping's own rule. Finishing early is a flow decision, taken from what the charge contributed, so it lives in a policy.

## Remember

1. A step reports what happened; a hook may end the journey early from it, with `FinishWorkflow` or `FailWorkflow`.
2. Only `on step success` may return `FinishWorkflow`, and workflow hooks return no lifecycle.
3. A journey finished early has succeeded: the steps after it stay `NotExecuted`, and what was committed stays.
4. `FailWorkflow` carries a reason, which the result carries too.
5. A step given up by its `on step abnormal termination` hook is neither retried nor rescued.
