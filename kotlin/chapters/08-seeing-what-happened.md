# Seeing what happened

## What you want to do

When an order ends the way it did, you want to know who decided what, and when: which step reported which outcome, what the executor decided because of it, and in what order. And you want every journey of every workflow to tell you that the same way, so that one log reads them all, without a `println` in any step.

## The code

Everything that happens in a journey is an **event**, and the executor reports every one. A **reporter** receives them, one at a time, and decides what to do with each. The shop's log is a reporter: a class implementing the `Reporter` interface, from `org.itineradev.itinera.report`, whose `report` method writes a line for each event. The line gives the event's number and its kind, and, for the kinds the log says more about, the step the event is about and what it names.

```example
name: ch08_seeing_what_happened
region: log
```

The workflow lists the log in its `@Workflow` annotation, with `reporters = [Log::class]`. Its providers no longer print, since the log now says so, and the carrier's is now called `mainCarrier`, with `key = "carrier"`:

```example
name: ch08_seeing_what_happened
region: workflow
```

Everything else is [chapter 7](07-giving-a-step-the-tools-it-needs.md)'s shop, which runs Ann's and Gil's orders as [chapter 6](06-passing-data-on.md) shows. The faked bank and carrier still print what they do. It prints:

```output
name: ch08_seeing_what_happened
```

## How it works

1. **Every event says who decided what, and when.** An event records one thing that happened, and who it came from. Some are facts: the journey started, an attempt of a step started, a service was provided, a contribution was committed, and the step's own outcome, which the step decided and returned, and which the executor records, as `step_failed` with `too-heavy` for Gil's shipping. Others are the executor's decisions about that outcome: `step_given_up` says it won't run the shipping again, and `journey_failed` that the journey failed because of it. Each event's number says when, in the journey's order, and its timestamp when on the clock.
2. **The executor records which provider served which step.** When a step asks for a service, the executor calls its provider, lends the service, and records `service_provided`, naming the step, the key, the provider, by its method's name, and the **implementation** it made: `ship gets carrier from mainCarrier, a ch08_seeing_what_happened.MainCarrier`. It is the instance's class's name, with its package, for people to read rather than for code to compare. This fact says which tool a step got, never the service itself: like your data, a service never goes into an event.
3. **Every journey reports the same stream.** The events come in a fixed order and shape, whoever wrote the steps: a step's outcome, then what the executor decided about it. So one reporter reads every journey of every workflow the same way. **Every decision is reported in the journey's events, naming who made it.**
4. **Every event carries the same fields.** In Kotlin, an event is an `Event`, from `org.itineradev.itinera.event`, and every event has `kind`, an `EventKind` such as `EventKind.STEP_FAILED`, which prints as `step_failed`; `sequence`, which numbers the journey's events from 1; a `timestamp`; and the `journeyId` and the `workflow` it belongs to. The log leaves out the timestamp and the journey ID, which change on every run.
5. **What else an event says depends on its kind.** `Event` is a sealed interface with a data class for each kind, such as `StepFailed` and `ServiceProvided`, each adding what its kind carries. `detail` matches the event with `when`, and each branch `is StepFailed ->` lets the compiler treat the event as that class, so inside it the event has a `reason`, whose `code` the log writes. Since `detail` uses its `when` as a value, the compiler insists that it covers every kind, so it ends with `else`, which gives `null` for every kind the log says nothing more about.
6. **The executor's events name your data, and never carry its value.** `journey_started` lists `order`, `price` and `weight`, but not Ann's price, and `contribution_committed` names `receipt`, not the receipt. A step's reason is carried, since the step wrote it, as `too-heavy` is for Gil's shipping. The executor's events only say what happened to your data.
7. **A reporter decides where events go.** Its `report` receives each event and returns nothing. The workflow lists the classes of the reporters every journey gets, and each journey gets its own, made with its constructor, which takes nothing. A reporter only ever sees the events of its own journey.
8. **Events are delivered as they happen.** Each event reaches the log at once, before the journey goes on, so the log's lines and the bank's interleave in the order things happened.

But one line in Ann's output still comes from a `println` in a step: "the parcel leaves with receipt R-A-1". Only the shipping knows the parcel left, and it says so outside the log, unnumbered and tied to no journey.

## Outcomes

We have shown how a reporter sees who decided what in a journey, and when, as events.

| The usual way | With Itinera |
|---|---|
| When an order fails, code usually keeps no record of whether a function's own error, a caller's retry or a fallback decided it, unless someone thought to log it. | Every event says whether a step reported it or the executor decided it, in order, the same way for every workflow. |
| A log is often written after the fact, or only where code remembers to, so its lines rarely match the order things happened in. | Each event reaches the reporter as it happens, numbered in the journey's order. |
| Logs often hold the data they talk about, such as amounts and card numbers, because that is what was at hand. | The executor's events name the data the journey started with or a step gave back, and never carry its value. |
