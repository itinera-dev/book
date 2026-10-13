# Doing something when the journey ends

## What you want to do

Every customer should hear how their order went: that the parcel is on its way, with its tracking number, or, for a gift card, its code. You want that message written once the journey has succeeded, whichever step it ended on, without any step having to know it is the last one.

## The code

The message is written by a **workflow policy**: a type deriving `Default`, with an `impl` block marked `#[workflow_policy(name = "notify")]`. Its hook, marked `#[on_workflow_success]`, asks for the order, its tracking number and its gift card's code from the data bag, and contributes the customer's message under `message`. A workflow hook decides nothing, so it returns `Ok(())`.

```example
name: ch17_doing_something_when_the_journey_ends
region: policy
```

The workflow lists it on its own line, `#[workflow(...)]`, with `policies = [Notify]`, rather than on a step's:

```example
name: ch17_doing_something_when_the_journey_ends
region: workflow
```

The log names a workflow hook by its policy and its hook, since it is about no step, and names the key of a request that found nothing:

```example
name: ch17_doing_something_when_the_journey_ends
region: workflow_hook
```

The rest is [chapter 16](16-skipping-a-step.md)'s shop, which now also reads back the message and prints it. It runs the same four orders, and prints:

```output
name: ch17_doing_something_when_the_journey_ends
```

## How it works

1. **A workflow policy acts on the whole journey.** A step policy is attached to steps, and its hooks are about one step's outcome. A workflow policy is attached to the workflow, and its hooks are about the journey. **What a journey does at its end belongs to a workflow policy, never to the step that happens to run last.**
2. **`on workflow success` runs once, when the journey has succeeded.** It runs after the last step, once the journey's fate is decided, and before the event that says so: in every journey, `hook_called notify: on workflow success` comes before `journey_succeeded`. Kim's journey ends on a skipped shipping, and Jo's needed a rescue, but each succeeds, so the hook runs for each.
3. **It reads the data bag, and nothing about any step.** A workflow hook asks for data with `#[from_workflow]`, by the parameter's name. `Option<String>` makes a request optional: Kim's order has no tracking number, and the others have no code, so each gets `None` for what its journey never contributed, and the executor records that as `optional_input_absent`, naming the key, just before the hook is called.
4. **It decides nothing.** By the time it runs, the journey has succeeded, so the hook can only do something about it. That is why its `Ok` holds `()`.
5. **Its contributions are part of the result.** The message is committed when the hook returns, before the journey ends, so the shop reads it back with the rest of the output.
6. **A workflow policy is built once for each journey.** One `Notify` serves the whole journey, made with `Default` before it starts, and is dropped when it ends.

Each customer's message now exists, but only in the data bag, and only `main` prints it. Sending it is something else: today the shop sends it by email, and the workflow policy shouldn't need to know how.

## Outcomes

We have shown how a workflow policy does something once a journey has succeeded, whichever step ended it.

| The usual way | With Itinera |
|---|---|
| A message at the end of a process is often sent by whichever function happens to finish it, so every path that can end the process has to remember to send it. | The workflow policy's hook runs once the journey has succeeded, however it got there, and no step knows about it. |
