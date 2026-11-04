# Saying what only a step knows

## What you want to do

The events say who decided what, and when, but one thing only the shipping knows: that the parcel has left. You want the shipping to say so in the journey's own stream of events, under its own name, so that the record of who did what is complete and in order, instead of printing it on the side.

## The code

The shipping asks for its **step reporter**, a parameter of `run` of type `StepReporter`, from `org.itineradev.itinera.step`, next to its service and its contributor. Where it used to print, it now calls `info` with a message.

```example
name: ch09_saying_what_only_a_step_knows
region: ship
```

The interface `Carrier` it asks for is unchanged since [chapter 7](07-giving-a-step-the-tools-it-needs.md) showed it.

The log is [chapter 8](08-seeing-what-happened.md)'s, with one more branch in the `when` of its `detail`, for the step's message:

```example
name: ch09_saying_what_only_a_step_knows
region: step_info
```

The rest of the shop is unchanged, and runs Ann's and Gil's orders again. It prints:

```output
name: ch09_saying_what_only_a_step_knows
```

## How it works

1. **A step can add events of its own.** `info` emits a `step_info` event with the step's message, which says whatever the step chooses, here the receipt it put in the parcel. The step reporter has two more such methods, `warning` and `error`, which emit `step_warning` and `step_error`, for something that went wrong without being the step's outcome. Each also takes some data after the message, if you give it, which the event carries too: a value, captured when the event is emitted.
2. **A step's event is part of the journey's stream.** The executor stamps it with the step and the attempt it comes from, so the stream says who said it, and gives it the next number, saying when, so in Ann's journey `step_info` is event 8, between the shipping's `service_provided` and its `step_succeeded`. The log receives it like any other event, and a reporter tells it apart by its kind: in the branch `is StepInfo ->` of the log's `when`, the event has the step's `message`.
3. **It is delivered before the call returns.** When `info` returns, every reporter has received the event, so the stream keeps the order in which things happened, and the shipping says how it went only after the log has the parcel leaving.
4. **The step reporter belongs to the attempt.** Like the contributor and the services, it is lent to the step for one attempt, as a parameter of `run`, and serves no other. Its type alone says what it is, so, like the contributor, it needs no annotation.
5. **A step reports only what it alone knows.** The shipping doesn't emit events for its start, its outcome or its contribution: the executor already records all of that, the same way for every step. **A step never emits what the executor already records.** Its own events are for what happens inside it, such as the parcel leaving, which nothing outside the step can see. Gil's shipping never reaches that line, so his journey has no `step_info`.

The shop's log now holds everything that happened to an order, in order. The shop's customer service wants something else from the same events: the orders that failed, and nothing more. The log is not the place for that, and changing it to suit the desk would change what the log is for.

## Outcomes

We have shown how a step adds what only it knows to the journey's events.

| The usual way | With Itinera |
|---|---|
| A function usually logs its own progress in words of its own, apart from any record of what was decided about it, so the two must be read together. | What a step says goes into the same stream as every decision about it, stamped with the step and attempt that said it. |
