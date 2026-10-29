# A first step

## What you want to do

The shop charges an order its price. You want that business rule written with Itinera: the charge, and nothing else.

## The code

In Itinera, a business rule is a **step**. In Kotlin, a step is a class implementing `ItineraStep`, whose constructor takes the data the step needs, and whose `run` method does the work. The shop's bank is faked, so that it would print what it charges, once something runs the step.

```example
name: ch04_first_step
region: step
```

That is the whole step. The program declares it and does nothing else: its `main` is empty, nothing builds a `Charge`, and nothing calls its `run`, so it prints nothing.

## How it works

1. **A step is a class marked `@Step`, implementing `ItineraStep`.** Both come from `org.itineradev.itinera.step`. The annotation tells Itinera that the class is a step, and the interface gives it what a step needs to say how it went. Neither builds a `Charge` or runs anything.
2. **A step asks for the data it needs.** Each parameter of the constructor marked `@Input` is an **input**, named after its parameter: `price: Long` asks for a whole number under the name `price`. Since the parameter is a `private val`, it is also a property of the class, and `run` reads it as `price`. The step says what it needs, and nothing about where it comes from.
3. **A step does its work in `run`.** It is an ordinary method of the class, with no `override`, since `ItineraStep` doesn't declare it: each step's `run` asks for what that step needs, and Itinera reads what it asks for from the class. Here it charges the bank, unless there is nothing to charge.
4. **A step says how it went, and nothing more.** `run` returns the step's **outcome**, of type `Outcome`, made with a method `ItineraStep` gives the class: `succeed()` when the charge went through, or `fail("nothing-to-charge")`, a failure with its reason, when the price is 0 or less. A failure is something the step returns, never an exception it throws. **A step reports what happened, never what to do next.** The charge doesn't know which order it is charging for, what the shop does before it or after it, or what happens to an order whose charge failed.

So the charge is written, and it can't charge anything. Nothing builds it with an order's price, nothing calls `run`, and nothing does anything with its outcome. A step on its own is a business rule waiting to be used. The shop still needs a way to run it for an order.

## Outcomes

We have shown a business rule written as a step, which asks for its data and says how it went, and nothing else.

| The usual way | With Itinera |
|---|---|
| A charge function usually signals a failure by throwing an exception of its own, or by returning a type of its own, so each caller learns to read it its own way. | Every step says how it went the same way: success, or a failure with a reason. |
| A function usually gets its data as parameters, from whichever caller calls it. | The step names each piece of data it needs as an input, in its own declaration, so what it reads is listed in one place. |
