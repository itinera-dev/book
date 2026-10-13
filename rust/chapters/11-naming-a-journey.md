# Naming a journey

## What you want to do

Every event says which journey it belongs to, by the journey's ID. You want that ID to be one the shop's people already know, the order's number, so that the log and the desk say which order each line is about.

## The code

The workflow names each journey with a method of its `#[workflow]` block marked `#[id_generator]`. It reads the order's number from the journey's data, and returns it as the journey's ID. The providers are [chapter 7](07-giving-a-step-the-tools-it-needs.md)'s.

```example
name: ch11_naming_a_journey
region: id
```

The log now writes the journey ID at the start of every line:

```example
name: ch11_naming_a_journey
region: log
```

And the desk says which order failed:

```example
name: ch11_naming_a_journey
region: desk
```

The rest of the shop is [chapter 10](10-more-than-one-reporter.md)'s, and it runs Ann's and Gil's orders again. It prints:

```output
name: ch11_naming_a_journey
```

## How it works

1. **Every journey has a journey ID.** It is made once, when the instance is created, and every event of the journey carries it, as `event.journey_id`. The executor never makes one of its own, and you never hand one to it.
2. **By default, it is random.** Unless the workflow names its journeys, each journey's ID is a new random UUID, different on every run and meaningless to people, which is why the log of chapter 8 left it out.
3. **Naming journeys is the workflow's decision.** An ID generator is part of the workflow, like its steps and its providers: a method marked `#[id_generator]` in the `#[workflow]` block, which takes `&self` and the journey's data, a `&DataBag`, and returns `Result<String, Error>`. A workflow has at most one. **No step and no reporter decides what a journey is called.**
4. **It reads the initial data.** The generator runs when the instance is created, before any step, so the data bag holds only the initial data. `data.required("order")` reads the value under `order` as the type the method returns, a `String`, and returns an error if it is missing or of another type. Ann's journey becomes `A-1`, and Gil's `A-7`.
5. **An ID should name one journey.** An order's number names one order, so the events of each journey can be told apart from every other's.

Now the desk says "order A-7 failed at ship", and the log shows why: the main carrier won't take Gil's parcel of 60 kg. The shop has a backup carrier too, which takes heavier parcels than the main one. But the shipping gets one attempt, with the one carrier its provider makes for the journey, so a parcel the main carrier refuses is never offered to the other.

## Outcomes

We have shown how a workflow names its journeys, so every event says which order it is about.

| The usual way | With Itinera |
|---|---|
| A log line usually carries an order's number only where the code that writes it remembers to add it. | Every event carries the journey's ID, made once by the workflow from the order's data. |
