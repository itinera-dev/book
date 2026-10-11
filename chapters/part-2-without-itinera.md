# Without Itinera

Part 2 charged an order and shipped it, as two steps of one workflow. This section shows the same shop written the usual way, and then the Itinera version beside it.

## The usual code

Without Itinera, a checkout function calls the charge and then the shipping, one after the other. It is short, and there is nothing wrong with it as code.

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
2. **What to do when the charge fails.** The early return on an error is a flow decision: "do not ship an order that was not paid". It sits in the middle of the business code, often as one short line that is easy to miss.
3. **How each function is called, and how it fails.** The checkout knows what each function takes, what it returns and the type of its error, and it shapes its own result and its own error from them. Every change to one function, such as a new kind of failure, reaches the checkout and every caller of the checkout.
4. **What happened.** Each function prints what it chose to print. To know what happened to an order, you read both functions and hope each one said enough.

None of these is a mistake. The first three are decisions about the flow and the last is logging, all made inside code whose job is the business.

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
3. **The caller knows one thing.** It runs a journey and reads the result: succeeded, failed or aborted, and the data the journey ended with when it has any. It never needs to know how many steps there are, or how each one fails.
4. **What happened is told the same way for every journey.** The executor reports every step's attempt and outcome as events, whoever wrote the step, as chapter 23 shows.
5. **Adding a step changes the list, not the callers.** Reserving the stock before charging, as the shop will later do, is one more line in the workflow.
