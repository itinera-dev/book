# Running many journeys

## What you want to do

Orders come in all day, and the shop shouldn't make a new executor for each one. You want one executor that runs every order of the day, one journey after another, each starting from its own order and nothing else.

## The code

The shop is [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md)'s, unchanged. Only `main` changes: it makes one executor before the loop, and hands it each order's instance in turn.

```example
name: ch24_one_executor_for_every_order
region: run
```

It runs [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s five orders, none of which the order book has completed, and prints:

```output
name: ch24_one_executor_for_every_order
```

An instance runs one journey and no more. Handing the same instance to the executor a second time is a mistake, and Rust won't compile it:

```compile-error
name: ch24_instance_run_twice
```

## How it works

1. **An executor keeps nothing between journeys.** `LocalExecutor` runs a journey inside the call to its `run`, and saves nothing: the journey lives only as long as that call. When `run` returns, nothing of that journey remains in the executor, so the next journey starts from its own order alone. Gil's shipping needed the backup carrier and Jo's card was replaced, yet Kim's journey, run next by the same executor, starts like every other: `journey_started` with Kim's data, the lookup's first attempt, and the bank made anew for Kim's charge, as `service_provided` shows. Its log starts again at `#1`, since an event's sequence number counts the events of its own journey, never those of the journeys before it. **What one journey did never reaches another through the executor.**
2. **The loop still creates an instance for every order.** An instance is made for one journey, and runs once, as [chapter 5](05-a-first-journey.md) showed. Rust checks it when the program is built: `run` takes the instance by value, so using it again is the "use of moved value" error above, never something found while the shop is open. To run an order again, for example after the customer has corrected it, create a new instance with the corrected data.
3. **An executor runs one journey at a time.** `run` borrows the executor mutably, which is why it is declared with `let mut`: it can't be handed a second journey until the first has ended, and each call returns only once its journey has. That is why the loop waits for each order before it starts the next.

The executor now runs every order. But for Lee's order, `main` learns only that the journey failed. The log shows why, line by line, but the code that runs the shop has only the result, and reads from it only how the journey ended and its output.

## Outcomes

We have shown how one executor runs every order of the day, each journey starting from its own data.

| The usual way | With Itinera |
|---|---|
| Code that handles many requests often keeps state between them, in a cache or a field, and a request can be affected by the ones before it. | The executor keeps nothing between journeys: each starts from its own order, with services made for it. |
| Running the same request twice by mistake is usually found at run time, if at all. | An instance moves into `run`, so running it twice doesn't compile. |
