# Testing your workflows

## What you want to do

You want to know that the charge step charges what it should, that the `notify` policy writes the right message, and that an order with nothing to charge is never shipped: each part checked on its own, and the whole journey checked by its result and its events.

## The code

The shop charges, ships, and writes the customer a message with the `notify` workflow policy of [chapter 17](17-doing-something-when-the-journey-ends.md).

```example
name: ch24_testing
region: shop
```

The charge step is tested alone, in a workflow of its own with no other step.

```example
name: ch24_testing
region: step_alone
```

The `notify` policy is tested alone, attached to a workflow of one stand-in step, whose outcome decides how the journey ends. The test chooses that outcome, and reads the message from the output.

```example
name: ch24_testing
region: policy_alone
```

The whole journey of an order with nothing to charge is tested by its result, by its events and by what the customer was told. The events are kept by `RecordingReporter`, `itinera::report::RecordingReporter`, a reporter for tests that keeps every event it receives, and its clones share what they keep: the test adds a clone to the instance, runs the journey, and reads the events from the one it kept. It comes with the `testing` feature of `itinera`, which you enable for tests only, under `[dev-dependencies]` in `Cargo.toml`. The `line` function turns each event into its kind and what it is about, as [chapter 23](23-seeing-what-happened.md) reads events; a `hook_called` event's `hook` is a `HookSource`, which names the policy and its hook.

```example
name: ch24_testing
region: journey
```

The program runs every check, and stops at the first that doesn't hold. So that the book can show you what it prints, this is a program with a small `check` helper. In your own crate, the same checks sit in a test module, `#[cfg(test)] mod tests`, one `#[test]` function per check, named after the rule it checks, such as `a_free_order_is_never_shipped`, with `assert_eq!` in place of `check`. Then `cargo test` runs them all and reports each one.

```example
name: ch24_testing
region: main
```

It prints:

```output
name: ch24_testing
```

## How it works

1. **A step is tested through its data and its outcome.** A step knows nothing of the workflow around it, so it behaves the same in a workflow where it's the only step. The test gives it the data it needs, and reads its outcome from the journey's result and its contributions from the output. Adding steps around the charge in the shop never changes these tests.
2. **A step runs only inside a journey.** A step's contributor and step reporter are made only by the executor, for one attempt, so you can't call a step's `run` on its own. A test runs it in a small workflow of its own instead, as `ChargeAlone` does. The same goes for a policy whose hooks take a contributor.
3. **A workflow policy is tested with a stand-in step.** `notify` reacts to how the journey ended, not to the steps that brought it there, so a workflow of one stand-in step that succeeds or fails on demand is all it needs: the stand-in's outcome decides how the journey ends. The test then checks what the policy did: here, the message it wrote.
4. **The outside world is faked where the workflow instance is given it.** These steps and this policy reach nothing outside the journey. A step that does gets it as a service, from a provider of the workflow, and a policy through a role the workflow implements. Both can use the workflow's fields, which each instance is given on its builder, as [chapter 18](18-letting-policies-act-on-the-outside-world.md) shows, so a test creates the instance with fakes in those fields, and the steps and policies never know. [Part 5](part-5-without-itinera.md#testing-a-flow-by-mocking-it) tests shipping this way, with two fake carriers.
5. **A whole journey is tested by its result and its events.** The result says the journey failed with `nothing-to-charge`. The event stream says the rest: the charge was attempted once, failed and was given up, the `notify` policy's `on workflow failure` hook was called and its message committed, and the journey failed at the charge. There is no attempt of the ship step in the stream, so the order was never shipped. The test reads that from the events, rather than counting calls to a fake carrier.
6. **The events are the specification's own.** Every implementation of Itinera, in whatever language, reports the same events in the same order, so a test that compares event streams checks exactly what the specification promises.
7. **Tests give the same answer every time.** The tests use no clock, no network and no randomness, and no check reads anything that changes between runs, so nothing in a check changes from one run to the next.

## Remember

1. Test a step through the data it is given and the outcome it reports, never through the steps around it.
2. Test a workflow policy with a workflow of one stand-in step, whose outcome you choose to decide how the journey ends.
3. Fake the outside world in what each workflow instance is created with, so steps and policies run unchanged in tests.
4. Check a whole journey by its result and its events, not by counting calls.
