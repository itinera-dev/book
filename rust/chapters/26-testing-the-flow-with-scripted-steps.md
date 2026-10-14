# Testing the flow with scripted steps

## What you want to do

You want to check the shop's flow: that a declined card is rescued, that a parcel over the limit is given up after its first attempt, and that a charge ending in an error is never rescued. Each check should say how the steps end, without setting up a bank or a parcel that makes them end that way.

## The code

The program is [chapter 23](23-reading-the-result.md)'s shop, with checks in place of the orders. Each check makes an instance of the shop's workflow for one of Ann's orders:

```example
name: ch26_testing_the_flow_with_scripted_steps
region: order
```

The first check runs the instance with a script. `scripted()`, a method of every workflow instance that comes with the `testing` feature, starts it. `step` names a step, and what follows says what its attempts do in place of running it: what they contribute, and how they end. `on_attempt` moves on to another attempt of the same step, `and_step` to another step, and `run` runs the journey:

```example
name: ch26_testing_the_flow_with_scripted_steps
region: rescued
```

The second runs one script twice, with the shipping's first attempt failing for a different reason each time:

```example
name: ch26_testing_the_flow_with_scripted_steps
region: given_up
```

The third ends the charge's first attempt in an error, as the bank's would:

```example
name: ch26_testing_the_flow_with_scripted_steps
region: outage
```

The shop's log, its desk and its messages still print, since the workflow is the shop's own. In a test, `cargo test` hides what a passing test prints. It prints:

```output
name: ch26_testing_the_flow_with_scripted_steps
```

## How it works

1. **The script replaces only a step's `run`.** Every step is still built for real, with its inputs and its services: the log shows the carriers provided for each attempt of the shipping, and the adapter supplying its weight. A step the script doesn't name runs for real, as the pricing does here. Everything else is the executor's, as in any journey: the hooks, the policies, the decisions, the events and the result.
2. **A script names steps by their names in the workflow.** A rescue step is named by its own name, `charge_rescue`. `will_fail`, `will_terminate_abnormally` and `will_succeed` end an attempt, and `will_contribute` adds a contribution before it ends, which the executor commits only if the attempt succeeds. An attempt `on_attempt` doesn't name is attempt 1.
3. **The script must cover what can happen.** `run` checks it before the journey starts, and panics, listing what is missing, if a scripted step leaves out an attempt it can have, after a retry or a rescue, or if a scripted step can fail on attempt 1 and its rescue step has no script. So the second check's script says what the shipping's attempt 2 would do, and the third's says what the rescue step and the charge's attempt 2 would do. The checks then show the workflow never got there.
4. **The run says what each step went through.** `run` returns a `ScriptedRun`: its `result` is the result an executor would have given, and `step` reads a step's attempts. `attempt(2)` of the charge holds the inputs it was built with, read with `input`: the card the rescue step contributed. Attempts that never ran aren't there, so the rescue step of the third check has none.
5. **The same script tests one decision of the flow.** The two runs of the second check differ only in the reason the shipping's first attempt gives. With `too-heavy`, the shipping is retried and the order succeeds. With `over-parcel-limit`, `heavy_parcels` gives it up, and the result carries the cause `FailWorkflow` with the policy's reason, `Failure::FailWorkflow` in Rust.

So the flow is tested apart from the steps' rules: a change to the charge's rules never breaks these checks, and a change to the workflow is caught by them. But neither kind of test runs one real journey from start to end, with the shop's real steps and its services faked.

## Outcomes

We have shown how to test a workflow's flow with scripted steps, checked before the journey runs.

| The usual way | With Itinera |
|---|---|
| Testing what happens after a failure often means making a real dependency fail, or mocking every call the code makes. | A script says how each attempt of a step ends, and the executor runs the rest of the journey as it would. |
| A test of the flow often breaks when a business rule changes. | Scripted steps never run their rules, so a change to those rules never changes what these tests see. |
