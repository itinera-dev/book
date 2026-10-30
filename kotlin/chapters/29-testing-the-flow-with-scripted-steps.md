# Testing the flow with scripted steps

## What you want to do

You want to check the shop's flow: that a declined card is rescued, that a parcel over the limit is given up after its first attempt, and that a charge ending in an error is never rescued. Each test should say how the steps end, without setting up a bank or a parcel that makes them end that way.

## The code

The program is [chapter 25](25-reading-the-result.md)'s shop, with its tests in a file beside it, as in [chapter 27](27-testing-a-step-alone.md). The shop's log prints a line for every event, and JUnit's console launcher shows all a test prints. So before each test, the method marked `@BeforeEach` sends what the program prints to nowhere, with Java's `System.setOut`, and after it, the one marked `@AfterEach` gives the console back:

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: quiet
asynchronous: ch29_testing_the_flow_with_scripted_steps_async.test
```

Each test makes an instance of the workflow for one of Ann's orders, a `WorkflowInstance<Sync>`, what `create()` gives for a synchronous workflow, from `org.itineradev.itinera.instance` and `org.itineradev.itinera.workflow`. `NewOrder` is as [chapter 25](25-reading-the-result.md) shows it, and `Parcel` as [chapter 21](21-deciding-where-a-steps-data-comes-from.md) does. The workflow, `Orders`, is declared as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) shows it, and its steps, policies and reporters are where that page says they are shown:

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: order
asynchronous: ch29_testing_the_flow_with_scripted_steps_async.test
```

The first test runs the instance with a script. `scripted(instance)`, from `org.itineradev.itinera.executor.testing`, starts it. `step` names a step, and what follows says what its attempts do in place of running it: what they contribute, and how they end. `onAttempt` moves on to another attempt of the same step, `andStep` to another step, and `run` runs the journey:

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: rescued
asynchronous: ch29_testing_the_flow_with_scripted_steps_async.test
```

The next three run one script, in which the shipping's first attempt fails with the reason each test chooses, and the last of them reads what the shipping was built with:

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: given_up
asynchronous: ch29_testing_the_flow_with_scripted_steps_async.test
```

The next ends the charge's first attempt in an error, as the bank's would:

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: outage
asynchronous: ch29_testing_the_flow_with_scripted_steps_async.test
```

The last test leaves out the shipping's attempt 2, and checks that `run` rejects the script by throwing a `TestMistake`, from `org.itineradev.itinera.testing`, whose `problems` hold a `Mistake.AttemptLeftOut`, from the same package, naming the attempt left out:

```example
name: ch29_testing_the_flow_with_scripted_steps.test
region: mistake
asynchronous: ch29_testing_the_flow_with_scripted_steps_async.test
```

In the asynchronous form, which tests [chapter 26](26-running-async-journeys-while-others-wait.md)'s shop, the instance is a `WorkflowInstance<Async>`, `run` is a `suspend` function, and each test runs its body in `runBlocking`, writing its script the same way.

The console launcher prints the same for both forms:

```test-run
name: ch29_testing_the_flow_with_scripted_steps
asynchronous: ch29_testing_the_flow_with_scripted_steps_async
```

## How it works

1. **The script replaces only a step's `run`.** Every step is still built for real, with its inputs and services: the shipping's first attempt was built with the weight the adapter supplied, 60. A step the script doesn't name runs for real, as the lookup and the pricing do here. Everything else is the executor's, as in any journey: the hooks, the decisions, the events and the result.
2. **A script names steps by their names in the workflow.** A rescue step is named by its own name, `charge_rescue`. `willFail`, `willTerminateAbnormally` and `willSucceed` end an attempt, and `willContribute` adds a contribution before it ends, which the executor commits only if the attempt succeeds. An attempt `onAttempt` doesn't name is attempt 1.
3. **The script must cover what can happen.** `run` checks it before the journey starts, and throws a `TestMistake` if a scripted step leaves out an attempt it can have, after a retry or a rescue, or can fail on attempt 1 while its rescue step has no script. So some of the other tests script attempts the workflow never reaches, such as the shipping's attempt 2 after `over-parcel-limit`.
4. **The run says what each step went through.** `run` returns a `ScriptedRun`: its `result` is the result an executor would have given, whose `isSucceeded` says whether the journey succeeded, as in [chapter 5](05-a-first-journey.md), and `step` gives a step's record. Its `attempts` lists the attempts it had, and `attempt(2)` gives attempt 2, or `null` when there was none. An attempt's `input` reads what its step was built with, as `Contributions`'s `read` does in [chapter 27](27-testing-a-step-alone.md): for the charge's attempt 2, the card the rescue step contributed. Attempts that never ran aren't there, so the rescue step of the charge that ended in an error has none.
5. **The same script tests one decision of the flow.** Two tests run the shipping's script with only the reason its first attempt gives changed. With `too-heavy`, the shipping is retried and the order succeeds. With `over-parcel-limit`, `failedShipping` gives it up, and `failedByAPolicy` reads the result's `failure`, [chapter 25](25-reading-the-result.md)'s `Failure`, or `null` for a journey that didn't fail: its cause is `FAIL_WORKFLOW`, with the policy's reason.

So the flow is tested apart from the steps' rules: a change to the charge's rules never breaks these tests, and a change to the workflow is caught by them. But neither kind of test runs one real journey from start to end, with the shop's real steps and its services faked.

## Outcomes

We have shown how to test a workflow's flow with scripted steps, checked before the journey runs.

| The usual way | With Itinera |
|---|---|
| Testing what happens after a failure often means making a real dependency fail, or mocking every call the code makes. | A script says how each attempt of a step ends, and the executor runs the rest of the journey as it would. |
| A test of the flow often breaks when a business rule changes. | Scripted steps never run their rules, so a change to those rules never changes what these tests see. |
