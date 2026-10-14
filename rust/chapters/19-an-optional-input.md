# An optional input

## What you want to do

Some orders come with a discount, and most do not, so you want the shop to charge the price less the discount, or the full price when there is none. Working out what an order costs and charging a card are two rules of the business, so you want each in a step of its own: the charge should charge an amount, and know nothing of prices or discounts.

## The code

The pricing is a new step. It asks for the order's price, and for the discount as an `Option<i64>`, an optional input. It gives back the price less the discount, or the full price when the discount is `None`, under `amount`.

```example
name: ch19_an_optional_input
region: price
```

The charge now asks for that `amount`, and charges it. There is no price or discount in it.

```example
name: ch19_an_optional_input
region: charge
```

The workflow lists the pricing first, under the name `price`, so the charge reads the amount it gave back:

```example
name: ch19_an_optional_input
region: workflow
```

The audit of [chapter 15](15-reacting-to-what-a-step-did.md) wrote down the order's price, which is no longer what was charged. It now asks for the amount, from the data bag:

```example
name: ch19_an_optional_input
region: audit
```

The initial data has a discount, as an `Option<i64>` too. Ann's order comes with a discount of 10, and the others with none.

```example
name: ch19_an_optional_input
region: data
```

The rest is [chapter 18](18-letting-policies-act-on-the-outside-world.md)'s shop, and it runs the same five orders. It prints:

```output
name: ch19_an_optional_input
```

## How it works

1. **An input's type says whether it is required.** An input of any type but `Option` is required, as every input so far has been. An input of type `Option<T>` is optional: the step is built with `Some` of the value when the data bag holds one, and with `None` when it holds none.
2. **Initial data leaves out what an order doesn't have.** A field of type `Option<T>` that is `None` adds no key to the data bag. Ann's journey starts with `discount` among its keys; Gil's, Jo's, Kim's and Lee's don't.
3. **An absent optional input is recorded.** When the pricing is built for an order without a discount, the executor records `optional_input_absent`, naming `discount`, as it does for a hook's request in [chapter 17](17-doing-something-when-the-journey-ends.md). The pricing has one attempt, so each journey without a discount records it once.
4. **The step decides what the absence means.** For the pricing, no discount means the full price, so it uses `unwrap_or(0)`. The executor only says the value isn't there; it is the step's business to know what to do without it. **A step that can work without a value asks for it as optional; one that can't asks for it as required.**
5. **Each rule of the business is a step.** The bank charged Ann 30, the price less her discount, but the charge never saw either: it charged the `amount` the pricing contributed, read from the data bag like any input. A shop that priced its orders another way, with a voucher or a price per kilogram, would change the pricing, and use the same charge. **A step holds one rule of the business; two rules are two steps, and the workflow puts them in order.**
6. **The audit writes down what was charged.** It reads `amount` with `#[from_workflow]`, since the pricing contributed it and the charge didn't, so it writes down what was charged, whatever it was worked out from.

The warehouse is changing how it describes parcels, though. From now on, each order comes with its parcel as one value, holding its weight and its size, and the shipping, which asks only for a weight, can't read that.

## Outcomes

We have shown how a step asks for a value it can do without, and how the pricing became a step of its own.

| The usual way | With Itinera |
|---|---|
| An optional value is often passed as a default, such as a discount of 0, so the code cannot tell "no discount" from "a discount of nothing". | An optional input is `None` when the data bag has no value, and the executor records that it was absent, by key. |
| The code that charges a card often works out the amount too, so a new way of pricing changes the charging. | The pricing and the charge are two steps, and the charge only charges the amount it is given. |
