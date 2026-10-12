# Doing something when the journey ends

## What you want to do

Every customer should hear how their order went: that it's on its way, or that it couldn't be taken. You want that message written once the journey is over, whichever step it ended on, without any step having to know it's the last one.

## The code

The message comes from a workflow policy: a policy attached to the whole workflow, whose hooks run when the journey ends. In Rust it's a type whose `impl` block carries `#[workflow_policy(name = "...")]`. This one, `notify`, defines both workflow hooks, as methods marked `#[on_workflow_success]` and `#[on_workflow_failure]`. The success hook reads the order from the journey's data with `#[from_workflow]`, and its tracking number when there is one, as an `Option<String>`: a digital order finished early, as in [chapter 16](16-ending-a-journey-early.md), has none. The failure hook reads only the order, since a failed order has nothing to track. Each writes the message under `message`, and returns `Ok(())`.

```example
name: ch17_notify
region: policy
```

The workflow lists the policy among its own, with `policies = [Notify]` in `#[workflow(...)]`, not on a step's line. The steps are the charge and the ship steps of the earlier chapters.

```example
name: ch17_notify
region: workflow
```

Two orders go through: one that is charged and shipped, and one with nothing to charge.

```example
name: ch17_notify
region: run
```

It prints:

```output
name: ch17_notify
```

## How it works

1. **Workflow hooks run once, at the end.** `on workflow success` runs when the journey succeeds, including when a hook finished it early, and `on workflow failure` when it fails, including when a hook failed it. They run once the journey's fate is decided, before the result is produced and before the event that says how the journey ended.
2. **They decide nothing.** The journey's fate is settled when they run, so a workflow hook returns no lifecycle: its type is `Result<(), Error>`. It does something about the end: it writes a message, records it, or tells someone. An `Err` aborts the journey with `hook failed`.
3. **A workflow hook reads data from the workflow.** It asks for keys of the data bag with `#[from_workflow]`, here `order` and `tracking`. It may also ask for a contributor with `#[contributor]`, as `notify` does to keep its message; the API documentation of `itinera` lists the others under `workflow_policy`. It can't ask for a step's reason or error: those belong to step hooks, which act on one step, so `#[reason]` or `#[from_step]` in a workflow hook doesn't compile. What it needs to know about a failure, a step hook contributes first, since a failed attempt's own contributions never reach the data bag.
4. **Its contributions are part of the output.** They are committed when the hook returns, before the result is produced, so the message is in the data bag of the result, whether the journey succeeded or failed.
5. **A workflow policy is built once per journey.** One instance serves the whole journey and is dropped at its end. It is built with `Default`, as `Notify` is, or with a function of its block marked `#[factory]`. If it can't be built, the executor's `run` refuses the journey before it starts, returning a `Refusal` in its `Err`.
6. **Nothing runs after an abort.** An aborted journey has no business outcome: neither workflow hook runs, and no message is written.
7. **Each workflow hook is defined once per workflow.** Several workflow policies can be attached to one workflow, but if two of them define the same hook, the workflow is refused before any journey with `hook defined twice`.

## Remember

1. A workflow policy is attached to the workflow; a step policy to steps.
2. Workflow hooks run once, after the journey's fate is decided, and return no lifecycle.
3. Neither workflow hook runs when a journey is aborted.
4. A workflow hook reads data from the data bag, never a step's reason or error.
5. A workflow hook's contributions are part of the journey's output.
