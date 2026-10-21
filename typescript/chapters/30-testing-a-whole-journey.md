# Testing a whole journey

## What you want to do

You want to run one order through the shop's real steps and policies, from start to end, and check the way its journey went: which steps succeeded, which failed and why, and how the journey ended. The bank and the carriers should be fakes, so the test never reaches the outside world.

## The code

The program is [chapter 26](26-running-async-journeys-while-others-wait.md)'s shop as a module, with its tests in a file beside it, as in [chapter 27](27-testing-a-step-alone.md), and the file quiets `console.log` as [chapter 29](29-testing-the-flow-with-scripted-steps.md)'s does. The tests bring their own bank, which charges or is down, and their own carrier, which collects every parcel or none, implementing `Payments` and `Carrier` as [chapter 26](26-running-async-journeys-while-others-wait.md) declared them, and throwing the error of [chapter 12](12-trying-again.md):

```example
name: ch30_testing_a_whole_journey.test
region: fakes
```

Each test runs an order through `pathOf`, which makes an instance of the shop's workflow, as the shop does, with a second argument to `instance`: the instance's own reporters, which it adds after those its workflow lists. Here it holds a `RecordingReporter`, from `itinera/testing`, which keeps every event of the journey, and whose `path` gives the journey's path once it has ended. `fakeProvider`, also from `itinera/testing`, then names one of the workflow's providers, `bank`, from [chapter 7](07-giving-a-step-the-tools-it-needs.md), or `carriers`, from [chapter 13](13-a-carrier-for-each-attempt.md), both as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) shows them, with the type token of the service it provides, and gives a function the executor calls in its place. That function is handed `provide`, which makes what it returns, as a provider's `this.provide(...)` does:

```example
name: ch30_testing_a_whole_journey.test
region: path
```

The shop approves the path of each kind of order once, and writes it down. The paths follow the workflow as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) declared it, with `outage` from [chapter 17](17-giving-up-a-step-early.md). Its steps and policies are as [chapter 26](26-running-async-journeys-while-others-wait.md) shows the charge, its rescue step, the shipping and `failedShipping`, [chapter 22](22-finishing-a-journey-early.md) the lookup and [chapter 20](20-an-optional-input.md) the pricing. Each test compares a journey's path with the approved one:

```example
name: ch30_testing_a_whole_journey.test
region: approved
```

`node --test` prints:

```test-run
name: ch30_testing_a_whole_journey
```

## How it works

1. **A fake provider replaces only the provider's function.** The workflow keeps its providers' keys and scopes. The steps still ask for `payments` and `carrier`, and `carriers` stays a provider called for every attempt. Only what the provider returns changes, and only for the journey of the instance `fakeProvider` was given.
2. **Everything else is real.** The steps run their own rules: the shipping refuses the parcel of 90 kg itself, with `over-parcel-limit`, as [chapter 27](27-testing-a-step-alone.md)'s test showed. The policies decide, the adapter supplies the weight, and the result and the events are the ones a real order would give.
3. **A path is the journey's flow, one line for each event of it.** `path` reads only the events that say how the flow went: each attempt's outcome, each decision about a step, and how the journey ended. Everything else, such as which provider served a step or which data was committed, is left out, so a path changes only when the flow does.
4. **Each line is a plain sentence.** `charge attempt 1 terminated abnormally` names the step, its attempt and how it ended, with the reason or the error after a colon. `ship given up: FailWorkflow from failedShipping` names the cause, and the policy that decided it. `journey failed at ship: no-carrier-can-take-it` names the step where the journey ended, and the reason it ended with: after retries are exhausted, the last attempt's reason, `too-heavy` for the parcel no carrier collects.
5. **An approved path is a decision of the flow, written down.** A person reads it once and agrees that this is what should happen to such an order. If a change to the workflow sends an order another way, the test fails. A change to a step's rules that keeps the flow keeps the path.
6. **The test awaits the journey,** as the shop does, with an executor of its own, and reads the path once the journey has ended.

So the shop tests each piece alone, its flow with scripts, and whole journeys by their paths. But the shop's log will soon write to a logging service, over the network too, where a write can fail.

## Outcomes

We have shown how to test a whole journey, with fake providers in place of the workflow's, by comparing its path with an approved one.

| The usual way | With Itinera |
|---|---|
| Testing a whole request often needs a test double for every dependency, wired by hand into the code under test. | `fakeProvider` replaces a workflow's provider by name, and the steps get the fake as they would the real service. |
| An end-to-end test usually checks the final state, and says little about the way a request went. | A journey's path lists each outcome and decision in order, and a test compares it with the path the shop approved. |
