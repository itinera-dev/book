# Reacting to what a step did

## What you want to do

The shop's accountants want a line for every charge: what it took when it succeeded and, when it failed, why. You want that written down after every charge, without adding a word about auditing to the charge step.

## The code

The audit is a step policy: a reusable object whose hooks the executor calls after a step's outcome. In Rust it's a type that derives `Default`, with an `impl` block marked `#[step_policy(name = "audit")]`. This one defines two hooks, each a method marked with the hook it is. `#[on_step_success]` reads what the step contributed under `charged`; `#[on_step_failure]` reads the reason the step gave, when it gave one: a step that ended in an abnormal termination gives none, and the audit then says so. Both write the audit line under `audit`, and both return `Self::carry_on()`, which leaves what happens next to the default.

```example
name: ch15_audit
region: policy
asynchronous: ch15_audit_async
```

In the asynchronous form, the hooks are `async fn`, which makes the whole policy asynchronous.

The workflow attaches the policy to the charge step, with `policies = [Audit]` on the step's line. The charge contributes what it took under `charged`, as a step contributes in [chapter 6](06-passing-data-on.md).

```example
name: ch15_audit
region: workflow
asynchronous: ch15_audit_async
```

An asynchronous policy can be attached only to the steps of an asynchronous workflow, one whose steps are asynchronous, as they are in the asynchronous form.

Two orders go through: one that can be charged, and one with nothing to charge.

```example
name: ch15_audit
region: run
asynchronous: ch15_audit_async
```

The asynchronous form runs the same journeys on `AsyncLocalExecutor`, awaiting each.

It prints:

```output
name: ch15_audit
```

## How it works

1. **A step policy holds flow decisions, and things to do around a step.** It defines hooks, and is attached to steps in the workflow. It names no step of its own, so the same policy can be attached to any step of any workflow; a hook that needs the name asks for it, as the audit does with `#[step_name] step: StepName`.
2. **The executor calls the hook that matches the outcome.** After a success, `on step success`. After a step is given up, `on step failure`. Before another attempt, `on step retry`, and after an abnormal termination, `on step abnormal termination`. In Rust these are the attributes `#[on_step_success]`, `#[on_step_failure]`, `#[on_step_retry]` and `#[on_step_abnormal_termination]`. A skipped step calls no hook at all: it has left the flow.
3. **A hook asks for what it needs, and gets only that.** Each parameter after `&self` says what it asks for. The audit asks for the step's name with `#[step_name]`, for what the step contributed under `charged` with `#[from_step]`, keyed by the parameter's name, for the failure's reason with `#[reason]`, and for a contributor with `#[contributor]`; the API documentation of `itinera` lists the others under `step_policy`. A hook can also ask for data from the workflow, read from the data bag, but never for the data bag itself.
4. **What a hook can't ask for doesn't compile.** A request a hook can't make, such as the reason in `on step success`, where nothing failed, is a compile error.
5. **A request is required or optional, by its type.** A parameter of type `Option<T>` is optional; any other type is required. The audit asks for the reason as `Option<Reason>`, since an abnormal termination has none. A required request with nothing to answer it would abort the journey.
6. **A hook's contributions are committed when it returns.** Unlike a step's, they don't depend on the step's outcome. That's why the second order's audit line is in the data bag, though its charge failed.
7. **Returning nothing means the default.** `Self::carry_on()` is `Ok(None)`: after a success, the next step runs; after a failure, the journey fails. A hook may instead return a lifecycle: a decision that replaces the default, such as failing the journey at once, as [chapter 16](16-ending-a-journey-early.md) shows. That's why the hooks return different types: `on step success` returns `Result<Option<OnSuccess>, Error>`, and the other three step hooks return `Result<Option<FailWorkflow>, Error>`.
8. **A hook that returns `Err` aborts the journey.** The journey is aborted with `hook failed`: a hook can't fail and carry on.
9. **A step policy is built afresh for every attempt.** The executor builds it with `Default`, unless its block has a function marked `#[factory]`, as [chapter 21](21-running-many-journeys.md) shows. Nothing is kept from one attempt to the next, and a hook never changes its policy, which is why it takes `&self`: everything it needs comes through what it asks for.
10. **Each hook is defined once per step.** Several policies can be attached to one step, but if two of them define the same hook, the workflow is refused before any journey with `hook defined twice`.
11. **The events record every hook.** Each call is a `HookCalled` event, naming the policy and the hook, and each contribution it committed names them as its source.

## Remember

1. A step never knows which policies are attached to it; the workflow attaches them.
2. A hook's contributions are committed when it returns, whatever the step's outcome.
3. A skipped step calls no hook.
4. Among the policies attached to one step, each hook is defined at most once.
5. A hook branches on what steps reported and contributed; business decisions stay in steps.
