# Seeing what happened

## What you want to do

When a customer asks what happened to their order, you want the answer in the shop's log: a line for everything that happened to every order, in the same shape whichever step or policy was involved, labelled with the customer's name so the order is easy to find, and with the parcel's tracking number once it has shipped. You also want to keep some orders' events in an archive for the accountants, and know what happens when the archive can't take any more.

## The code

Most of what the log shows is reported by the executor, but a step can add events of its own, about what only it knows. The ship step says when it has handed the parcel to the carrier: it asks for its step reporter with a `#[reporter]` parameter of type `StepReporter<'_>`, and calls `info`. Then it gives the reporters the tracking number with `set_global_metadata`, and contributes it too.

```example
name: ch23_events
region: ship
asynchronous: ch23_events_async
```

A reporter receives every event of a journey, one at a time, and decides what to do with it. This one implements the `Reporter` trait and writes a line to the log for each event, with the event's kind and the details the log cares about. It also receives the journey's global metadata, entries of a key and a value such as the customer and the tracking number, and keeps them to label its lines.

```example
name: ch23_events
region: log
asynchronous: ch23_events_async
```

The workflow lists the reporter in `#[workflow(...)]`, so every journey of the shop is logged.

```example
name: ch23_events
region: workflow
asynchronous: ch23_events_async
```

The archive is a reporter too, with room for only a few events. When it's full, it fails.

```example
name: ch23_events
region: archive
asynchronous: ch23_events_async
```

Each order's instance is given the customer's name as global metadata, with `.reporters_global_metadata(...)` on its builder. The first order runs with the log alone. The second is also archived: the archive is added to that order's instance only.

```example
name: ch23_events
region: run
asynchronous: ch23_events_async
```

In the asynchronous form, the reporters implement `AsyncReporter`, with an `async fn report`; the workflow, asynchronous because its steps are, lists them under `async_reporters`; the instance adds one with `.async_reporter(...)`; and the step's `run` is an `async fn` whose `AsyncStepReporter` is awaited, for its event and for the tracking number alike. A reporter's `set_global_metadata` stays a plain `fn` in `AsyncReporter`, since receiving an entry is only keeping it.

It prints:

```output
name: ch23_events
```

## How it works

