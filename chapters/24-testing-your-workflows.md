# Testing your workflows

## What you want to do

You want to know that the charge step charges what it should, that the `notify` policy tells the customer the right thing, and that an order with nothing to charge is never shipped, without a bank, a carrier or an email service: each part checked on its own, and the whole journey checked by its result and its events.

## The code

The shop charges, ships, and tells the customer how the order went. Each workflow instance is created with the outbox through which messages leave the shop, an email service in production, and lends it to the `notify` workflow policy through the `Messages` role.

```example
name: ch24_testing
region: shop
```

The tests replace the outside world with fakes that keep what they are given: an outbox that sends nothing and keeps every text, and a reporter that keeps a line for every event.

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

The program runs every check, and stops at the first that does not hold.

```example
name: ch24_testing
region: main
```

It prints:

```output
name: ch24_testing
```

```note
name: ch24_testing
```

## How it works

1. **A step is tested through its data and its outcome.** A step knows nothing of the workflow around it, so it behaves the same in a workflow where it is the only step. The test gives it the data it needs, and reads its outcome from the journey's result and its contributions from the output. Adding steps around the charge in the shop never changes these tests.
2. **A workflow policy is tested with a stand-in step.** `notify` reacts to how the journey ended, not to the steps that brought it there, so a workflow of one stand-in step that succeeds or fails on demand is all it needs: the stand-in's outcome decides how the journey ends. The test then checks what the policy did: here, what it told the customer.
3. **The outside world is faked where the workflow instance is given it.** The `notify` policy reaches the outbox only through the `Messages` role, which each workflow instance provides from the outbox it was created with, so the test creates the instance with a fake outbox and the policy never knows. Nothing in the steps or the policy changes for the test.
4. **A whole journey is tested by its result and its events.** The result says the journey failed with `nothing-to-charge`. The event stream says the rest: the charge was attempted once, failed and was given up, the `notify` policy's `on workflow failure` hook was called, and the journey failed at the charge. There is no attempt of the ship step in the stream, so the order was never shipped. The test reads that from the events, rather than counting calls to a fake carrier.
5. **The events are the specification's own.** Every implementation of Itinera reports the same events in the same order, so a test that compares event streams checks what the specification promises, and reads the same in every language.
6. **Tests give the same answer every time.** The fakes use no clock, no network and no randomness, and no check reads anything that changes between runs, so nothing in a check changes from one run to the next.

## Remember

1. Test a step through the data it is given and the outcome it reports, never through the steps around it.
2. Test a workflow policy with a workflow of one stand-in step, whose outcome you choose to decide how the journey ends.
3. Fake the outside world in what each workflow instance is created with, so steps and policies run unchanged in tests.
4. Check a whole journey by its result and its events, not by counting calls.
