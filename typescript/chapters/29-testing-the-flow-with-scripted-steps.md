# Testing the flow with scripted steps

## What you want to do

You want to check the shop's flow: that a declined card is rescued, that a parcel over the limit is given up after its first attempt, and that a charge ending in an error is never rescued. Each test should say how the steps end, without setting up a bank or a parcel that makes them end that way.

## The code

The program is [chapter 26](26-running-async-journeys-while-others-wait.md)'s shop as a module, with its tests in a file beside it, as in [chapter 27](27-testing-a-step-alone.md). The tests run the shop's own workflow, whose log prints a line for every event, and Node's test runner shows all a test prints. So the file starts by replacing `console.log` with a function that prints nothing: `mock.method`, from `node:test`, replaces a method of an object, here for every test of the file.

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: quiet
```

Each test makes an instance of the shop's workflow for one of Ann's orders, given whole to `instance`, as [chapter 19](19-letting-policies-act-on-the-outside-world.md) does. The workflow is declared as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) shows it, with the policy `outage` from [chapter 17](17-giving-up-a-step-early.md). Its steps and policies are as [chapter 26](26-running-async-journeys-while-others-wait.md) shows the charge, its rescue step, the shipping and `failedShipping`, [chapter 22](22-finishing-a-journey-early.md) the lookup and [chapter 20](20-an-optional-input.md) the pricing:

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: order
```

The first test runs the instance with a script. `scripted`, from `itinera/testing`, starts one for an instance. `step` names a step, and what follows says what its attempts do in place of running it: what they contribute, and how they end. `onAttempt` moves on to another attempt of the same step, `andStep` to another step, and `run` runs the journey, resolving once it has ended:

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: rescued
```

The next three run one script, in which the shipping's first attempt fails with the reason each test chooses, and the last of them reads what the shipping was built with:

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: given_up
```

The last test ends the charge's first attempt in an error, as the bank's would:

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: outage
```

With the shop's lines quiet, `node --test` prints only the tests:

```test-run
name: ch29_testing_the_flow_with_scripted_steps
```

## How it works

1. **The script replaces only a step's `run`.** Every step is still built for real, with its inputs and its services: the shipping's first attempt was built with the weight the adapter supplied, 60. A step the script doesn't name runs for real, as the lookup and the pricing do here. Everything else is the executor's, as in any journey: the hooks, the decisions, the events and the result.
2. **A script names steps by their names in the workflow.** A rescue step is named by its own name, `charge_rescue`. `willFail`, `willTerminateAbnormally` and `willSucceed` end an attempt, and `willContribute` adds a contribution before it ends, which the executor commits only if the attempt succeeds. An attempt `onAttempt` doesn't name is attempt 1.
3. **The script must cover what can happen.** `run` checks it before the journey starts, and rejects with a `TestMistake`, from `itinera/testing`, listing what is missing, if a scripted step leaves out an attempt it can have, after a retry or a rescue, or if a scripted step can fail on attempt 1 and its rescue step has no script. So the shipping's script says what its attempt 2 would do, and the script of the charge that ends in an error says what the rescue step and the charge's attempt 2 would do. The tests then show the workflow never got there.
4. **The run says what each step went through.** `run` resolves to a `ScriptedRun`: its `result` is the result an executor would have given, read as [chapter 25](25-reading-the-result.md) does, and `step` gives a step's record. Its `attempts` lists the attempts it had, and `attempt(2)` gives attempt 2 as an `Option`, which `expect` unwraps, as in [chapter 11](11-naming-a-journey.md). An attempt's `input` reads what its step was built with, as the data bag's `read` does: for the charge's attempt 2, the card the rescue step contributed. Attempts that never ran aren't there: the rescue step of the charge that ended in an error has none.
5. **The same script tests one decision of the flow.** Two tests run the shipping's script with only its first attempt's reason changed. With `too-heavy`, the shipping is retried and the order succeeds. With `over-parcel-limit`, `failedShipping` gives it up, and the result's failure has the cause `FailWorkflow`, with the policy's reason, which `failedByAPolicy` reads once it has compared the cause, as `why` does in [chapter 25](25-reading-the-result.md).
6. **A scripted run is awaited like any journey,** whatever its steps' execution mode.

So the flow is tested apart from the steps' rules: a change to the charge's rules never breaks these tests, and a change to the workflow is caught by them. But neither kind of test runs one real journey from start to end, with the shop's real steps and its services faked.

## Outcomes

We have shown how to test a workflow's flow with scripted steps, checked before the journey runs.

| The usual way | With Itinera |
|---|---|
| Testing what happens after a failure often means making a real dependency fail, or mocking every call the code makes. | A script says how each attempt of a step ends, and the executor runs the rest of the journey as it would. |
| A test of the flow often breaks when a business rule changes. | Scripted steps never run their rules, so a change to those rules never changes what these tests see. |
