# Without Itinera

Part 2 charged an order and shipped it, as two steps of one workflow. This section shows the same shop written the usual way, in plain Rust, and then the Itinera version beside it.

## The usual code

Without Itinera, a checkout function calls the charge and then the shipping, one after the other, and passes a failed charge up with `?`. It is short, and there is nothing wrong with it as code.

```example
name: part2_calls_in_a_row
region: calls
```

The caller runs it for two orders, the second with nothing to charge.

```example
name: part2_calls_in_a_row
region: run
```

It prints:

```output
name: part2_calls_in_a_row
```

## What the checkout has to know

The checkout is two lines, but each line carries a decision that belongs to neither function:

1. **The order of the calls.** Shipping comes after charging because the checkout says so. Nothing outside the checkout's body shows it.
2. **What to do when the charge fails.** The `?` after `charge(price)` returns early on an error, and that is a flow decision: "do not ship an order that was not paid". It sits in the middle of the business code, as one character that is easy to miss.
3. **How each function is called, and how it fails.** The checkout knows what each function takes, what it returns and the type of its error, `ChargeError`, and it shapes its own result, `Shipped`, and its own error from them. Every change to one function reaches the checkout and every caller of the checkout: a new kind of failure is a new variant of the error they all see, and a `ship` that could fail would change the checkout's error type.

None of these is a mistake. The first two are decisions about the flow, and the third ties the checkout, and everything that calls it, to the signatures and errors of both functions, all in code whose job is the business.

## The same shop with Itinera

With Itinera, the order of the steps is the workflow's list, and nothing else:

```example
name: ch07_two_steps
region: workflow
```

The shipping step knows nothing of the charge. It asks for the order's ID and reports what happened:

```example
name: ch07_two_steps
region: step
```

## What changed

1. **The flow has one place.** The order of the steps, and the rule that a failed step stops the journey, are read from the workflow's declaration, not from a function body.
2. **Each step stands alone.** The charge and the shipping never call each other, never know each other's data or failures, and can be tested on their own.
3. **The caller knows one thing.** It runs a journey and reads the result: `result.status` says whether the journey succeeded, failed or was aborted, and `result.output()` reads the data it ended with, when it has any. It never needs to know how many steps there are, or how each one fails, and its types don't change when a step gains a new kind of failure.
4. **What happened is told the same way for every journey.** The executor reports every step's attempt and outcome as events, whoever wrote the step, as [chapter 23](23-seeing-what-happened.md) shows.
5. **Adding a step changes the list, not the callers.** Reserving the stock before charging, as the shop will later do, is one more `#[with_step(...)]` line in the workflow.
