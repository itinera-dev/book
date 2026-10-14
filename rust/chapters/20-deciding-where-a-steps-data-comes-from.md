# Deciding where a step's data comes from

## What you want to do

The charge works out a price less a discount, which is how this shop prices its orders, not what charging is. You want the charge to charge an amount and know nothing of prices or discounts, and the shop's workflow to work the amount out, so that a shop pricing its orders another way could use the same charge.

## The code

The charge now asks for an `amount`, and charges it. There is no price or discount in it.

```example
name: ch20_deciding_where_a_steps_data_comes_from
region: charge
```

The workflow works the amount out with an **input adapter**: a method of its `#[workflow]` block marked `#[input_adapter(steps = ["charge"])]`. The executor asks it about each input of the charge as the charge is built, and tells it which one with its parameter marked `#[key]`. It returns `Ok` of an `Option<AnyValue>`, where `AnyValue` holds a value of any type. For `amount`, it reads the price and the optional discount from the data bag, and supplies the difference with `Self::supply(...)`, which is `Some` of it. For any other input, it returns `Self::from_data_bag()`, which is `None`, and that input is read from the data bag as before.

```example
name: ch20_deciding_where_a_steps_data_comes_from
region: adapter
```

The log says which input an adapter supplied, to which step:

```example
name: ch20_deciding_where_a_steps_data_comes_from
region: supplied
```

The rest is [chapter 19](19-an-optional-input.md)'s shop, and it runs the same five orders. It prints:

```output
name: ch20_deciding_where_a_steps_data_comes_from
```

## How it works

1. **Without an adapter, an input is read from the data bag under its key.** Every input so far came from there: a step asking for `price` got the value under `price`.
2. **An input adapter belongs to the workflow.** It is named after its method, `pricing`, and attached by name to the steps it serves, here only the charge. Each workflow knows how its own data becomes what its steps ask for, so the same charge could run in a workflow that works the amount out another way. **A step says what it needs; the workflow decides where each value comes from.**
3. **It is asked once for each input, as the step is built.** For each of the charge's inputs, the adapter either supplies a value, which the charge gets, or returns `Self::from_data_bag()`. The charge has four inputs, so the adapter is asked four times for each attempt, and supplies only `amount`: `input_adapter_supplied charge gets amount from pricing` appears once in each attempt, before the charge runs. The bank charges Ann 30, her price less her discount, as before.
4. **Its requests are resolved each time it is asked.** The adapter asks for the price and the optional discount with `#[from_workflow]`, as a hook does, and the executor resolves them for every call. Orders without a discount record `optional_input_absent discount` once for each of the four calls, which is why four of them come before Gil's charge runs.
5. **It reads, and never writes.** An adapter contributes nothing: what it supplies goes to the step being built, never to the data bag. That is why the charge still contributes `charged`, which the audit reads.
6. **What it supplies is recorded, never its value.** Each value it supplies is an `input_adapter_supplied` event, naming the step, the key and the adapter, like every event of the executor, without the value itself.

The shop now charges, repairs, ships, audits and tells its customers, each in the place where it belongs. But look at how `main` runs the orders: it makes a new executor for each one, and waits for each journey to end before it starts the next.

## Outcomes

We have shown how the workflow decides where a step's data comes from, so the step only says what it needs.

| The usual way | With Itinera |
|---|---|
| A function that charges an amount usually works the amount out itself, or relies on every caller to work it out the same way. | The charge asks for an amount, and the workflow's input adapter works it out from the order's price and discount. |
