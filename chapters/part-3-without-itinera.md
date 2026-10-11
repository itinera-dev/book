# Without Itinera

Part 3 wrote the shop's steps: what they need, what they contribute, how they say how it went, and what happens when something goes wrong inside them. This section shows two habits of the usual code that those chapters replace, each with the Itinera version beside it.

## A function that decides what happens next

Without Itinera, the charge calls the bank, and when the bank times out, it decides on its own to try again, up to three times. Once the money is taken, it ships the order itself.

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
2. **A flow decision:** a timeout is worth trying again, three attempts in all. The limit is a constant inside the charge, invisible to anyone reading the checkout.
3. **Another flow decision:** shipping comes next. The charge knows it, and calls it.
4. **Logging:** the charge prints what it tried, in its own words.

To answer "how many times is a card charged before the shop gives up?", you read the body of the charge, and hope no other function retries around it.

## Deciding the flow with Itinera

With Itinera, a step only reports what happened. In chapter 10 the shipping meets the same situation, a carrier that is busy, and reports a failure that might work next time:

```example
name: ch10_outcomes
region: ship
```

How many times it is tried, and what comes after it, are in the workflow:

```example
name: ch10_outcomes
region: workflow
```

The step never retries itself and never calls the next step. The retry budget is read from the workflow's declaration, and every attempt shows in the events, the same way for every step.

## Data written halfway

Without Itinera, an order is often one record that each function fills in as the checkout goes. Shipping books the carrier and records the tracking number, then learns the carrier cannot deliver.

```example
name: part3_half_written
region: record
```

The checkout charges Bo's order, then tries to ship it, and prints what the record says afterwards:

```example
name: part3_half_written
region: run
```

It prints:

```output
name: part3_half_written
```

The record has a tracking number for a parcel that never left. Whatever reads it next, such as an email telling Bo the parcel is on its way, believes it. Keeping the record true means remembering, on every path that fails, to undo what was written before the failure.

## Keeping the data true with Itinera

With Itinera, the shipping of chapter 9 does the same thing in the same order: it contributes the tracking number, then fails.

```example
name: ch09_contributions
region: ship
```

Because the attempt failed, its contribution is never committed. Bo's journey ends with the amount charged and the receipt, which the charge's successful attempt committed, and no tracking number:

```output
name: ch09_contributions
```

## What changed

1. **Flow decisions left the business code.** Whether to try again, how many times, and what runs next, are declared in the workflow. Steps report what happened, and nothing more.
2. **One place to read the retries.** Every retry budget is in the workflow's declaration, and every attempt is in the events.
3. **The data stays true by construction.** A step's contributions reach the data bag only when its attempt succeeds, so no failure path has to undo anything, and nothing half written reaches a later step or the result.
4. **Each step can be tested alone.** The charge no longer needs a fake carrier to be tested, since it never calls the shipping.
