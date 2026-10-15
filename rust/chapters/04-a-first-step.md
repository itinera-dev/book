# A first step

## What you want to do

The shop charges an order its price. You want that business rule written with Itinera: the charge, and nothing else.

## The code

In Itinera, a business rule is a **step**. In Rust, a step is a struct holding the data it needs, with a `run` method that does the work. The shop's bank is faked, so that it would print what it charges, once something runs the step.

```example
name: ch04_first_step
region: step
```

That is the whole step. The program declares it and does nothing else: nothing builds a `Charge`, and nothing calls its `run`, so it prints nothing. Rust notices too, and would warn that `Charge` is never constructed and `run` is never used; the program turns that warning off, with `#![allow(dead_code)]` at its top.

## How it works

1. **A step is a struct with a `run`, in an `impl` block marked `#[step]`.** `#[derive(Step)]` lets Itinera build the struct from its fields.
2. **A step asks for the data it needs.** Each field marked `#[input]` is an **input**, named after its field: `price: i64` asks for a number under the name `price`. The step says what it needs, and nothing about where it comes from.
3. **A step does its work in `run`.** `run` takes the step by value, `self`, so a `Charge` is used up by running it. Here it charges the bank, unless there is nothing to charge.
4. **A step says how it went, and nothing more.** `run` returns the step's **outcome** inside a `Result`, the one `itinera::error` defines, whose error type is `itinera`'s own: `Self::succeed()` when the charge went through, or `Self::fail("nothing-to-charge")`, a failure with its reason, when the price is 0 or less. Both give `Ok` with the outcome, and this step never returns an `Err`. **A step reports what happened, never what to do next.** The charge doesn't know which order it is charging for, what the shop does before it or after it, or what happens to an order whose charge failed.

So the charge is written, and it can't charge anything. Nothing builds it with an order's price, nothing calls `run`, and nothing does anything with its outcome. A step on its own is a business rule waiting to be used. The shop still needs a way to run it for an order.

## Outcomes

We have shown a business rule written as a step, which asks for its data and says how it went, and nothing else.

| The usual way | With Itinera |
|---|---|
| A charge function usually signals a failure with an error type of its own, so each caller learns to read it its own way. | Every step says how it went the same way: success, or a failure with a reason. |
| A function usually gets its data as parameters, from whichever caller calls it. | The step names each piece of data it needs as an input, in its own declaration, so what it reads is listed in one place. |
