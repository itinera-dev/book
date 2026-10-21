# Using a step in another workflow

## What you want to do

Customers can now collect their orders at the shop's counter, paying at the counter's card terminal and taking the goods away, so nothing is shipped. You want the counter to look the order up, price it and charge it exactly as delivered orders are, without copying any of that code, and to decide for itself everything else.

## The code

The shop is the one [chapter 37](37-mistakes-caught-before-a-journey.md) lists. The counter's terminal is one more implementation of the `Payments` interface the charge already asks for, as [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md) shows it. Like the bank, it reads the year a card expires with `expiry`, a helper of the faked bank:

```example
name: ch42_using_a_step_in_another_workflow
region: terminal
```

The counter's workflow lists three of the shop's steps, `Lookup`, `Price` and `Charge`, under the same names, and provides the order book and the terminal:

```example
name: ch42_using_a_step_in_another_workflow
region: counter
```

What it names is as earlier chapters show it: the lookup, `AlreadyCompleted`, `OrderBook`, `ORDER_BOOK` and `Ledger` in [chapter 22](22-finishing-a-journey-early.md), the pricing in [chapter 20](20-an-optional-input.md), the charge in [chapter 33](33-giving-every-reporter-what-it-needs.md), the audit in [chapter 34](34-saying-which-attempt-took-the-money.md), the token `PAYMENTS` in [chapter 7](07-giving-a-step-the-tools-it-needs.md), and the log in [chapter 33](33-giving-every-reporter-what-it-needs.md), [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md) and [chapter 36](36-orders-refused-before-they-start.md). The shop's own workflow class, `Orders`, is as [chapter 39](39-deciding-where-a-decision-belongs.md) shows it.

The program prints the listing of each workflow with `entry`, as [chapter 37](37-mistakes-caught-before-a-journey.md) did:

```example
name: ch42_using_a_step_in_another_workflow
region: listings
```

Then it runs two new orders at the counter: Nia's, with a discount, and Omar's, whose card has expired. Each order's initial data has what the three steps read, and no parcel or customer, since nothing at the counter reads them; the customer's name goes to the log as global metadata, as in [chapter 33](33-giving-every-reporter-what-it-needs.md):

```example
name: ch42_using_a_step_in_another_workflow
region: run
```

`followUp` now reads the counter's output, with the methods of [chapter 25](25-reading-the-result.md), and says to hand back a card the terminal declined:

```example
name: ch42_using_a_step_in_another_workflow
region: follow_up
```

It prints:

```output
name: ch42_using_a_step_in_another_workflow
```

## How it works

1. **A step belongs to no workflow.** `Lookup`, `Price` and `Charge` say what they read and which services they ask for, by key, and how their work went. Nothing in them names `orders`, the next step or a policy, so the counter lists them as they are, and they run as they do for a delivered order: Nia's price is 18 less 3, and the charge takes 15.
2. **Each workflow provides its own services.** The charge asks for `payments`, and gets the bank in `orders` and the terminal in `counter`: the listings say so, and so does `service_provided`, Nia's event 11. The counter's receipts start with `C`, and the charge never knew which client it had.
3. **Each workflow decides its own flow.** In `orders`, a charge that fails because the card has expired is rescued, as [chapter 14](14-repairing-then-trying-once-more.md) showed. In `counter`, it has no rescue step, so the same failure, `card-expired`, gives the charge up and fails Omar's journey: the customer is standing there, and can hand over another card. `counter` has no `ship`, so none of its retry budget or policies, and no `outage` policy, since the counter's terminal has no network to lose.
4. **Each workflow decides its own policies and reporters.** `Audit` and `AlreadyCompleted` are attached in both, and `Audit` writes the same audit line. `counter` lists the `Log` reporter alone, and no workflow policy, so nobody is told by email: the log's lines start with the workflow's name, `counter`.
5. **Both workflows live in one program.** Each is declared once and checked as its class is defined, as [chapter 37](37-mistakes-caught-before-a-journey.md) showed: had `counter` listed `Charge` without providing `payments`, `@workflow` would throw a `Violations` with `service not provided` as the program loads, before any order runs.

A step can now be changed once for every workflow that lists it, which is how the shop wants its coding agents to work: they change its business rules, often in steps. The shop wants to know that such a change leaves every order's flow as a person agreed.

## Outcomes

We have shown how one step runs in two workflows, each deciding its own services, flow, policies and reporters.

| The usual way | With Itinera |
|---|---|
| A second sales channel often copies the first one's code, or adds flags to it for the cases that differ. | The second workflow lists the same steps, and declares only what differs. |
| A function reused by a second channel often has to learn which channel called it. | The charge runs unchanged in both workflows, with whichever client each provides. |
