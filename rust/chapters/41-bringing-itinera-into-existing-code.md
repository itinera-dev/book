# Bringing Itinera into existing code

## What you want to do

The shop's first checkout, [chapter 1](01-but-why-itinera.md)'s, still takes some of its orders, and it works. You want to bring it into Itinera without stopping it or rewriting it: one piece at a time, keeping the bank's client, the audit trail and the log it already has, with the checkout behaving as before after each move.

## The code

The program is [chapter 1](01-but-why-itinera.md)'s checkout. Its payment, which charged the card, renewed an expired one and audited the charge inside one function, is the piece moved first, into a workflow of its own, `payment`. The bank's client stays as it was: the steps ask for a trait, and the client implements it by calling its own methods:

```example
name: ch41_bringing_itinera_into_existing_code
region: payments
asynchronous: ch41_bringing_itinera_into_existing_code_async
```

The charge becomes a step, and the renewal its rescue step. The charge no longer renews anything: it says why it failed.

```example
name: ch41_bringing_itinera_into_existing_code
region: steps
asynchronous: ch41_bringing_itinera_into_existing_code_async
```

The audit becomes a policy, which writes to the audit trail with the function the checkout already used:

```example
name: ch41_bringing_itinera_into_existing_code
region: audit
asynchronous: ch41_bringing_itinera_into_existing_code_async
```

The log line the payment wrote becomes a reporter, which writes with the shop's existing `log::info`:

```example
name: ch41_bringing_itinera_into_existing_code
region: log
asynchronous: ch41_bringing_itinera_into_existing_code_async
```

The workflow has one step, and its provider makes the bank's client as the checkout did:

```example
name: ch41_bringing_itinera_into_existing_code
region: workflow
asynchronous: ch41_bringing_itinera_into_existing_code_async
```

The rest of the checkout is chapter 1's, with three changes: `Shop` no longer holds the bank; `CheckoutError` gains a variant for a payment that did not run, which `not_run` makes from any error; and the line that took the payment now calls `pay`:

```example
name: ch41_bringing_itinera_into_existing_code
region: checkout
asynchronous: ch41_bringing_itinera_into_existing_code_async
```

`pay` runs the workflow, and turns how the journey ended back into the checkout's own errors:

```example
name: ch41_bringing_itinera_into_existing_code
region: pay
asynchronous: ch41_bringing_itinera_into_existing_code_async
```

The asynchronous form starts from a checkout that was already asynchronous, its bank's client, `checkout` and `main` awaiting as they go. `Payments` and the client's implementation of it take `#[async_trait]`, the implementation awaiting the client, the steps' `run` and the audit's hook are `async fn`, and `pay` awaits the journey where the checkout awaited the payment. `main` is chapter 1's, with the same four orders, and both forms print:

```output
name: ch41_bringing_itinera_into_existing_code
asynchronous: ch41_bringing_itinera_into_existing_code_async
```

## How it works

1. **The rest of the program doesn't know.** `checkout` still returns the same result and the same errors, the shipping and the messages are untouched, and each order ends as it did in chapter 1. `pay` is the only seam: it creates an instance, runs its journey and reads how it ended.
2. **Existing code is kept, behind the workflow's parts.** The bank's client is a service through the trait it implements, the audit trail is called from a policy, and the log from a reporter. What moves into the workflow is the decisions: when to renew a card, and when to audit. The code that talks to the outside world stays where it was.
3. **The log now says what happened.** Chapter 1's payment logged that it was renewing a card, before it tried. Now the log follows the journey's events: Ben's charge failed with `card-expired`, the rescue step repaired it, and the charge succeeded. Dan's was repaired too, then failed again, `declined`, which chapter 1's log never said.
4. **One piece at a time.** The shipping, with its backup carrier, and the messages to the customer are still chapter 1's code, running beside the workflow. Each can move next: the shipping as a step with a retry budget and a provider for each attempt, the messages as a policy acting through a role. Move one piece, run the program, and compare what it prints with what it printed before.
5. **Where Itinera fits, and where it doesn't.** It fits a piece of business with decisions around it: whether to try again, how to repair, what to record and whom to tell. A single call with nothing to decide around it, such as reserving the goods here, gains nothing from a workflow of its own. It stays a call, or becomes a step of a workflow that has decisions to make.

The checkout's payment is now a workflow, and the shop has one way to charge a card. It also wants a second way to sell: customers collecting their orders at the shop's counter, paying at the counter's terminal, with nothing to ship.

## Outcomes

We have shown how to bring Itinera into code that already works, one piece at a time, keeping the code that talks to the outside world.

| The usual way | With Itinera |
|---|---|
| Untangling a function often means rewriting it, and its callers, in one go. | One piece becomes a workflow behind a function the callers already call. |
| A client, a log or an audit trail written for the old code is often rewritten for the new. | Each stays, called from a service's implementation, a policy or a reporter. |
