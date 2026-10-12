# Deciding where a step's data comes from

## What you want to do

The charge step charges an amount. The shop knows a price and, sometimes, a discount, and the amount to charge is the price less the discount. You want that worked out by the shop's workflow, so the charge step charges an amount without knowing where it came from, and could be reused by a shop that prices orders differently.

## The code

The charge step declares the two inputs it needs: the order, and the amount. There is no price or discount in it.

```example
name: ch19_input_adapter
region: step
```

The workflow declares an input adapter, `pricing`, attached to the charge. In Rust it's a method of the workflow, in an `impl` block marked `#[workflow]`, with `#[input_adapter(steps = ["charge"])]`. It is asked for each input of the charge as the step is built, and learns which one through its `#[key]` parameter. For `amount`, it reads the price and the optional discount from the journey's data and returns the difference, wrapped in an `AnyValue`. For any other input it returns `Ok(None)`, so that input is read from the data bag as usual.

```example
name: ch19_input_adapter
region: workflow
```

Three orders go through, each at a price of 40: one with a discount of 5, one without a discount, and one with a discount of 40. For each, the shop prints how the journey ended and, when something was charged, the receipt and the amount.

```example
name: ch19_input_adapter
region: run
```

It prints:

```output
name: ch19_input_adapter
```

## How it works

1. **Without an adapter, an input is read from the data bag under its key.** Until now, every step's inputs came from there: a step that needs `price` gets the value under `price`.
2. **An input adapter belongs to the workflow.** It has a name, which is the method's name unless `name = "..."` gives another, and is attached to one or more of the workflow's steps by their names; a step has at most one. Each workflow knows how to adapt its own data, so the same charge step can run in workflows that work out the amount in different ways. Since the adapter takes `&self`, it can also use the workflow's own fields.
3. **It is asked once for each input, as the step is built.** For each input, it either returns a value, `Ok(Some(...))`, which the step gets, or returns `Ok(None)`, and the input is read from the data bag. The order here is read from the data bag; the amount comes from the adapter. For A-3, the adapter supplies 0, and the charge fails with `nothing-to-charge` without knowing there ever was a discount.
4. **It reads, and never writes.** Each parameter after `&self` says what the adapter asks for. It may ask for data from the workflow with `#[from_workflow]`, which it reads from the data bag, never through another adapter. It may also ask for the key being resolved with `#[key]`, the name of the step being built with `#[step_name]` and the journey ID with `#[journey_id]`, or for read-only access to the whole data bag with `#[data_bag]`, of type `DataBagAccess<'_>`. It contributes nothing and asks for no role.
5. **What it supplies is recorded.** Each value it supplies is an `InputAdapterSupplied` event, naming the step, the key and the adapter. The value itself isn't in the event: the executor's events carry keys, never data values. [Chapter 23](23-seeing-what-happened.md) shows how to read events.
6. **Its mistakes stop the journey.** An adapter that returns `Err` stops the step from being built, and the journey is aborted with `step could not be built`. Since an `AnyValue` can hold any value, the compiler can't check that it holds the type the step asks for: a value of the wrong type is caught only during the journey, which is aborted with `wrong type`. Attaching two adapters to one step, or an adapter to a step the workflow doesn't have, is refused before any journey; with the macros, it doesn't compile.
7. **Only steps are adapted.** Hooks read data from the workflow straight from the data bag, since a policy acts on behalf of the workflow and needs no adapting.

## Remember

1. A step declares what it needs; the workflow decides where it comes from.
2. A step has at most one input adapter, and an adapter belongs to the workflow, never to the step.
3. An adapter that returns `Ok(None)` for an input leaves it to be read from the data bag.
4. An adapter only supplies values: it never changes the data bag.
5. Hooks never go through input adapters.