1. **Every journey reports the same stream of events.** An event is a record of one thing that happened: the journey started, listing the keys of its initial data; an attempt of a step started; the step succeeded, failed or skipped itself; a contribution was committed to the data bag; a policy's hook was called; the executor decided to try a step again or to give up on it; the journey ended. The order of these events is fixed, so the stream of every journey reads the same way: what happened, the hooks that were consulted, what was decided.
2. **Every event carries the same fields.** Its kind, a sequence number that increases through each journey, a timestamp, the journey ID and the workflow's name; an event about a step also carries the step's name and the attempt number. In Rust these are fields of `Event`, such as `event.journey_id` and `event.sequence`, and `event.kind()` names the kind in snake_case. The log above leaves the timestamp out, so that it prints the same on every run.
3. **What an event says is an enum.** The event's `body` is an `EventBody`, with one variant per kind, holding what that kind carries: `JourneyStarted` its initial keys, `ContributionCommitted` its key, `JourneyAborted` its abort. `EventBody` is `#[non_exhaustive]`, so the log's `detail` ends with a `_` arm for every kind it has nothing more to say about.
4. **Events carry keys, never your data.** `journey_started` lists `order` and `price`, but not their values, and `contribution_committed` names the key `charged`, not the amount. Failure reasons and error messages are carried, since you wrote them.
5. **Steps and policies add only what they alone know.** A step can emit `step_info`, `step_warning` and `step_error`, through the `info`, `warning` and `error` methods of its step reporter, and a policy's hook can emit `journey_info`, `journey_warning` and `journey_error` through its own `#[reporter]` parameter, each with a message and, if you like, some data. They never need to log what the executor already reports.
6. **A reporter decides where events go.** It can write them to a log, send them to a monitoring service, keep only the failures, or ignore them all. Its `report` receives each `&Event` and returns `Result<(), Error>`. The workflow lists the reporters every journey gets, with `reporters = [Log]`, and each is made anew with `Default` for each journey, so a reporter only ever sees the events of its own journey. An instance can add reporters for its journey alone, with `.reporter(...)` on its builder, like the archive here.
7. **Events are delivered in order, and delivery waits.** For each journey the executor creates one dispatcher, which hands every event, in order, to each reporter in turn. The default dispatcher calls the reporters in the order they were added: first those the workflow lists, in their order, then those the instance adds, which is why the log receives each event before the archive. The journey goes on only once every reporter has returned, so a slow reporter slows the journey.
8. **A reporter handles its own trouble.** A reporter should never fail: when it can't report an event, it deals with that itself, by dropping the event or keeping it to try later, and returns `Ok(())`. An `Err` that escapes a reporter aborts the journey with `reporter failed`. In the second order the archive was full when the charge's attempt started. Reporters receive each event in turn: the log had already written that event, as the output shows, and a reporter after the archive would never have received it. The journey was aborted before the charge ran, so the customer wasn't charged, and `journey_aborted`, the last event of an aborted journey, went to every reporter except the one that failed.
9. **A step passes a reporter's failure on with `?`.** The ship step propagates what `info` and `set_global_metadata` return with `?`: if a reporter fails on that event or that entry, the step ends there, and the journey is aborted whatever the step does next.
10. **The stream is never rewritten.** An event, once delivered, stays: an attempt whose contributions were never committed still has its events, and an aborted journey keeps every event emitted before the abort.
11. **Global metadata labels a journey for its reporters.** Global metadata is entries of a key and a value, kept apart from the data bag, that every reporter receives as soon as each is set. Before `journey_started`, the executor gives the reporters the journey ID, under the key `journey_id`, then the entries the instance was given with `.reporters_global_metadata(key, value)`, here the customer, without emitting any event. While the journey runs, a step, a hook or a service provider sets an entry with `set_global_metadata(key, value)` on its reporter, as the ship step does with the tracking number. Each value must implement `Value`, like data. Setting a key again replaces its value, and an entry stands even if the attempt that set it fails.
12. **Nothing in the journey can read it.** Global metadata never enters the data bag or the result, and no step, policy or input adapter can read it back: it exists only for reporters, so it never decides anything. That's why the ship step contributes the tracking number as well: a later step, or whoever reads the result, finds it in the data bag.
13. **A reporter keeps the entries it needs.** `Reporter::set_global_metadata` receives each entry, and does nothing unless the reporter implements it, which is why the archive has no code for it. The log keeps every entry in a `GlobalMetadata`, and reads the customer and the tracking number back with `get` and `downcast_ref`, to put on its lines. An entry set while the journey runs reaches every reporter first; then the event `global_metadata_set` names its key and, in its `source`, who set it, never its value, like every event. Its `source` is a `Source`, such as `Source::Step`, which the log matches to name the step. So the log already holds the tracking number when that event arrives, and labels that line and every line after it.
14. **Only the executor sets the journey ID, and a reporter that fails on an entry fails the journey.** A step, hook or service provider that sets `journey_id` aborts the journey with `reserved metadata key`, and an instance given `journey_id` as global metadata is refused by `run` with `Refusal::ReservedMetadataKey`. A reporter that fails on an entry set while the journey runs aborts it with `reporter failed`, as it would on an event. One that fails on the journey ID or on an entry the instance was given makes `run` refuse the journey with `Refusal::GlobalMetadata`, before any event.

## Remember

1. A step or a policy never logs what the executor already reports; it emits only what it alone knows.
2. Events carry the keys of your data, never its values; a reporter writes them in whatever form it needs.
3. A reporter handles its own trouble and returns `Ok(())`; a reporter that fails aborts the journey it reports.
4. A journey waits for every reporter to return before it goes on: a slow reporter slows every journey.
5. Global metadata is for reporters only: nothing in the journey reads it, so anything a step or policy needs goes in the data bag.
6. Only the executor sets `journey_id`; your code sets any other key.
