# A first step

## What you want to do

The shop charges an order its price. You want that business rule written with Itinera: the charge, and nothing else.

## The code

In Itinera, a business rule is a **step**. In TypeScript, a step is a class extending `Step`, whose constructor takes the data the step needs, and whose `run` method does the work. The shop's bank is faked, so that it would print what it charges, once something runs the step.

```example
name: ch04_first_step
region: step
```

That is the whole step. The program declares it and does nothing else: nothing builds a `Charge`, and nothing calls its `run`, so it prints nothing.

## How it works

1. **A step is a class extending `Step`, marked `@step()`.** `Step` comes from `itinera/step`, and so does the decorator `@step()`, which tells Itinera that the class is a step and records what it asks for. It does that once, when the class is defined; it builds no `Charge` and runs nothing.
2. **A step asks for the data it needs.** Each parameter of the constructor marked `@input(...)` is an **input**: `@input("price", integer)` asks for a number under the name `price`. The constructor's `private readonly price: number` is TypeScript's shorthand for a parameter that is also a field, so `run` reads it as `this.price`. The step says what it needs, and nothing about where it comes from.
3. **An input names the type of its value.** The type the compiler records for the running program says only that `price` is a `number`, so each input also names a **type token**, from `itinera/value`, which Itinera checks the value against: `integer` for a whole number, and `string` for a string. The parameter's type is what the compiler checks, and the token what the running program checks.
4. **A step does its work in `run`.** `override` says that this `run` is the one `Step` declares. Here it charges the bank, unless there is nothing to charge.
5. **A step says how it went, and nothing more.** `run` returns the step's **outcome**, of type `Outcome`, made with a method `Step` gives it: `this.succeed()` when the charge went through, or `this.fail("nothing-to-charge")`, a failure with its reason, when the price is 0 or less. **A step reports what happened, never what to do next.** The charge doesn't know which order it is charging for, what the shop does before it or after it, or what happens to an order whose charge failed.

So the charge is written, and it can't charge anything. Nothing builds it with an order's price, nothing calls `run`, and nothing does anything with its outcome. A step on its own is a business rule waiting to be used. The shop still needs a way to run it for an order.

## Outcomes

We have shown a business rule written as a step, which asks for its data and says how it went, and nothing else.

| The usual way | With Itinera |
|---|---|
| A charge function usually signals a failure by throwing an error of its own, or by returning a value of its own, so each caller learns to read it its own way. | Every step says how it went the same way: success, or a failure with a reason. |
| A function usually gets its data as parameters, from whichever caller calls it. | The step names each piece of data it needs as an input, in its own declaration, so what it reads is listed in one place. |
