# Testing your workflows

## What you want to do

You want to know that the charge step charges what it should, that the `notify` policy tells the customer the right thing, and that an order with nothing to charge is never shipped, without a bank, a carrier or an email service: each part checked on its own, and the whole journey checked by its result and its events.

## The code

The shop charges, ships, and tells the customer how the order went. The workflow struct holds an `outbox` field, the way messages leave the shop: an email service in production. Each workflow instance is created with an outbox, and lends it to the `notify` workflow policy through the `Messages` role.

```example
name: ch24_testing
region: shop
```

The tests replace the outside world with fakes that keep what they are given: an outbox that sends nothing and keeps every text, and a reporter that keeps a line for every event. Each sends what it keeps down a `std::sync::mpsc` channel, whose receiving end the test holds on to.

```example
name: ch24_testing
region: fakes
```

The charge step is tested alone, in a workflow of its own with no other step.

```example
name: ch24_testing
region: step_alone
```

The `notify` policy is tested alone, attached to a workflow of one stand-in step, whose outcome decides how the journey ends. The test chooses that outcome, and creates the instance with the fake outbox.

```example
name: ch24_testing
region: policy_alone
```

The whole journey of an order with nothing to charge is tested by its result, by its events and by what the customer was told.

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
3. **A workflow policy is tested with a stand-in step.** `notify` reacts to how the journey ended, not to the steps that brought it there, so a workflow of one stand-in step that succeeds or fails on demand is all it needs: the stand-in's outcome decides how the journey ends. The test then checks what the policy did: here, what it told the customer.
4. **The outside world is faked where the workflow instance is given it.** The `notify` policy reaches the outbox only through the `Messages` role, which each workflow instance provides from the outbox it was created with. The workflow's own state, here the `outbox` field, is given to each instance on its builder, with `.outbox(...)`, so the test creates the instance with a fake outbox and the policy never knows. Nothing in the steps or the policy changes for the test.
5. **The fakes report through channels.** The instance and its reporters move into `run`, so the test can't look inside them afterwards. Instead, the fakes send what they receive down a channel, and the test reads the other end once the journey is over.
6. **A fake that can't report fails the test.** The `Messages` role's `tell` returns nothing, so the fake outbox can't hand a failure back through it. If the test has stopped reading, the fake panics, and `itinera` never catches a panic, so the test fails as a failed assertion would. The recording reporter, by contrast, can return an `Err`, which would abort the journey and show up in the result.
7. **A whole journey is tested by its result and its events.** The result says the journey failed with `nothing-to-charge`. The event stream says the rest: the charge was attempted once, failed and was given up, the `notify` policy's `on workflow failure` hook was called, and the journey failed at the charge. There is no attempt of the ship step in the stream, so the order was never shipped. The test reads that from the events, rather than counting calls to a fake carrier.
8. **The events are the specification's own.** Every implementation of Itinera, in whatever language, reports the same events in the same order, so a test that compares event streams checks exactly what the specification promises.
9. **Tests give the same answer every time.** The fakes use no clock, no network and no randomness, and no check reads anything that changes between runs, so nothing in a check changes from one run to the next.

## Remember

1. Test a step through the data it is given and the outcome it reports, never through the steps around it.
2. Test a workflow policy with a workflow of one stand-in step, whose outcome you choose to decide how the journey ends.
3. Fake the outside world in what each workflow instance is created with, so steps and policies run unchanged in tests.
4. Check a whole journey by its result and its events, not by counting calls.
