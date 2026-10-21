# An optional input

## What you want to do

Some orders come with a discount, and most do not, so you want the shop to charge the price less the discount, or the full price when there is none. Working out what an order costs and charging a card are two rules of the business, so you want each in a step of its own: the charge should charge an amount, and know nothing of prices or discounts.

## The code

The pricing is a new step. It asks for the order's price, and for the discount with the type token `optional(integer)`, as an optional input, which it holds as an `Option<number>`. It gives back the price less the discount, or the full price when there is no discount, under `amount`.

```example
name: ch20_an_optional_input
region: price
```

The charge now asks for that `amount`, and charges it. There is no price or discount in it.

```example
name: ch20_an_optional_input
region: charge
```

The payment client it asks for is [chapter 14](14-repairing-then-trying-once-more.md)'s `Payments`, with [chapter 7](07-giving-a-step-the-tools-it-needs.md)'s token `PAYMENTS`.

The workflow lists the pricing first, under the name `price`, so the charge reads the amount it gave back:

```example
name: ch20_an_optional_input
region: workflow
```

The other steps, policies and reporters it names are as earlier chapters show them: `RenewCard` in [chapter 14](14-repairing-then-trying-once-more.md), the shipping, `Outage` and `HeavyParcels` in [chapter 17](17-giving-up-a-step-early.md), the log in [chapter 18](18-doing-something-when-the-journey-ends.md), `Notify` and the role `Messages` in [chapter 19](19-letting-policies-act-on-the-outside-world.md) and the desk in [chapter 11](11-naming-a-journey.md). The providers are [chapter 14](14-repairing-then-trying-once-more.md)'s, with the tokens `PAYMENTS` and `CARRIER` of [chapter 7](07-giving-a-step-the-tools-it-needs.md).

The audit of [chapter 15](15-reacting-to-what-a-step-did.md) wrote down the order's price, which is no longer what was charged. It now asks for the amount, from the data bag:

```example
name: ch20_an_optional_input
region: audit
```

Ann's order comes with a discount of 10, and the others with none, so their data has no `discount` at all:

```example
name: ch20_an_optional_input
region: data
```

The rest is [chapter 19](19-letting-policies-act-on-the-outside-world.md)'s shop, and it runs the same five orders. It prints:

```output
name: ch20_an_optional_input
```

## How it works

1. **An input's type token says whether it is required.** An input whose token is anything but `optional(...)` is required, as every input so far has been. An input whose token is `optional(integer)`, from `itinera/value`, the wrapper the output uses in [chapter 6](06-passing-data-on.md), is optional: the step is built with `Some` of the value when the data bag holds one, and with `None` when it holds none, and its parameter's type is `Option<number>`, from `itinera/option`.
2. **Initial data leaves out what an order doesn't have.** An order without a discount has no `discount` in its object, so no key is added to the data bag. Ann's journey starts with `discount` among its keys; Gil's, Jo's, Kim's and Lee's don't.
3. **An absent optional input is recorded.** When the pricing is built for an order without a discount, the executor records `optional_input_absent`, naming `discount`, as it does for a hook's request in [chapter 18](18-doing-something-when-the-journey-ends.md). The pricing has one attempt, so each journey without a discount records it once.
4. **The step decides what the absence means.** For the pricing, no discount means the full price, so it uses `unwrapOr(0)`. The executor only says the value isn't there; it is the step's business to know what to do without it. **A step that can work without a value asks for it as optional; one that can't asks for it as required.**
5. **Each rule of the business is a step.** The bank charged Ann 30, the price less her discount, but the charge never saw either: it charged the `amount` the pricing contributed, read from the data bag like any input. A shop that priced its orders another way, with a voucher or a price per kilogram, would change the pricing, and use the same charge. **A step holds one rule of the business; two rules are two steps, and the workflow puts them in order.**
6. **The audit writes down what was charged.** It reads `amount` with `@fromWorkflow`, since the pricing contributed it and the charge didn't, so it writes down what was charged, whatever it was worked out from.

The warehouse is changing how it describes parcels, though. From now on, each order comes with its parcel as one value, holding its weight and its size, and the shipping, which asks only for a weight, can't read that.

## Outcomes

We have shown how a step asks for a value it can do without, and how the pricing became a step of its own.

| The usual way | With Itinera |
|---|---|
| An optional value is often passed as a default, such as a discount of 0, so the code cannot tell "no discount" from "a discount of nothing". | An optional input is `None` when the data bag has no value, and the executor records that it was absent, by key. |
| The code that charges a card often works out the amount too, so a new way of pricing changes the charging. | The pricing and the charge are two steps, and the charge only charges the amount it is given. |
