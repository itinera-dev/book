# Testing a whole journey

## What you want to do

You want to run one order through the shop's real steps and policies, from start to end, and check the way its journey went: which steps succeeded, which failed and why, and how the journey ended. The bank and the carriers should be fakes, so the test never reaches the outside world.

## The code

The program is [chapter 25](25-reading-the-result.md)'s shop, with its tests in a module that only a test build compiles, as in [chapter 26](26-testing-a-step-alone.md). The tests bring their own bank, which charges or is down, and their own carrier, which collects every parcel or none:

```example
name: ch29_testing_a_whole_journey
region: fakes
```

Each test runs an order through `path_of`, which makes an instance of the shop's workflow, as `main` does, with two more calls on its builder: `fake_provider`, which comes with the `testing` feature, and `reporter`, which any journey's builder has. `fake_provider` names one of the workflow's providers, `bank` or `carriers`, and gives a function the executor calls in its place; the type in `::<...>` is the service it provides. `reporter` adds a `RecordingReporter`, from `itinera::report`, which also comes with the `testing` feature, to the journey's reporters. It keeps every event, and its clones share what they keep, so the test reads the events from the clone it kept. Its `path` gives the journey's path:

```example
name: ch29_testing_a_whole_journey
region: path
```

The shop approves the path of each kind of order once, and writes it down. Each test compares a journey's path with the approved one:

```example
name: ch29_testing_a_whole_journey
region: approved
```

As in [chapter 28](28-testing-the-flow-with-scripted-steps.md), `cargo test` hides what the shop's log, its desk and its messages print. It prints:

```test-run
name: ch29_testing_a_whole_journey
```

## How it works

1. **A fake provider replaces only the provider.** The workflow keeps its providers' keys and scopes. The steps still ask for `payments` and `carrier`, and `carriers` stays a provider made for every attempt. Only what the provider returns changes.
2. **Everything else is real.** The steps run their own rules: the shipping refuses the parcel of 90 kg itself, with `over-parcel-limit`, as [chapter 26](26-testing-a-step-alone.md)'s test showed. The policies decide, the adapter supplies the weight, and the result and the events are the ones a real order would give.
3. **A path is the journey's flow, one line for each event of it.** `path` reads only the events that say how the flow went: each attempt's outcome, each decision about a step, and how the journey ended. Everything else, such as which provider served a step or which data was committed, is left out, so a path changes only when the flow does.
4. **Each line is a plain sentence.** `charge attempt 1 terminated abnormally` names the step, its attempt and how it ended, with the reason or the error after a colon. `ship given up: FailWorkflow from failed_shipping` names the cause, and the policy that decided it. `journey failed at ship: no-carrier-can-take-it` names the step where the journey ended, and the reason it ended with: after retries are exhausted, the last attempt's reason, `too-heavy` for the parcel no carrier collects.
5. **An approved path is a decision of the flow, written down.** A person reads it once and agrees that this is what should happen to such an order. If a change to the workflow sends an order another way, the test fails. A change to a step's rules that keeps the flow keeps the path.

So the shop tests each piece alone, its flow with scripts, and whole journeys by their paths. But the shop's real bank and carriers answer over the network, and while a step waits for them, its journey holds the program's thread.

## Outcomes

We have shown how to test a whole journey, with fake providers in place of the workflow's, by comparing its path with an approved one.

| The usual way | With Itinera |
|---|---|
| Testing a whole request often needs a test double for every dependency, wired by hand into the code under test. | `fake_provider` replaces a workflow's provider by name, and the steps get the fake as they would the real service. |
| An end-to-end test usually checks the final state, and says little about the way a request went. | A journey's path lists each outcome and decision in order, and a test compares it with the path the shop approved. |
