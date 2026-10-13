# An optional input

## What you want to do

Some orders come with a discount, and most do not. You want the charge to take the order's discount when there is one, charge the price less the discount, and charge the full price when there is none.

## The code

The charge asks for the discount as an `Option<i64>`, an optional input, and charges the price less the discount, or the full price when the discount is `None`. It now also gives back what it charged, under `charged`.

```example
name: ch19_an_optional_input
region: charge
```

The audit of [chapter 15](15-reacting-to-what-a-step-did.md) wrote down the order's price, which is no longer what was charged. It now reads what the charge gave back:

```example
name: ch19_an_optional_input
region: audit
```

The initial data has a discount, as an `Option<i64>` too. Ann's order comes with a discount of 10, and the others with none.

```example
name: ch19_an_optional_input
region: data
```

The rest is [chapter 18](18-letting-policies-act-on-the-outside-world.md)'s shop, and it runs the same four orders. It prints:

```output
name: ch19_an_optional_input
```

## How it works

1. **An input's type says whether it is required.** An input of any type but `Option` is required, as every input so far has been. An input of type `Option<T>` is optional: the step is built with `Some` of the value when the data bag holds one, and with `None` when it holds none.
2. **Initial data leaves out what an order doesn't have.** A field of type `Option<T>` that is `None` adds no key to the data bag. Ann's journey starts with `discount` among its keys; Gil's, Jo's and Kim's don't.
3. **An absent optional input is recorded.** When the charge is built for an order without a discount, the executor records `optional_input_absent`, naming `discount`, as it does for a hook's request in [chapter 17](17-doing-something-when-the-journey-ends.md). It is recorded for each attempt the step is built for, which is why Jo's journey records it twice, once in each attempt of the charge, before the charge runs.
4. **The step decides what the absence means.** For the charge, no discount means the full price, so it uses `unwrap_or(0)`. The executor only says the value isn't there; it is the step's business to know what to do without it. **A step that can work without a value asks for it as optional; one that can't asks for it as required.**
5. **What the step charged is its contribution.** The bank charged Ann 30, the price less her discount, and the charge contributed it as `charged`. The audit reads it with `#[from_step] charged: i64`, so it writes down what was charged, whatever the charge worked it out from.

The charge now works out the amount itself: the shop's way of pricing, a price less a discount, has moved into the step that talks to the bank. A shop that prices orders differently, with a voucher or a price per kilogram, could not use this charge as it is.

## Outcomes

We have shown how a step asks for a value it can do without, and what it does when the value isn't there.

| The usual way | With Itinera |
|---|---|
| An optional value is often passed as a default, such as a discount of 0, so the code cannot tell "no discount" from "a discount of nothing". | An optional input is `None` when the data bag has no value, and the executor records that it was absent, by key. |
