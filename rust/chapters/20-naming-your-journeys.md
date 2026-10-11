# Naming your journeys

## What you want to do

When a customer calls about order A-1, you want to find that order's journey at once: in the logs, in the events and in the result. You want each journey named after its order, instead of a random identifier that nobody can connect to an order.

## The code

The workflow declares how its journey IDs are made, with a method marked `#[id_generator]` in its `#[workflow]` block. It takes the initial data as a `&DataBag`, reads the order and names the journey after it. `DataBag::required` reads a key as a type, here a `String`, and fails when the key is absent or holds another type; the `?` passes that failure on.

```example
name: ch20_journey_ids
region: workflow
```

Each instance gets its ID when it is created, before it is run, and you read it with `journey_id()`, from the `WorkflowInstance` trait. A small reporter prints the ID that every event carries, and the result carries it in its `journey_id` field. The last instance is created from data without an order, so no ID can be made from it.

```example
name: ch20_journey_ids
region: run
```

It prints:

```output
name: ch20_journey_ids
```

## How it works

1. **Every journey has a journey ID.** It's made once, when the workflow instance is created, after the initial data and before the reporters, which may receive it. The executor uses it and never makes one of its own, and you never pass one to the executor.
2. **By default, it is random.** Unless the workflow says otherwise, the ID is a random UUID, different for every journey and meaningless to people.
3. **The workflow may make its own.** Making the ID is part of the workflow, like its steps and policies, and the generator may read the instance's initial data. Here, the order `A-1` becomes the journey `order-A-1`. The generator takes `&self`, so it may also use the workflow's own fields, such as a shop's prefix. A workflow has at most one generator: two don't compile.
4. **The ID travels everywhere.** Every event carries it, so a reporter can file a journey's events under it, and the result carries it, so you can tell which order a result belongs to, whether the journey succeeded, failed or was aborted.
5. **A journey that can't be named doesn't start.** The generator returns `Result<String, Error>`. If the ID can't be made, as for the order with no ID, the generator returns `Err`, and `create()` fails with `InstanceError::JourneyId`, whose message is the one printed above, before any executor is involved. There is no journey, no event and no result.
6. **One ID names one journey.** If the same order may go through the workflow twice, for example after a customer corrects their address, make sure the two journeys get different IDs, or their events can't be told apart.

## Remember

1. The journey ID is made when the instance is created, never by the executor.
2. Without an `#[id_generator]` of its own, a workflow gives each journey a random ID.
3. A workflow's own way of making IDs may read the initial data, since nothing else of the journey exists yet.
4. Every event and every result carries the journey ID.
5. Give each journey an ID of its own, even when two journeys share an order.
