# Seeing what happened

## What you want to do

When an order ends the way it did, you want to know who decided what, and when: which step reported which outcome, what the executor decided because of it, and in what order. And you want every journey of every workflow to tell you that the same way, so that one log reads them all, without a `println!` in any step.

## The code

Everything that happens in a journey is an **event**, and the executor reports every one. A **reporter** receives them, one at a time, and decides what to do with each. The shop's log is a reporter: a struct implementing the `Reporter` trait, whose `report` writes a line for each event. The line gives the event's number and its kind, and, for the kinds the log says more about, the step the event is about and what it names.

```example
name: ch08_seeing_what_happened
region: log
```

The workflow lists the log in `#[workflow(...)]`, with `reporters = [Log]`:

```example
name: ch08_seeing_what_happened
region: workflow
```

Everything else is [chapter 7](07-giving-a-step-the-tools-it-needs.md)'s shop, run for Ann's and Gil's orders, with one change: the providers no longer print when the shop gets a tool, since the log now says so. The faked bank and carrier still print what they do. It prints:

```output
name: ch08_seeing_what_happened
```

## How it works

1. **Every event says who decided what, and when.** An event records one thing that happened, and who it came from. Some are facts: the journey started, an attempt of a step started, a service was provided, a contribution was committed, and the step's own outcome, which the step decided and returned, and which the executor records, as `step_failed` with `too-heavy` for Gil's shipping. Others are the executor's decisions about that outcome: `step_given_up` says it won't run the shipping again, and `journey_failed` that the journey failed because of it. Each event's number says when, in the journey's order, and its timestamp when on the clock.
2. **Every journey reports the same stream.** The events come in a fixed order and shape, whoever wrote the steps: a step's outcome, then what the executor decided about it. So one reporter reads every journey of every workflow the same way. This is one of the things Itinera is for: since every decision has its place, **every decision is reported in the journey's events, naming who made it.**
3. **Every event carries the same fields.** In Rust, an event is an `Event`, and its fields are the same for every kind: `sequence`, which numbers the journey's events from 1; a `timestamp`; the `journey_id` and the `workflow` it belongs to; and its `body`, which says what happened. `event.kind()` names its kind, such as `step_failed`. The log leaves out the timestamp and the journey ID: nothing names the journeys here, so each gets a new random ID, and the two change on every run.
4. **What an event says is an enum.** `body` is an `EventBody`, with one variant for each kind, holding what that kind carries: `JourneyStarted` the names of the initial data, `StepFailed` the step and its reason, `ContributionCommitted` the contribution's name. `EventBody` is marked `#[non_exhaustive]`, since later versions may add kinds, so the log's `match` ends with a `_` arm, for every kind it says nothing more about.
5. **The executor's events name your data, and never carry its value.** `journey_started` lists `order`, `price` and `weight`, but not Ann's price, and `contribution_committed` names `receipt`, not the receipt. A step's reason is carried, since the step wrote it, as `too-heavy` is for Gil's shipping. Your data stays in the data bag; the executor's events only say what happened to it.
6. **A reporter decides where events go.** It could write them to a log, send them to a monitoring service, keep only the failures, or ignore them all. Its `report` receives each event as `&Event`, and returns `Result<(), Error>`: the log never fails, and returns `Ok(())`. The workflow lists the reporters every journey gets, and each journey gets its own, made with `Default`, which is why `Log` derives it. A reporter only ever sees the events of its own journey.
7. **Events are delivered as they happen.** Each event reaches the log at once, before the journey goes on, which is why the log's lines and the bank's are interleaved in the order things happened: `service_provided` comes just before the bank charges, and `step_succeeded` just after.

The log shows everything the executor did, and the shop's steps don't print it anymore. But one line in Ann's output still comes from a `println!` in a step: "the parcel leaves with receipt R-A-1". The shipping alone knows that the parcel left, and it says so outside the log, with no number and nothing to tie it to the journey.

## Outcomes

We have shown how a reporter sees who decided what in a journey, and when, as events.

| The usual way | With Itinera |
|---|---|
| When an order fails, code usually keeps no record of whether a function's own error, a caller's retry or a fallback decided it, unless someone thought to log it. | Every event says whether a step reported it or the executor decided it, in order, the same way for every workflow. |
| A log is often written after the fact, or only where code remembers to, so its lines rarely match the order things happened in. | Each event reaches the reporter as it happens, numbered in the journey's order. |
| Logs often hold the data they talk about, such as amounts and card numbers, because that is what was at hand. | The executor's events name the data the journey started with or a step gave back, and never carry its value. |
