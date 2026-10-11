# Giving a step what it needs

## What you want to do

Some orders come with a discount and most do not. You want the charge to require the order's price, take the discount when there is one, and charge the difference, without knowing where either value comes from.

## The code

The charge declares two inputs: the price, which it cannot do without, and the discount, which it can.

```example
name: ch08_inputs
region: step
```

Each journey starts with its initial data: what the shop knows about the order. A discount that is not given leaves its key out. The second kind of order is a mistake on purpose: the shop forgot the price.

```example
name: ch08_inputs
region: data
```

The shop runs three journeys: one with a discount, one without, and one without a price. Its `report` function prints how each journey ended; chapter 22 shows how to read a result.

```example
name: ch08_inputs
region: run
```

It prints:

```output
name: ch08_inputs
```

Each field of the step marked `#[input]` is an input whose key is the field's name; `#[input("key")]` names another key. The field's type says whether it is optional: a field of type `Option<T>` is an optional input, `None` when the key is absent, and a field of any other type is required. On the initial data, a field of type `Option<T>` that is `None` leaves its key out of the data bag.

A value is any type that can be serialized, deserialized and cloned, such as `i64`, `String`, `bool` or a `Vec` of them. Types must match exactly: a price given as an `i32` and asked for as an `i64` is `wrong type`.

The step owns the values it receives, since each is its own clone, so changing one can never reach the data bag.

## How it works

1. **An input is data the step asks the workflow for.** The charge asks for a value by a key, `price`, and a type, a whole number. It never says where the value comes from: the workflow decides. Here it comes from the data bag, the journey's data, which starts as the initial data you gave the instance.
2. **A step receives only what it asks for.** The order's ID is in the data bag too, but the charge does not ask for it, so it never sees it. A step cannot look through the data bag, the workflow or the executor; it has its inputs and nothing more.
3. **Every input is resolved before the step is built.** The workflow finds each value first, then builds the step with all of them, so the step's code never checks whether its data arrived. A fresh step is built in this way for every attempt: each run of the step is an attempt, and chapter 13 shows a step run more than once.
4. **An optional input may be absent.** The discount is optional: for the order without one, the step is built with the discount absent, and it charges the full price.
5. **A required input with no value aborts the journey.** For the order without a price, the step is never built. The journey is aborted with `required data missing`, naming the key that has no value. Unlike a declined card, an aborted journey is not a business outcome: it means the workflow was given data that does not fit it, a mistake to fix, here in the code that starts journeys. An aborted journey has no output.
6. **A value of the wrong type aborts it too.** A price given as text aborts the journey with `wrong type`, whether the input is required or optional.
7. **What a step receives is its own.** Changing a value the step received changes nothing in the data bag. The only way to change the data bag is to contribute, which the next chapter shows.

Chapter 19 shows the other place an input can come from: an input adapter, part of the workflow, that works the value out when the step is built. The step does not change.

## Remember

1. A step declares every piece of data it needs, by key and type, as required or optional; where the data comes from is the workflow's decision.
2. Make an input optional only when the business rule has an answer for its absence.
3. A required input with no value, or any input of the wrong type, aborts the journey before the step is built.
4. Changing a value a step received never changes the data bag.
