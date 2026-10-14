# Seeing every event of a journey

## What you want to do

The log stopped at Pat's charge, so no record says how Pat's journey ended. You want to see every event of a journey as it happens, without writing a reporter of your own.

## The code

The shop is [chapter 31](31-stopping-a-journey-when-its-log-fails.md)'s, with one more reporter. `DebugReporter`, from `itinera::report`, comes with `itinera`, and prints every event of a journey on a line of its own. The workflow lists it first:

```example
name: ch32_seeing_every_event_of_a_journey
region: reporters
asynchronous: ch32_seeing_every_event_of_a_journey_async
```

In the asynchronous form, `DebugReporter` stays under `reporters` too, since printing waits for nothing, and the reporters under `reporters` receive each event before those under `async_reporters`. `main` runs Pat's order alone, and prints:

```output
name: ch32_seeing_every_event_of_a_journey
asynchronous: ch32_seeing_every_event_of_a_journey_async
```

## How it works

1. **A ready-made reporter prints every event.** `DebugReporter` prints each event as Rust's `Debug` shows it: its sequence number, its timestamp, the journey's ID, the workflow and the body, with everything the body carries. It suits trying a workflow out, debugging one and watching a test run. A reporter for the shop's real log still formats its own lines, as the log does.
2. **It never fails.** So `DebugReporter` never costs a journey, as [chapter 31](31-stopping-a-journey-when-its-log-fails.md) said every reporter should.
3. **Reporters receive each event in the order the workflow lists them.** `DebugReporter` comes first, so it printed event 13, the charge's `step_succeeded`, before the log failed on it.
4. **`journey_aborted` reaches every reporter but the one that failed.** Event 14 reached `DebugReporter`, and not the log. It names the abort's reason, `ReporterFailed`, the step the journey was in, `charge`, and the log's error.

Pat's journey is now on record to its end. But the shop still has to refund Pat, and no line holds the receipt the bank gave: the charge never got to commit it. Nor does any line name Pat: they name the journey, `A-15`.

## Outcomes

We have shown how a ready-made reporter shows every event of a journey, up to its end.

| The usual way | With Itinera |
|---|---|
| Seeing what a request did means adding print statements, then taking them out. | A reporter that comes with Itinera prints every event, listed once on the workflow. |
| A crash in one part of the logging often hides what happened next. | `journey_aborted` reaches every reporter but the one that failed, with the reason and the error. |
