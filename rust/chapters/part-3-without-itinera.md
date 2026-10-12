# Without Itinera

Part 3 wrote the shop's steps: what they need, what they contribute, how they say how it went, and what happens when something goes wrong inside them. This section shows two habits of the usual Rust code that those chapters replace, each with the Itinera version beside it.

## A function that decides what happens next

Without Itinera, the charge calls the bank, and when the bank times out, it decides on its own to try again, up to three attempts in all, in a `loop` around a `match` on the bank's `Result`. Once the money is taken, it ships the order itself.

```example
name: part3_retry_in_business_code
region: charge
```

The bank times out the first time it is called. The shop runs two orders, the second with nothing to charge:

```output
name: part3_retry_in_business_code
```

The function is careful and correct, and it mixes four things:

1. **A business rule:** charge the price.
2. **A flow decision:** a timeout is worth trying again, three attempts in all. The limit is the constant `CHARGE_ATTEMPTS`, next to the charge, invisible to anyone reading the checkout.
3. **Another flow decision:** shipping comes next. The charge knows it, and calls `ship` itself.
4. **Logging:** the charge prints what it tried with `println!`, in its own words.

To answer "how many times is a card charged before the shop gives up?", you read the body of the charge, and hope no other function retries around it.

## Deciding the flow with Itinera

With Itinera, a step only reports what happened. In [chapter 10](10-saying-how-it-went.md) the shipping meets a failure that might work next time, a parcel too heavy for its carrier, and returns `Self::fail_retriably("parcel-refused")`:

```example
name: ch10_outcomes
region: ship
```

The step never retries itself and never calls the next step. What comes after it is the order of the workflow's step lines, and how many times it is tried is the step's retry budget in the workflow's declaration, which [chapter 13](13-trying-again.md) teaches. Every attempt shows in the events, the same way for every step.

## Data written halfway

Without Itinera, an order is often one struct that each function fills in as the checkout goes, through a `&mut Order`. Shipping books the carrier and records the tracking number, then learns the carrier can't deliver.

```example
name: part3_half_written
region: record
```

The checkout charges Ben's order, then tries to ship it, and prints what the record says afterwards:

```example
name: part3_half_written
region: run
```

It prints:

```output
name: part3_half_written
```

The record has a tracking number for a parcel that never left. Whatever reads it next, such as an email telling Ben the parcel is on its way, believes it. The borrow checker can't help here: `ship` holds its `&mut Order` quite legitimately, and returning `Err` doesn't undo what it already wrote. Keeping the record true means remembering, on every path that fails, to undo what was written before the failure.

## Keeping the data true with Itinera

With Itinera, the shipping of [chapter 9](09-passing-data-on.md) does the same thing in the same order: it contributes the tracking number, then fails.

```example
name: ch09_contributions
region: ship
```

Because the attempt failed, its contribution is never committed. Ben's journey ends with the amount charged and the receipt, which the charge's successful attempt committed, and no tracking number:

```output
name: ch09_contributions
```

## What changed

1. **Flow decisions left the business code.** Whether to try again, how many times, and what runs next, are declared in the workflow. Steps report what happened, and nothing more.
2. **One place to read the retries.** Every retry budget is in the workflow's declaration, and every attempt is in the events.
3. **The data stays true by construction.** A step's contributions reach the data bag only when its attempt succeeds, so no failure path has to undo anything, and nothing half written reaches a later step or the result.
4. **Each step can be tested alone.** The charge no longer needs a fake carrier to be tested, since it never calls the shipping.
