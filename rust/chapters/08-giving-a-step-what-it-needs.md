# Giving a step what it needs

## What you want to do

Some orders come with a discount and most do not. You want the charge to require the order's price, take the discount when there is one, and charge the difference, without knowing where either value comes from.

## The code

The charge declares two inputs: the price, which it can't do without, and the discount, which it can.

```example
name: ch08_inputs
region: step
```

Each field marked `#[input]` is an input, and its key is the field's name; `#[input("key")]` names another key. The field's type says whether the input is required. The price is an `i64`, so the charge can't be built without one. The discount is an `Option<i64>`, so it is optional, and it is `None` when the order has none.

Each journey starts with its initial data: what the shop knows about the order. Deriving `InitialData` turns each field of a struct into a key of the data bag, and a field of type `Option<T>` that is `None` leaves its key out, so an order without a discount simply has no `discount`. The second struct is a mistake on purpose: the shop forgot the price.

```example
name: ch08_inputs
region: data
```

The shop runs three journeys: one with a discount, one without, and one without a price. Its `report` function prints how each journey ended; [chapter 22](22-reading-the-result.md) shows how to read a result.

```example
name: ch08_inputs
region: run
```

It prints:

```output
name: ch08_inputs
```

## How it works

1. **An input is data the step asks the workflow for.** The charge asks for a value by a key, `price`, and a type, `i64`. It never says where the value comes from: the workflow decides. Here it comes from the data bag, the journey's data, which starts as the initial data you gave the instance.
2. **A step receives only what it asks for.** The order's ID is in the data bag too, but the charge doesn't ask for it, so it never sees it. A step can't look through the data bag, the workflow or the executor; it has its fields and nothing more.
3. **Every input is resolved before the step is built.** The workflow finds each value first, then builds the struct with all of them, so `run` never checks whether its data arrived: by the time it runs, `self.price` is there. A fresh step is built in this way for every attempt: each run of the step is an attempt, and [chapter 13](13-trying-again.md) shows a step run more than once.
4. **An optional input may be absent.** For the order without a discount, the step is built with `discount` set to `None`, and it charges the full price.
5. **A required input with no value aborts the journey.** For the order without a price, the step is never built. The journey is aborted with `required data missing`, naming the key that has no value. Unlike a declined card, an aborted journey isn't a business outcome: it means the workflow was given data that doesn't fit it, a mistake to fix, here in the code that starts journeys. An aborted journey has no output.
6. **A value of the wrong type aborts it too.** A value is any type that can be serialized, deserialized and cloned, such as `i64`, `String`, `bool` or a `Vec` of them, and the type has to match exactly. A price given as a `String`, or even as an `i32`, when the step asks for an `i64` aborts the journey with `wrong type`, whether the input is required or optional.
7. **What a step receives is its own.** Each field holds its own clone of the value, owned by the step, so changing it changes nothing in the data bag. The only way to change the data bag is to contribute, which the next chapter shows.

[Chapter 19](19-deciding-where-a-steps-data-comes-from.md) shows the other place an input can come from: an input adapter, part of the workflow, that works the value out when the step is built. The step doesn't change.

## Remember

1. A step declares every piece of data it needs, by key and type, as required or optional; where the data comes from is the workflow's decision.
2. Make an input an `Option<T>` only when the business rule has an answer for its absence.
3. A required input with no value, or any input of the wrong type, aborts the journey before the step is built.
4. Changing a value a step received never changes the data bag.
