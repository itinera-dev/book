# Using a step in another workflow

## What you want to do

Customers can now collect their orders at the shop's counter, paying at the counter's card terminal and taking the goods away, so nothing is shipped. You want the counter to look the order up, price it and charge it exactly as delivered orders are, without copying any of that code, and to decide for itself everything else.

## The code

The shop is [chapter 37](37-mistakes-caught-before-a-journey.md)'s. The counter's terminal is one more implementation of the `Payments` trait the charge already asks for:

```example
name: ch41_using_a_step_in_another_workflow
region: terminal
asynchronous: ch41_using_a_step_in_another_workflow_async
```

The counter's workflow lists three of the shop's steps, `Lookup`, `Price` and `Charge`, under the same names, and provides the order book and the terminal. Its initial data has what those steps read, and no parcel:

```example
name: ch41_using_a_step_in_another_workflow
region: counter
asynchronous: ch41_using_a_step_in_another_workflow_async
```

`main` prints the listing of each workflow, as [chapter 37](37-mistakes-caught-before-a-journey.md) did:

```example
name: ch41_using_a_step_in_another_workflow
region: listings
asynchronous: ch41_using_a_step_in_another_workflow_async
```

Then it runs two new orders at the counter: Nia's, with a discount, and Omar's, whose card has expired:

```example
name: ch41_using_a_step_in_another_workflow
region: run
asynchronous: ch41_using_a_step_in_another_workflow_async
```

In the asynchronous form, `Payments` and the terminal take `#[async_trait]`, the counter lists `Log` under `async_reporters`, and each journey is awaited. It prints:

```output
name: ch41_using_a_step_in_another_workflow
asynchronous: ch41_using_a_step_in_another_workflow_async
```

## How it works

1. **A step belongs to no workflow.** `Lookup`, `Price` and `Charge` say what they read and which services they ask for, by key, and how their work went. Nothing in them names `orders`, the next step or a policy, so the counter lists them as they are, and they run as they do for a delivered order: Nia's price is 18 less 3, and the charge takes 15.
2. **Each workflow provides its own services.** The charge asks for `payments`, and gets the bank in `orders` and the terminal in `counter`: the listings say so, and so does `service_provided`, event 11. The counter's receipts start with `C`, and the charge never knew which client it had.
3. **Each workflow decides its own flow.** In `orders`, a charge that fails because the card has expired is rescued, as [chapter 14](14-repairing-then-trying-once-more.md) showed. In `counter`, it has no rescue step, so the same failure, `card-expired`, gives the charge up and fails Omar's journey: the customer is standing there, and can hand over another card. `counter` has no `ship`, so none of its retry budget or policies, and no `outage` policy, since the counter's terminal has no network to lose.
4. **Each workflow decides its own policies and reporters.** `Audit` and `AlreadyCompleted` are attached in both, and `Audit` writes the same audit line. `counter` lists the `Log` reporter alone, and no workflow policy, so nobody is told by email: the log's lines start with the workflow's name, `counter`.
5. **Both workflows live in one program.** Each is declared once and checked as it compiles, as [chapter 37](37-mistakes-caught-before-a-journey.md) showed: had `counter` listed `Charge` without providing `payments`, it would not compile, with the violation `service not provided`.

A step can now be changed once for every workflow that lists it, which is how the shop wants its coding agents to work: they change its business rules, often in steps. The shop wants to know that such a change leaves every order's flow as a person agreed.

## Outcomes

We have shown how one step runs in two workflows, each deciding its own services, flow, policies and reporters.

| The usual way | With Itinera |
|---|---|
| A second sales channel often copies the first one's code, or adds flags to it for the cases that differ. | The second workflow lists the same steps, and declares only what differs. |
| A function reused by a second channel often has to learn which channel called it. | The charge runs unchanged in both workflows, with whichever client each provides. |
