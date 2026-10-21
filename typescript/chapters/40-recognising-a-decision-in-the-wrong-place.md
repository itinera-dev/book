# Recognising a decision in the wrong place

## What you want to do

A rule put in the wrong place often still runs, and the shop's orders all go through. You want to recognise one when you read a step or a hook, before it hides a decision the shop can no longer see.

## The code

Here is a counter-example. This charge renews an expired card itself, tells the customer, and charges the new card:

```example
name: ch40_counter_example
region: charge
```

It asks for [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s `Payments` and `Cards`, with their tokens `PAYMENTS`, from [chapter 7](07-giving-a-step-the-tools-it-needs.md), and `CARDS`.

Since the charge renews the card itself, the workflow gives it no rescue step, and the card registry's provider serves the charge itself rather than a rescue step:

```example
name: ch40_counter_example
region: workflow
```

What it names is as earlier chapters show it: the lookup, `AlreadyCompleted`, `OrderBook`, `ORDER_BOOK` and `Ledger` in [chapter 22](22-finishing-a-journey-early.md), the pricing in [chapter 20](20-an-optional-input.md), `Audit` and `Outage` in [chapter 34](34-saying-which-attempt-took-the-money.md), the shipping, `Notify`, `FailedShipping`, `Carrier`, `Messages`, `Refunds` and `network` in [chapter 26](26-running-async-journeys-while-others-wait.md), `cardRegistry` in [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md), `Parcel` and `PARCEL` in [chapter 21](21-deciding-where-a-steps-data-comes-from.md), the log in [chapter 33](33-giving-every-reporter-what-it-needs.md), [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md) and [chapter 36](36-orders-refused-before-they-start.md), and the desk in [chapter 11](11-naming-a-journey.md). The program runs Jo's order, whose card has expired, with `followUp` as [chapter 34](34-saying-which-attempt-took-the-money.md) shows it, and prints:

```output
name: ch40_counter_example
```

## How it works

What the counter-example shows:

1. **It works, and its journey records none of it.** Jo's order is charged and shipped. But the log says the charge succeeded on its first attempt, events 11 and 15, and that the card was overwritten, event 17. Nothing says the card had expired, that the registry renewed it, or that Jo was told: the email came from inside the step, as a `console.log` the journey never sees.
2. **It holds three decisions where one belongs.** Charging is the step's. Renewing the card is a repair, which the rescue step made visible as `step_rescuing` and `step_rescued` in [chapter 14](14-repairing-then-trying-once-more.md). Telling the customer is a reaction, which belongs to a policy, through a role, as [chapter 19](19-letting-policies-act-on-the-outside-world.md) showed. **A step does one thing of the business, and says how it went.**
3. **Back in their places, the journey records them.** With the renewal in the rescue step `charge_rescue`, as [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md) gave it its own card registry, the charge fails with the reason `card-expired` when the card has expired, the one failure the rescue step knows how to repair, and the journey records the repair as `step_rescuing` and `step_rescued`. With the message in a policy acting through a role, as `notify` does in [chapter 19](19-letting-policies-act-on-the-outside-world.md), the journey records the hook it ran as `hook_called`.
4. **The places it doesn't belong are recognisable.** A step that tries its work again in a loop, calls a second service when the first fails, or tells anyone anything has taken a decision that isn't its own. A hook that works out a price, or judges whether a card is good enough, has taken one of the business's. **When a step or a hook takes a decision the journey doesn't record, such as a repair, a retry or a message, that decision is out of its place.**

Now you can tell a decision in its place from one out of it. But the shop's first checkout, [chapter 1](01-but-why-itinera.md)'s, still runs beside the workflow, and moving a system like it is rarely done in one go.

## Outcomes

We have shown how to recognise a decision in the wrong place, and where to move it.

| The usual way | With Itinera |
|---|---|
| A rule put in the wrong place often goes unnoticed, since the code still works. | A rule in the wrong place is spotted from the journey, which records none of what it decided. |
