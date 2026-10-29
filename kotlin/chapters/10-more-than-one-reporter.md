# More than one reporter

## What you want to do

The shop's customer service desk wants to hear about every order that failed, and where it failed, and about nothing else. You want the desk to read the same record of who decided what as the log, for a purpose of its own, without changing the log or any step.

## The code

The desk is a second reporter. Of all the events it receives, it keeps only `journey_failed`, and ignores the rest.

```example
name: ch10_more_than_one_reporter
region: desk
```

The workflow lists both reporters, the log first:

```example
name: ch10_more_than_one_reporter
region: workflow
```

The log is [chapter 8](08-seeing-what-happened.md)'s, with the branch for the step's message that [chapter 9](09-saying-what-only-a-step-knows.md) added. The shipping is chapter 9's, the charge is [chapter 7](07-giving-a-step-the-tools-it-needs.md)'s, `Payments` and `Carrier` are the interfaces chapter 7 shows, and the shop runs Ann's and Gil's orders again. It prints:

```output
name: ch10_more_than_one_reporter
```

## How it works

1. **Every reporter receives every event.** For each journey, the executor makes one **dispatcher**, which delivers the journey's events to its reporters: it hands each event, in order, to each reporter in turn. One stream fans out to every reporter, and each decides on its own what to do with it. The log writes a line for every event; the desk ignores all but one kind, which `event is JourneyFailed` tells apart, and inside the `if`, the compiler treats the event as a `JourneyFailed`, with its `step`. Ann's journey doesn't fail, so the desk says nothing about it, though it received all eleven of her journey's events.
2. **The order of the list is the order of delivery.** The dispatcher Itinera uses unless told otherwise calls the reporters in the order the workflow lists them. That's why, for Gil's journey, the log writes `journey_failed` before the desk hears of it.
3. **The journey waits for every reporter.** An event is delivered to every reporter before the journey goes on, so the desk had Gil's failure before `run` returned. A reporter that takes its time slows every journey it reports, so a reporter that sends events to another system should be quick about it.
4. **Adding a reporter changes nothing else.** The desk was added with one more class in the workflow's annotation. No step, no service and no other reporter changed, and the log prints what it printed before, but for the program's package, `ch10_more_than_one_reporter`, at the start of each implementation. **Steps report what happened; reporters decide who hears about it.**
5. **Each reporter is its own.** Every journey gets one of each listed reporter, made anew with its constructor, as [chapter 8](08-seeing-what-happened.md) said of the log. A reporter can keep what it receives in its own properties, and nothing it keeps is shared with another reporter or another journey.

The desk now hears about every failed order, but look at what it says: "an order failed at ship". Which order? Each event carries the journey's ID, but nothing names the journeys, so every journey gets a new random one, meaningless to the people at the desk.

## Outcomes

We have shown how several reporters each get every event of a journey, and keep what they need.

| The usual way | With Itinera |
|---|---|
| A new audience for what the code does, such as a support desk, usually means adding a call to it wherever something it cares about happens. | A new audience is one more reporter listed on the workflow, and it receives every event without changing a step. |
| A second log is often written apart from the first, so the two can tell different stories about the same order. | Every reporter receives the same events in the same order, so the log and the desk tell one story of who decided what. |
