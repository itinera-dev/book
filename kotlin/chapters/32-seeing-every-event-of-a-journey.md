# Seeing every event of a journey

## What you want to do

The log stopped at Pat's charge, so no record says how Pat's journey ended. You want to see every event of a journey as it happens, without writing a reporter of your own.

## The code

The shop is [chapter 31](31-stopping-a-journey-when-its-log-fails.md)'s, with one more reporter. `DebugReporter`, from `org.itineradev.itinera.report`, comes with Itinera, and prints every event of a journey on a line of its own. The workflow lists it first:

```example
name: ch32_seeing_every_event_of_a_journey
region: reporters
asynchronous: ch32_seeing_every_event_of_a_journey_async
```

The log is [chapter 31](31-stopping-a-journey-when-its-log-fails.md)'s, the desk is [chapter 11](11-naming-a-journey.md)'s and `Notify` is [chapter 19](19-letting-policies-act-on-the-outside-world.md)'s. The rest of the workflow is as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) declares it.

In the asynchronous form, the workflow lists `AsyncDebugReporter`, from `org.itineradev.itinera.report.async`, which prints the same lines as an `AsyncReporter`, since every reporter of an asynchronous workflow is one, as [chapter 26](26-running-async-journeys-while-others-wait.md) said. Its workflow is otherwise as chapter 26 shows it.

`main` runs Pat's order alone, as [chapter 31](31-stopping-a-journey-when-its-log-fails.md) runs it, and prints:

```output
name: ch32_seeing_every_event_of_a_journey
asynchronous: ch32_seeing_every_event_of_a_journey_async
```

## How it works

1. **A ready-made reporter prints every event.** `DebugReporter` prints each event as its data class prints it: the class's name, then its sequence number, its timestamp, the journey's ID, the workflow and everything its kind carries. An event is plain data, so nothing is left out. It is a reporter like any other, a class implementing `Reporter`, so the workflow lists it by its class, as it does the log. It suits trying a workflow out, debugging one and watching a test run. A reporter for the shop's real log still formats its own lines, as the log does.
2. **It never fails.** It prints and returns, so `DebugReporter` never costs a journey, as [chapter 31](31-stopping-a-journey-when-its-log-fails.md) said every reporter should.
3. **Reporters receive each event in the order the workflow lists them,** as [chapter 10](10-more-than-one-reporter.md) showed. `DebugReporter` comes first, so it printed event 13, the charge's `StepSucceeded`, before the log failed on it.
4. **`journey_aborted` reaches every reporter but the one that failed.** Event 14, a `JourneyAborted`, reached `DebugReporter`, and the desk, which prints nothing for it, but not the log. Its `abort` is the `ReporterFailed` of [chapter 31](31-stopping-a-journey-when-its-log-fails.md), naming the step the journey was in, `charge`, and the message of the log's error.

Pat's journey is now on record to its end. But the shop still has to refund Pat, and no line holds the receipt the bank gave: the charge never got to commit it. Nor does any event name Pat: each names the journey, `A-15`.

## Outcomes

We have shown how a ready-made reporter shows every event of a journey, up to its end.

| The usual way | With Itinera |
|---|---|
| Seeing what a request did often means adding `println` calls, then taking them out. | A reporter that comes with Itinera prints every event, listed once on the workflow. |
| A crash in one part of the logging often hides what happened next. | `journey_aborted` reaches every reporter but the one that failed, with the reason and the error. |
