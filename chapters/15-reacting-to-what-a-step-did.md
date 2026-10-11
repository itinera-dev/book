# Reacting to what a step did

## What you want to do

The shop's accountants want a line for every charge: what it took when it succeeded and, when it failed, why. You want that written down after every charge, without adding a word about auditing to the charge step.

## The code

The audit is a step policy: a reusable object whose hooks the executor calls after a step's outcome. This one defines two hooks. `on step success` reads what the step contributed under `charged`; `on step failure` reads the reason the step gave, when it gave one: a step that ended in an abnormal termination gives none, and the audit then says so. Both write the audit line under `audit`, and both return nothing, which leaves what happens next to the default.

```example
name: ch15_audit
region: policy
asynchronous: ch15_audit_async
```

The workflow attaches the policy to the charge step. The charge itself is the same as in chapter 6.

```example
name: ch15_audit
region: workflow
asynchronous: ch15_audit_async
```

Two orders go through: one that can be charged, and one with nothing to charge.

```example
name: ch15_audit
region: run
asynchronous: ch15_audit_async
```

It prints:

```output
name: ch15_audit
```

In the asynchronous form, the hooks are asynchronous too, and the policy is attached to an asynchronous workflow's steps.

```note
name: ch15_audit
```

## How it works

1. **A step policy holds flow decisions, and things to do around a step.** It defines hooks, and is attached to steps in the workflow. It names no step of its own, so the same policy can be attached to any step of any workflow; a hook that needs the name asks for it, as the audit does.
2. **The executor calls the hook that matches the outcome.** After a success, `on step success`. After a step is given up, `on step failure`. Before another attempt, `on step retry`, and after an abnormal termination, `on step abnormal termination`. A skipped step calls no hook at all: it has left the flow.
3. **A hook asks for what it needs, and gets only that.** The audit asks for the step's name, for what the step contributed under `charged`, and for the failure's reason, as optional, since an abnormal termination has none. A required request with nothing to answer it would abort the journey. A hook can also ask for data from the workflow, read from the data bag, but never for the data bag itself.
4. **A hook's contributions are committed when it returns.** Unlike a step's, they do not depend on the step's outcome. That is why the second order's audit line is in the data bag, though its charge failed.
5. **Returning nothing means the default.** After a success, the next step runs; after a failure, the journey fails. A hook may instead return a lifecycle: a decision that replaces the default, such as failing the journey at once, as chapter 16 shows.
6. **A step policy is built afresh for every attempt.** Nothing is kept from one attempt to the next, and a hook never changes its policy: everything it needs comes through what it asks for.
7. **Each hook is defined once per step.** Several policies can be attached to one step, but if two of them define the same hook, the workflow is refused before any journey with `hook defined twice`.
8. **The events record every hook.** Each call is a `hook_called` event, naming the policy and the hook, and each contribution it committed names them as its source.

## Remember

1. A step never knows which policies are attached to it; the workflow attaches them.
2. A hook's contributions are committed when it returns, whatever the step's outcome.
3. A skipped step calls no hook.
4. Among the policies attached to one step, each hook is defined at most once.
5. A hook branches on what steps reported and contributed; business decisions stay in steps.
