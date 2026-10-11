# Deciding where a step's data comes from

## What you want to do

The charge step charges an amount. The shop knows a price and, sometimes, a discount, and the amount to charge is the price less the discount. You want that worked out by the shop's workflow, so the charge step charges an amount without knowing where it came from, and could be reused by a shop that prices orders differently.

## The code

The charge step declares the two inputs it needs: the order, and the amount. There is no price or discount in it.

```example
name: ch19_input_adapter
region: step
```

The workflow declares an input adapter, `pricing`, attached to the charge. It is asked for each input of the charge as the step is built. For `amount`, it reads the price and the optional discount from the journey's data and returns the difference. For any other input it returns nothing, so that input is read from the data bag as usual.

```example
name: ch19_input_adapter
region: workflow
```

Two orders go through, one with a discount and one without. A small reporter prints each input the adapter supplied.

```example
name: ch19_input_adapter
region: run
```

It prints:

```output
name: ch19_input_adapter
```

```note
name: ch19_input_adapter
```

## How it works

1. **Without an adapter, an input is read from the data bag under its key.** Until now, every step's inputs came from there: a step that needs `price` gets the value under `price`.
2. **An input adapter belongs to the workflow.** It has a name, and is attached to one or more of the workflow's steps; a step has at most one. Each workflow knows how to adapt its own data, so the same charge step can run in workflows that work out the amount in different ways.
3. **It is asked once for each input, as the step is built.** For each input, it either returns a value, which the step gets, or returns nothing, and the input is read from the data bag. The order here is read from the data bag; the amount comes from the adapter.
4. **It reads, and never writes.** An adapter may ask for data from the workflow, which it reads from the data bag, never through another adapter. It may also ask for the name of the step being built, the key being resolved and the journey ID, or for read-only access to the whole data bag. It contributes nothing and asks for no role.
5. **What it supplies is recorded.** Each value it supplies is an `input_adapter_supplied` event, naming the step, the key and the adapter. The value itself is not in the event: the executor's events carry keys, never data values.
6. **Its mistakes stop the journey.** An adapter that fails stops the step from being built, and the journey is aborted with `step could not be built`. A value of the wrong type aborts it with `wrong type`. Attaching two adapters to one step, or an adapter to a step the workflow does not have, is refused before any journey.
7. **Only steps are adapted.** Hooks read data from the workflow straight from the data bag, since a policy acts on behalf of the workflow and needs no adapting.

Try giving the first order a discount of 40: the adapter supplies 0, the charge fails with `nothing-to-charge` without knowing there ever was a discount, and the journey fails. The failed journey's data has no amount charged and no receipt, so the program stops with an error when it reads them back.

## Remember

1. A step declares what it needs; the workflow decides where it comes from.
2. A step has at most one input adapter, and an adapter belongs to the workflow, never to the step.
3. An adapter that returns nothing for an input leaves it to be read from the data bag.
4. An adapter only supplies values: it never changes the data bag.
5. Hooks never go through input adapters.
