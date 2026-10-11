# Doing something when the journey ends

## What you want to do

Every customer should hear how their order went: that it is on its way, or that it could not be taken. You want that message written once the journey is over, whichever step it ended on, without any step having to know it is the last one.

## The code

The message comes from a workflow policy: a policy attached to the whole workflow, whose hooks run when the journey ends. This one, `notify`, defines both workflow hooks. `on workflow success` reads the order and its tracking number from the journey's data; `on workflow failure` reads only the order, since a failed order has nothing to track. Each writes the message under `message`.

```example
name: ch17_notify
region: policy
```

The workflow lists the policy among its own, not on a step. The steps are the charge and the ship steps of the earlier chapters.

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

```note
name: ch17_notify
```

## How it works

1. **Workflow hooks run once, at the end.** `on workflow success` runs when the journey succeeds, including when a hook finished it early, and `on workflow failure` when it fails, including when a hook failed it. They run after the last step, before the result is produced.
2. **They decide nothing.** The journey's fate is settled when they run, so a workflow hook returns no lifecycle. It does something about the end: it writes a message, records it, or tells someone.
3. **A workflow hook reads data from the workflow.** It asks for keys of the data bag, here `order` and `tracking`. It cannot ask for a step's reason or error: those belong to step hooks, which act on one step. What it needs to know about a failure, a step or a step hook contributes first.
4. **Its contributions are part of the output.** They are committed when the hook returns, before the result is produced, so the message is in the data bag of the result, whether the journey succeeded or failed.
5. **A workflow policy is built once per journey.** One instance serves the whole journey and is discarded at its end. If it cannot be built, `run` refuses the journey before it starts.
6. **Nothing runs after an abort.** An aborted journey has no business outcome: neither workflow hook runs, and no message is written. Each hook can be defined only once per workflow, across all its policies.

## Remember

1. A workflow policy is attached to the workflow; a step policy to steps.
2. Workflow hooks run once, after the journey's fate is decided, and return no lifecycle.
3. Neither workflow hook runs when a journey is aborted.
4. A workflow hook reads data from the data bag, never a step's reason or error.
5. A workflow hook's contributions are part of the journey's output.
