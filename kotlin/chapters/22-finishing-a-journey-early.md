# Finishing a journey early

## What you want to do

Now and then the website sends an order a second time: Mia's order A-13, paid and shipped yesterday, arrives again today. You want the shop to see that the order is already done, and end its journey there, as succeeded, without charging Mia or booking her parcel a second time.

## The code

The shop keeps an order book, which says whether an order was already completed. It is a service, like the bank, so it is an interface of the shop's own:

```example
name: ch22_finishing_a_journey_early
region: order_book
```

A new step, the lookup, asks the order book about the order and gives back what it says, under `completed`. It decides nothing about what happens next:

```example
name: ch22_finishing_a_journey_early
region: lookup
```

A step policy, `alreadyCompleted`, reads that answer. Its `on step success` asks for `completed` from the step, with `@FromStep`, as the audit asks for the receipt in [chapter 15](15-reacting-to-what-a-step-did.md). For an order already completed, it returns `finishWorkflow()`, which `ItineraStepPolicy` gives the class beside `keepDefault()`: the lifecycle `FinishWorkflow`. Otherwise it keeps the default:

```example
name: ch22_finishing_a_journey_early
region: already_completed
```

The workflow runs the lookup first, with the policy attached to it. Its new provider, `ledger`, makes the order book, which in this program is faked, and holds the one order completed before today, Mia's:

```example
name: ch22_finishing_a_journey_early
region: workflow
```

The other steps, policies and reporters it names are as earlier chapters show them: the pricing, the charge and the audit in [chapter 20](20-an-optional-input.md), `RenewCard` in [chapter 14](14-repairing-then-trying-once-more.md), the shipping, `Outage` and `HeavyParcels` in [chapter 17](17-giving-up-a-step-early.md), `Notify` and the role `Messages` in [chapter 19](19-letting-policies-act-on-the-outside-world.md) and the desk in [chapter 11](11-naming-a-journey.md). The bank's provider is [chapter 12](12-trying-again.md)'s, the interface `Payments` it lends [chapter 14](14-repairing-then-trying-once-more.md)'s, the carriers' provider [chapter 13](13-a-carrier-for-each-attempt.md)'s, with [chapter 12](12-trying-again.md)'s `Carrier`, and the input adapter [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s.

The log is [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s, with the parts that page points to, and now also says which policy finished a journey:

```example
name: ch22_finishing_a_journey_early
region: decided
```

`NewOrder` and `Parcel` are as [chapter 21](21-deciding-where-a-steps-data-comes-from.md) shows them, and `Order`, the output, as [chapter 19](19-letting-policies-act-on-the-outside-world.md) does. `main` runs two orders, Ann's and Mia's:

```example
name: ch22_finishing_a_journey_early
region: run
```

It prints:

```output
name: ch22_finishing_a_journey_early
```

## How it works

1. **`FinishWorkflow` ends the journey at once, as succeeded.** It is the second lifecycle, after `FailWorkflow` in [chapter 17](17-giving-up-a-step-early.md). Mia's lookup succeeded and contributed `completed`, so `alreadyCompleted` finished the journey. Her price, charge and shipping never ran: her log has no attempt of theirs, and the bank and the carriers were never asked, so Mia is charged only once.
2. **The journey says who finished it.** `journey_succeeded`, a `JourneySucceeded`, carries `decidedBy`, which is one of two things, so the log's `when` on it needs no `else`: a `DecidingHook`, naming the policy whose hook returned `FinishWorkflow`, or `DecidedBy.Default`, which names none. Mia's is a `DecidingHook`, and the log prints it: `FinishWorkflow from alreadyCompleted`. Ann's journey ended by default, after its last step, so hers is `DecidedBy.Default`.
3. **Only `on step success` may finish a journey, and its type says so.** What `finishWorkflow()` returns is an `OnSuccess`, which `on step success` returns, and never an `OnFailure`, which `on step retry` and `on step abnormal termination` return. So a step that hasn't succeeded can never finish its journey.
4. **What the step contributed is kept.** The hook runs after the lookup's contributions are committed, so `completed` is in Mia's data bag, as in Ann's.
5. **The workflow's own hooks still run.** A journey finished early has succeeded, so `Notify`'s `on workflow success` tells Mia. Her data bag holds no tracking number and no gift card's code, so she reads `Order A-13 is complete.` The receipt and the tracking of yesterday's journey belong to that journey, so `main` reads none of them in this one.
6. **The step reports what it found, and the policy decides what follows.** The lookup only says whether the order was completed. Ending the journey there is a decision about the flow, so it lives in a policy. The price, the charge and the shipping could each skip an order already completed, but each of them would then need to know about the order book, and a step added later would need to know too. **A decision about the whole journey belongs in a policy, not in every step it would stop.**

The shop no longer charges an order twice. But it still keeps the money of an order it charges and never ships, such as Ned's in [chapter 17](17-giving-up-a-step-early.md).

## Outcomes

We have shown how a step policy finishes a journey early, as succeeded, once nothing is left to do.

| The usual way | With Itinera |
|---|---|
| A process that stops early often does so through a `return` deep in the code, and nothing records which check stopped it. | `on step success` returns `FinishWorkflow`, and `journey_succeeded` names the policy that finished the journey. |
| Skipping work for a request already handled often means a check at the top of each function that does the work. | One policy, attached to one lookup, ends the journey, and the steps it stops know nothing about it. |
