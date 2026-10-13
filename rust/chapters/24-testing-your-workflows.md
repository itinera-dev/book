# Testing your workflows

## What you want to do

You want to know that the charge step charges what it should, that the `notify` policy writes the right message, and that an order with nothing to charge is never shipped: each part checked on its own, and the whole journey checked by its result and its events.

## The code

The shop charges, ships, and writes the customer a message with the `notify` workflow policy of [chapter 17](17-doing-something-when-the-journey-ends.md).

```example
name: ch24_testing
region: shop
```

The charge step is tested alone, outside any journey: the test builds it with a price, calls its `run` itself, and checks the outcome it returns. The contributor `run` asks for is lent by `Contributions`, `itinera::journey::Contributions`, a recorder for tests that keeps what the step contributes, read back with `read`. `read` returns a `Read`: `Present` with the value, `Absent` when nothing was contributed under that key, or `OtherType` when the value has another type. It comes with the `testing` feature of `itinera`, which you enable for tests only, under `[dev-dependencies]` in `Cargo.toml`.

```example
name: ch24_testing
region: step_alone
```

The `notify` policy is tested alone too: the test calls its hooks, `shipped` and `failed`, with the data they ask for and a contributor lent by `Contributions`, and reads back the message they wrote.

```example
name: ch24_testing
region: policy_alone
```

The whole journey of an order with nothing to charge is tested by its result, by its events and by what the customer was told. The events are kept by `RecordingReporter`, `itinera::report::RecordingReporter`, a reporter for tests that keeps every event it receives, and its clones share what they keep: the test adds a clone to the instance, runs the journey, and reads the events from the one it kept. It comes with the `testing` feature too. The `line` function turns each event into its kind and what it is about, as [chapter 8](08-seeing-what-happened.md) reads events; a `hook_called` event's `hook` is a `HookSource`, which names the policy and its hook.

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

1. **A step is tested through its data and its outcome.** A step knows nothing of the workflow around it, so a test can build it and call its `run` like any method. The test gives it the data it needs, checks the outcome `run` returns, with methods such as `is_success` and `reason`, and reads back what it contributed. Adding steps around the charge in the shop never changes these tests.
2. **Recorders lend what the executor would.** In a journey, the executor makes a step's contributor and step reporter for one attempt, and commits the contributions only if the attempt succeeds. In a test, recorders lend them and keep what they receive: `Contributions` lends a contributor, and a step or a hook that asks for its reporter is lent one the same way, by `StepEventReporter` or `HookEventReporter`. A recorder only keeps what it is given: committing a successful attempt's contributions is the executor's work, which the journey's test checks.
3. **A policy's hooks are called like any method.** A hook asks for what it needs through its parameters, so the test passes the order, the tracking number and a contributor itself. `notify` reacts to how a journey ended, but its hooks never know how the journey got there, so the test calls `shipped` or `failed` directly and checks what they did: here, the message they wrote.
4. **The outside world is faked where it reaches your code.** These steps and this policy reach nothing outside the journey. A step that does gets it as a service, a `#[service]` parameter of its `run`, so a test that calls `run` itself passes a fake there, and the step never knows. In a whole journey, the service comes from a provider of the workflow, and a policy reaches the outside through a role the workflow implements. Both can use the workflow's fields, which each instance is given on its builder, as [chapter 18](18-letting-policies-act-on-the-outside-world.md) shows, so a journey's test creates the instance with fakes in those fields. [Part 5](part-5-without-itinera.md#testing-a-flow-by-mocking-it) tests shipping both ways, with fake carriers.
5. **A whole journey is tested by its result and its events.** The result says the journey failed with `nothing-to-charge`. The event stream says the rest: the charge was attempted once, failed and was given up, the `notify` policy's `on workflow failure` hook was called and its message committed, and the journey failed at the charge. There is no attempt of the ship step in the stream, so the order was never shipped. The test reads that from the events, rather than counting calls to a fake carrier.
6. **The events are the specification's own.** Every implementation of Itinera, in whatever language, reports the same events in the same order, so a test that compares event streams checks exactly what the specification promises.
7. **Tests give the same answer every time.** The tests use no clock, no network and no randomness, and no check reads anything that changes between runs, so nothing in a check changes from one run to the next.

## Remember

1. Test a step through the data it is given and the outcome it reports, never through the steps around it.
2. Test a policy by calling its hooks with the data they ask for and a contributor lent by `Contributions`.
3. Fake the outside world where it reaches your code: a fake service passed to `run`, or fakes in what a workflow instance is created with, so steps and policies run unchanged in tests.
4. Check a whole journey by its result and its events, not by counting calls.
