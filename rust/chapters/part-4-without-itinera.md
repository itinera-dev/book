# Without Itinera

## The shop, written the usual way

This part gave the shop its flow: shipping tried twice, a backup carrier, an expired card repaired by the charge's rescue step, digital orders that aren't shipped, and a payment network that may be down. Here is the same shop written the way such code is usually written in Rust, by a careful developer, without Itinera. Each piece of flow control sits where the code first needed it.

The usual code is plain Rust, with no dependency. One thread runs every order, so its breaker and warehouse keep their state in `Cell`s, and its retries are written the way Rust code usually writes them: a `loop`, a helper that takes a closure, and a builder-style policy object.

The program takes four orders. Ann's card has expired, and her bank has a new one. Ben's order is digital. The main carrier is down all day, and after two failures in a row a circuit breaker stops calling it. The warehouse is busy the first time it is asked, and the bank times out on the first charge of the day.

```output
name: part4_usual_checkout
```

The output is honest about what the code logs, and silent about the rest: the bank's timeout on Ann's order was retried, and nothing says so.

## Retry strategies that multiply

The usual code retries three calls in three different ways. Reserving the stock is retried in a `loop`, inside the checkout:

```example
name: part4_usual_checkout
region: checkout
```

The charge goes through `retrying`, a helper that wraps any closure in retries, which is how a retry decorator is usually written in Rust:

```example
name: part4_usual_checkout
region: decorator
```

Shipping goes through a retry policy object, of the kind a retry crate offers:

```example
name: part4_usual_checkout
region: library
```

Each is reasonable on its own. Together, they mix a business rule (reserve, charge, ship) with a retry strategy, and each strategy has its own limit, its own idea of which errors are worth retrying, and its own logging, or none. To answer "how many times is an order charged before we give up?", you read the charge, then `retrying`, then `Trouble::is_transient`. Nothing outside the code can tell you which retries happened.

With Itinera, every retry budget is in the workflow's declaration, on the line of the step it belongs to:

```example
name: ch13_trying_again
region: workflow
```

Which failures are retried follows from the step's outcome: a retriable failure, from `Self::fail_retriably`, is retried while the budget lasts, and so is an abnormal termination when the step's line says `abnormal_termination_retriable`; anything else is given up. A step with a rescue step is never retried: its failure goes to the rescue step instead. Any earlier stop is a policy's `FailWorkflow`. Every attempt, retry and decision to give up is an event, the same for every step.

## Flow decisions in business code

The usual charge knows what comes after it. It charges, then calls `self.ship(order)` itself, except for digital orders:

```example
name: part4_usual_checkout
region: charge
```

The rule "digital orders aren't shipped" lives inside charging, where nobody looking at shipping will find it. The charge can't be tested without shipping, or reused in a shop that ships differently.

With Itinera, the charge only charges, and contributes what it learned: for a digital order, a download.

```example
name: ch16_ending_early
region: charge
```

What follows is in the workflow, and whether a digital order ships is decided where it belongs. Either the ship step skips itself, as shipping's own rule, or a policy finishes the journey from what the charge contributed:

```example
name: ch16_ending_early
region: downloads
```

## A circuit breaker

The usual breaker wraps the call to the carrier in a closure, and decides inside `call` whether the call happens at all:

```example
name: part4_usual_checkout
region: breaker
```

The shipping code uses it, matches on what it refuses, and falls back to the backup carrier:

```example
name: part4_usual_checkout
region: ship
```

Shipping now knows about two carriers, a breaker, a retry policy and a fallback. The breaker's state is hidden inside it: when the fourth order goes to the backup carrier, only one log line hints at why.

With Itinera, the ship step only ships, through the carrier it is lent, without knowing which one it is:

```example
name: part4_circuit_breaker
region: step
```

A step policy sees how every attempt to ship went, and records it, with the attempt's number, through a role the workflow provides, `CircuitBreaker`. The breaker's state belongs to the workflow, which keeps it across journeys:

```example
name: part4_circuit_breaker
region: policy
```

The carrier's provider, part of the workflow too, reads that state before it hands out a carrier, and gives the backup one while the breaker is open. The role counts only the attempts that went to the main carrier, which it tells apart by the same rule as the provider, `main_carrier_for`. The workflow keeps the breaker's state in an `Arc` field, shared by every instance the shop creates. A workflow is `Send` and `Sync`, so the state is held in atomics rather than `Cell`s:

```example
name: part4_circuit_breaker
region: workflow
```

The program ships three orders with the main carrier down, and a small reporter prints every failed attempt to ship. The first two orders each fail once with the main carrier and ship with the backup one; those two failures in a row open the breaker, so the third order goes to the backup carrier on its first attempt, and nothing fails:

```output
name: part4_circuit_breaker
```

Every attempt the breaker hears about is in the events, with the hook that told it. The breaker itself is the workflow's own code, written once, next to the carriers it chooses between.

## Another carrier on the next attempt

The usual fallback is in the shipping code above: try the main carrier, match on the failure, call the backup one. Every new carrier, or every change in the order they are tried, is a change to shipping.

With Itinera, the ship step only ships, through the carrier service it is given:

```example
name: ch13_backup_carrier
region: step
```

The provider makes a fresh carrier for every attempt, and picks it from the attempt number:

```example
name: ch13_backup_carrier
region: workflow
```

It could instead read a choice that the `on step retry` hook of a step policy contributed, as data from the workflow. That's how [chapter 13](13-trying-again.md) does it while the step reads its carrier's name from the data bag:

```example
name: ch13_trying_again
region: policy
```

## Repair code inside the business rule

Look again at the usual charge above. It matches on `Trouble::ExpiredCard`, asks the bank for the new card and charges again, all in one function, between the retry helper and the digital rule. The repair is invisible from outside: Ann's order printed one line about it, because the developer chose to log it.

With Itinera, the charge stays a charge: an expired card is a failure with a reason.

```example
name: ch14_rescue
region: charge
```

The repair is a rescue step of its own, which reads why the charge failed and contributes the new card:

```example
name: ch14_rescue
region: rescue
```

The workflow attaches it to the charge, and the events say when a rescue happened and how it went:

```example
name: ch14_rescue
region: workflow
```

## What changed

| Decision | Without Itinera | With Itinera |
|---|---|---|
| How many times to try | a `loop`, a helper and a retry object, each with its own limit | the retry budget of each step, in the workflow |
| Which failures are retried | each strategy's own test of the error | the same rule for every step: a retriable failure, or an abnormal termination the step's line lets be retried; a step with a rescue step is rescued instead |
| Whether a digital order ships | an early return inside the charge | a policy finishing the journey, or the ship step skipping |
| When to stop calling a carrier | a breaker wrapped around the call | a policy recording failures through a role, and the provider choosing the carrier |
| Which carrier to use next | a fallback inside the shipping code | the carrier's provider, part of the workflow |
| How to repair an expired card | a `match` arm inside the charge | a rescue step, with its own events |

What you gain is the same each time:

1. **One place to read the flow.** The workflow's declaration lists the steps, their budgets, their rescue steps and their policies. Shipping and charging hold only their business rules.
2. **Business code that can be tested alone.** The charge can be tested without the bank's retries, shipping without two carriers and a breaker, and the repair without a charge.
3. **Observation that is the same everywhere.** Every attempt, retry, rescue and early end is an event, emitted by the executor, whoever wrote the step.
