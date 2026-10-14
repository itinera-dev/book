# Without Itinera

Part 4 ran the shop for real: many journeys, their results, their events and their tests. This section shows two things that are usually hard without Itinera, written the usual way in plain Rust, then with Itinera beside them. Both usual programs are real Rust, compiled and run like every other example, so what they print is what they really do.

## Logging that depends on who wrote the code

### The usual code

The checkout has three parts, each a module written by a different person, and each logs in its own way. The warehouse code logs before every call.

```example
name: part5_scattered_logging
region: inventory
```

The payments code logs only what goes wrong.

```example
name: part5_scattered_logging
region: payments
```

The shipping code decided that a carrier that can't take a parcel isn't worth failing the order for, so it turns the carrier's `Result` into an `Option` with `.ok()`, dropping the error, and logs nothing.

```example
name: part5_scattered_logging
region: shipping
```

The checkout calls the three parts in turn.

```example
name: part5_scattered_logging
region: checkout
```

Three orders go through it. It prints:

```output
name: part5_scattered_logging
```

Ask what happened to order A-2. The log says it is done, with no tracking number, and nothing else. To learn that the carrier could not take the parcel, you read the shipping code and find the error it dropped. Each part of this code mixes its business rule with a logging habit of its own, and the shipping part also hides a flow decision: an order the carrier could not take is still called done. Reconstructing what happened to one order means reading all three parts and knowing which of them stays silent.

### With Itinera

The same three parts are steps. None of them logs: each reports its outcome, and the ship step fails with the reason `carrier-down`, carrying the carrier's message, when the carrier can't take the parcel.

```example
name: part5_one_event_stream
region: steps
```

One reporter, listed on the workflow with `reporters = [Log]`, writes every event of every journey to the log.

```example
name: part5_one_event_stream
region: log
```

It prints:

```output
name: part5_one_event_stream
```

Every journey reports the same stream of events, in the same order, whoever wrote the steps. Order A-2's stream says the stock was reserved, the price was charged, and the ship step failed with `carrier-down`, then the journey failed at the ship step. Nothing could stay silent: a step reports an outcome, or the executor reports why it did not. Where the events go is the reporter's decision alone: change the reporter, and every journey of the shop is logged the new way.

This version behaves differently, and says so: the usual code called order A-2 done, and the Itinera version fails it. Keeping the old behaviour, an order completed without a parcel sent, is still a choice the shop can make, but it is now a decision someone writes down on purpose, and the events show it every time, rather than an error dropped inside the shipping code.

## Testing a flow by mocking it

### The usual code

Shipping falls back to the backup carrier when the main one fails. The fallback lives inside the shipping code.

```example
name: part5_mocked_carriers
region: ship
```

To test the fallback, the test mocks both carriers behind the `Carrier` trait, and each mock counts how often it's called in a `Cell`.

```example
name: part5_mocked_carriers
region: mock
```

The tests then check the tracking number and count the calls.

```example
name: part5_mocked_carriers
region: tests
```

It prints:

```output
name: part5_mocked_carriers
```

These tests check two rules at once, how to ship and when to fall back, and to do so they must know how shipping is built inside: which carrier it calls first, and how many times. Counting calls checks the code's structure rather than what happened to the order. Change the fallback, for example to use the backup carrier first at weekends, and the mocks and their counts are rewritten, though shipping itself has not changed.

### With Itinera

The ship step ships through the carrier it is lent as a service, a `#[service] carrier: &dyn Carrier` parameter of its `run`, and knows nothing of a second one. Its retry budget lets it be tried twice, and the workflow's `#[provider(per_attempt)]` method gives each attempt a fresh carrier: the main carrier on the first attempt, the backup carrier after. The test's carriers are fakes that are up or down as the test says, and count nothing.

```example
name: part5_testing_the_fallback
region: fake
```

The ship step is tested alone: the test calls its `run` itself, with one fake carrier as its service and a contributor lent by `Contributions`, as in [chapter 27](27-testing-a-step-alone.md). It ships when the carrier is up, and fails in a way worth trying again when the carrier is down.

```example
name: part5_testing_the_fallback
region: step_alone
```

The fallback belongs to the workflow, so it is tested on a whole journey, by its events: with the main carrier down and the backup up, the ship step's first attempt fails, the executor decides to try it again, and the second attempt succeeds. The events are kept by `RecordingReporter`, as in [chapter 30](30-testing-a-whole-journey.md), and a small function, `attempt_line`, not shown here, keeps a line for each event about an attempt.

```example
name: part5_testing_the_fallback
region: journey
```

It prints an `ok` line for each of its four checks, as the programs of [chapter 27](27-testing-a-step-alone.md) do:

```output
name: part5_testing_the_fallback
```

Each rule is tested where it lives. The step's test checks shipping and nothing else. The journey's test checks the fallback through the event stream, which says what happened in the specification's own words: a failed attempt, a retry, a successful attempt. Change which carrier comes first, and only the workflow's provider and the journey's expected tracking number change; the step and its tests stay as they are. [Chapters 27 to 30](27-testing-a-step-alone.md) test the shop the same way: a step alone, a hook or an adapter alone, the flow with scripted steps, and a whole journey by its path.

## What you gain

1. **One place to read the flow.** What follows a parcel the carrier could not take, and which carrier each attempt of shipping uses, are read from the workflow and the outcomes it reacts to, not searched for through the business code.
2. **Business code you can test alone.** Each step is checked through its data and its outcome, with one fake for what it uses, without mocking the rest of the shop.
3. **Observation that is the same everywhere.** Every journey reports the same events in the same order, so one reporter logs the whole shop, and tests read the same stream to check what happened.
