# Naming a journey

## What you want to do

Every event says who decided what, and when, and which journey it belongs to, by the journey's ID. You want that ID to be one the shop's people already know, the order's number, so that every decision in the log and at the desk is tied to its order.

## The code

The workflow names each journey with a method of its class marked `@idGenerator()`, from `itinera/workflow`. It reads the order's number from the journey's data, and returns it as the journey's ID. The providers are [chapter 8](08-seeing-what-happened.md)'s, and `Payments`, `Carrier` and their tokens are those [chapter 7](07-giving-a-step-the-tools-it-needs.md) shows.

```example
name: ch11_naming_a_journey
region: id
```

The log now writes the workflow's name and the journey ID at the start of every line:

```example
name: ch11_naming_a_journey
region: log
```

`detail` is the function [chapter 8](08-seeing-what-happened.md) shows, with the case for the step's message that [chapter 9](09-saying-what-only-a-step-knows.md) added.

And the desk says which order failed:

```example
name: ch11_naming_a_journey
region: desk
```

The steps the workflow lists are the shipping of [chapter 9](09-saying-what-only-a-step-knows.md) and the charge of [chapter 7](07-giving-a-step-the-tools-it-needs.md), and the shop runs Ann's and Gil's orders again. It prints:

```output
name: ch11_naming_a_journey
```

## How it works

1. **Every journey has a journey ID.** It is made once, when the instance is created, and every event of the journey carries it, as `event.journeyId`. The executor never makes one of its own, and you never hand one to it.
2. **By default, it is random.** Unless the workflow names its journeys, each journey's ID is a new random UUID, different on every run and meaningless to people, which is why the log of chapter 8 left it out.
3. **Naming journeys is the workflow's decision.** An ID generator is part of the workflow, like its steps and its providers: a method of the workflow's class marked `@idGenerator()`. Its one parameter, marked `@dataBag()` and typed `DataBag`, both from `itinera/journey`, asks for the journey's data, and it returns the ID as a `string`, at once, never as a promise. A workflow has at most one. **No step and no reporter decides what a journey is called.**
4. **It reads the initial data.** The generator runs when the instance is created, before any step, so the data bag holds only the initial data. `data.read("order", string)` reads the value under `order`, checked by the type token `string`, and `.option()` gives it as an `Option`: `Some` of it when it is there and is a string, `None` when it is missing or of another type. `expect` gives the value, or throws an error with its message when there is none. Ann's journey becomes `A-1`, and Gil's `A-7`.
5. **The workflow's name says what it does; the ID says which journey.** Every event also carries the name of its workflow, as `event.workflow`: the one in `@workflow({ name: "orders", ... })`, the same for every journey of the workflow. The name should say what the workflow does, as `orders` says it handles the shop's orders, and the ID should name one journey of it, as an order's number names one order, so the events of each journey can be told apart from every other's.

Now the desk says "order A-7 failed at ship", and the log shows why: the main carrier won't take Gil's parcel of 60 kg. The shop has a backup carrier too, which takes heavier parcels than the main one. But the shipping gets one attempt, with the one carrier its provider makes for the journey, so a parcel the main carrier refuses is never offered to the other.

## Outcomes

We have shown how a workflow names its journeys, so every event says which order it is about.

| The usual way | With Itinera |
|---|---|
| A log line usually carries an order's number only where the code that writes it remembers to add it. | Every event, and so every decision, carries the journey's ID, made once by the workflow from the order's data. |
