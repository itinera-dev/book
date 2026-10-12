# Without Itinera

Part 1 has already set the two versions of the shop against each other: the checkout of [chapter 1](01-but-why-itinera.md), written the usual way, and the workflow of [chapter 3](03-read-the-workflow-know-what-happens.md). This section puts them side by side.

## The usual checkout

The checkout of [chapter 1](01-but-why-itinera.md) is spread over Rust modules: payments, shipping, the checkout itself, the log and the audit trail. It is written honestly, the way a competent developer writes it, and it works. Its top level reserves, charges and ships, and tells the customer how it went:

```example
name: ch01_checkout
region: checkout
```

What it mixes, in almost every function:

1. **Business rules:** the amount is the price less any discount; the goods are reserved, the card charged and the parcel handed to a carrier.
2. **Flow decisions:** reserve, then charge, then ship; renew an expired card and charge once more; fall back to the backup carrier; give up on a refused card; tell the customer at the end.
3. **Ways of recovering:** two of them, written differently in two modules: the card renewal is a `match` arm in the payments module, and the carrier fallback is an `Err` arm in the shipping module that calls the backup carrier.
4. **Auditing and logging:** an audit line at the end of the payment function, and log lines wherever each author thought one useful.

## The same shop with Itinera

Here is the workflow of [chapter 3](03-read-the-workflow-know-what-happens.md) again.

```example
name: ch03_shop
region: workflow
```

## Side by side

| What the shop decides | In the usual checkout | With Itinera |
|---|---|---|
| Reserve, then charge, then ship | the checkout's function that reserves, charges and ships | the order of the step lines in the workflow |
| Renew an expired card and charge once more | a `match` arm inside the payment function | the rescue step `RenewCard`, declared on `charge` |
| Give up when the card is refused twice | an `Err` passed up by `?` through two functions | the rule for rescued steps: a second failure fails the journey |
| Try the backup carrier | the `Err` arm of the shipping function, which drops the backup's error with `.ok()` | the retry budget of `ship` and the `carrier` provider |
| Audit every successful charge | a call at the end of the payment function | the `audit` policy, attached to `charge` |
| Tell the customer how it went | both branches at the end of the checkout | the `notify` policy, attached to the workflow, through the `Messages` role |
| Record what happened | log lines in each module, each in its own style | events, the same for every journey, sent to the workflow's reporters |

## What you gain

1. **One place to read the flow.** The declaration tells you the order of the steps, how many times each may be tried, which step is rescued, which policies are attached, and, by what it leaves out, which defaults apply. What remains in the steps is their own business, such as which failures the rescue step can repair. In the usual checkout, each answer meant following calls through several modules.
2. **Business code you can test alone.** The charge step takes a card, a price and a discount, and reports what happened. Testing it needs only a fake payment client, a small struct implementing the `Payments` trait: no carrier, no customer messages and no audit trail, and it cannot accidentally ship anything.
3. **Observation that is the same everywhere.** The usual checkout drops the backup carrier's error and never says that it was the new card the bank refused. With Itinera, the outcome of every attempt of every step is an event, with the reason of a failure, whoever wrote the step.
4. **Decisions you can change in one place.** Giving shipping a third try is one change to its `#[with_step(...)]` line, changing the backup carrier is one change to the `carrier` provider, and the steps do not change at all.
