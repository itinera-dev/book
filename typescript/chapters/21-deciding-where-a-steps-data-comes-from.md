# Deciding where a step's data comes from

## What you want to do

The warehouse now describes each order's parcel as one value, its weight and its size, but the shipping asks only for a weight, and shouldn't have to know how the warehouse describes a parcel. You want the shop's workflow to take the weight out of the parcel and give it to the shipping.

## The code

The parcel is a plain object of the shop's own, described by an interface. Plain objects, lists and numbers are values, so the data bag can hold a parcel as it is. To read one back, the shop makes a type token for it with `objectOf` and `listOf`, from `itinera/value`, which check each property as the tokens they are given do:

```example
name: ch21_deciding_where_a_steps_data_comes_from
region: parcel
```

Each order's initial data holds the parcel, in place of the weight it held before:

```example
name: ch21_deciding_where_a_steps_data_comes_from
region: data
```

The shipping doesn't change at all: it is still [chapter 17](17-giving-up-a-step-early.md)'s. The workflow gives it the weight with an **input adapter**: a method of the workflow class marked `@inputAdapter({ steps: ["ship"] })`, from `itinera/workflow`. The executor asks it about each input of the shipping as the shipping is built, and tells it which one through its parameter marked `@key()`, from `itinera/workflow` too. It returns a `Supplied`, from the same module, made by one of two methods it inherits from `Workflow`. For `weight`, it reads the parcel from the data bag and supplies its weight with `this.supply(...)`. For any other input, it returns `this.fromDataBag()`, and that input is read from the data bag as before.

```example
name: ch21_deciding_where_a_steps_data_comes_from
region: adapter
```

The log is [chapter 18](18-doing-something-when-the-journey-ends.md)'s, and now also says which input an adapter supplied, to which step:

```example
name: ch21_deciding_where_a_steps_data_comes_from
region: supplied
```

The rest is [chapter 20](20-an-optional-input.md)'s shop, and it runs the same five orders. It prints:

```output
name: ch21_deciding_where_a_steps_data_comes_from
```

## How it works

1. **Without an adapter, an input is read from the data bag under its key.** Every input so far came from there: a step asking for `weight` got the value under `weight`. Now there is no `weight` in the data bag, only `parcel`.
2. **An input adapter satisfies what a step needs.** The shipping says what it needs, a weight in kg, and the adapter gives it exactly that, so the shipping never sees the parcel, its size or the warehouse's way of describing it. **A step says what it needs; the workflow decides where each value comes from.**
3. **It belongs to the workflow.** It is named after its method, `parcels`, unless its options give a `name`, and attached by name to the steps it serves, here only the shipping. Each workflow knows its own data, so the same shipping could run in a workflow whose orders carry their weight in another form.
4. **It is asked once for each input, as the step is built.** For each of the shipping's inputs, the adapter either supplies a value, which the shipping gets, or returns `this.fromDataBag()`. It supplies only `weight`, so `input_adapter_supplied ship gets weight from parcels` appears once in each attempt of the shipping, before it runs: twice for Gil, whose shipping has two attempts, and once for Kim, though her shipping then skips.
5. **It adapts the data's form, never the business.** Taking one field out of a value is one of its uses. Others are converting a value to the type a step asks for, and fetching data from somewhere else, because it is expensive to get or is kept elsewhere, only when a step needs it. Working out what an order costs is a rule of the business, so it stays in the pricing, a step of its own.
6. **It reads, and never writes.** An adapter contributes nothing: what it supplies goes to the step being built, never to the data bag. What it supplies is recorded as an `input_adapter_supplied` event, naming the step, the key and the adapter, as `event.step`, `event.key` and `event.adapter`, without the value itself.

The shop now charges, repairs, ships, audits and tells its customers, each in the place where it belongs. But now and then the website sends an order a second time, and the shop would charge it again, and book another parcel.

## Outcomes

We have shown how the workflow gives a step the data it asks for, in the form it asks for it.

| The usual way | With Itinera |
|---|---|
| A function that needs one field of a larger value often takes the whole value, and comes to depend on how its callers describe their data. | The shipping asks for a weight, and the workflow's input adapter takes it out of the warehouse's parcel. |
