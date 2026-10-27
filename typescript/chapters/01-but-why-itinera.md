# But why Itinera?

## What you want to do

Fair question. You already know how to write a checkout, so why would you hand its flow to a framework? Let's not argue about it in the abstract. Let's look at code you have probably written yourself, and see what it costs.

You run a small online shop. When a customer checks out, the shop reserves the goods, charges the card, ships the parcel and tells the customer how it went. You want to answer simple questions about any order: what the shop tried, in what order, what it did when something failed, and who was told.

This chapter looks at a checkout written the usual way, in plain TypeScript, by a competent developer, and asks it those questions. No Itinera yet: just the usual code, and four orders to try it on.

## The code

The checkout is split the way most code bases split it: a module for payments, one for shipping, one for the checkout itself, and small ones for the log and the audit trail. Here they are sections of one file, marked by comments, so that the whole program fits on the page; in a real project each would be a file of its own. The bank, the warehouse, the carriers and the customer messages are faked inside the program, so it prints the same thing every time it runs. Errors travel the way they usually do in TypeScript: thrown, and caught by whichever function up the call stack has a `try` around the call.

The checkout reserves, charges and ships, then tells the customer how it went:

```example
name: ch01_checkout
region: checkout
```

Taking the payment charges the card. When the bank refuses it because the card has expired, it asks the bank for the new card and charges that one. It also records every successful charge in the audit trail:

```example
name: ch01_checkout
region: payment
```

Shipping hands the parcel to the main carrier, and to a backup carrier when the main one cannot take it. It returns the tracking number, or `undefined` when neither carrier takes the parcel:

```example
name: ch01_checkout
region: shipping
```

The program sets up the fakes and checks out four orders. Each card ends with the year it expires, and the bank, set up for the year 2026 with a limit of 1,000, refuses a card that expired before this year, and any charge above its limit. The main carrier, Rapid, takes parcels up to 20 kg; the backup carrier, Steady, up to 50 kg. So the orders' own data decides what happens to them: Ann's goes through without trouble. Ben's card expired in 2024, and the bank's new card for him expires in 2029. Cat's parcel weighs 30 kg, too heavy for the main carrier. Dan's card has expired too, and his order of 1,200 is above the bank's limit, so the bank refuses the new card as well.

```example
name: ch01_checkout
region: main
```

It prints:

```output
name: ch01_checkout
```

## How it works

Every piece of this code is reasonable. We would have written it this way too. Now try to answer some questions about it.

1. **In what order does a checkout do its work?** This one is easy: `reserveChargeAndShip` says it in a few lines. But each of those lines hides decisions of its own, and nothing in the checkout shows them.
2. **What happens to an order when the card is declined twice?** The checkout does not say. You follow the charge into the payments module. There, the `catch` in `takePayment` looks at why the bank refused: a card declined because it has expired is renewed with the bank and charged once more. If the bank refuses that second charge, nothing in `takePayment` catches it. The error thrown by the second `bank.charge(...)` leaves `takePayment`, passes through `reserveChargeAndShip`, which never mentions it, and lands in the `catch` of `checkout`, which logs a warning, tells the customer and throws it again for whoever called. A card refused for any other reason is never tried again: the `throw error` at the top of the `catch` sends the refusal on at once. To learn this you read three functions in two modules, and the bank's `Declined` error, with its reasons for refusing a charge, and not one line between the second charge and the checkout says that its refusal may pass through: `takePayment` and `reserveChargeAndShip` both say only that they return a `string`, and nothing in their types says what they may throw. Dan's order shows the answer, but its log does not say that it was the new card the bank refused.
3. **What happens when the main carrier cannot take the parcel?** The shipping module tries the backup carrier. If the backup cannot take the parcel either, its error is thrown away: the inner `catch` has no name for the error and returns `undefined`, and the reason goes with it. The log shows only the main carrier's trouble, and the checkout reports "no carrier could take it", without saying why.
4. **Is every successful charge audited?** Only the payments module tells you. The audit happens at the end of the payment function, after the renewal. Reading the checkout, you would never know that charges are audited at all.
5. **When is the customer told, and what?** At the end of the checkout, whether the order went through or not. On a failure, the customer is told only that the order could not be completed. Why it failed is in the log, where each module writes what its author thought worth writing, in its own way: the checkout logs failures, the payments module logs renewals, the shipping module logs the switch to the backup carrier.

The code mixes two kinds of decision. **Business rules** decide what the shop does: an order is charged its price less any discount, goods are reserved before they are shipped, a parcel goes to a carrier. **Flow control** decides what happens around that work: renew the card and try once more, fall back to another carrier, give up, audit, tell the customer. Logging is woven through both. **Every flow decision lives inside a business function, so the only way to find it is to read that function**, and the one it calls, and the one after.

That is the answer to "why Itinera?": the hardest part of understanding a system like this is knowing what is going on. Answering one question means reading every module, and a decision is easy to miss: the audit tucked into the payment function, the backup carrier's error dropped, the second refusal that travels up without a word. Changing a decision is harder still, because you first have to find every place that makes it.

The shop is small, and you can still read all of it. A system that has grown for years, across teams and repositories, has the same problem with no one left who has read everything, and agents changing it faster than anyone can follow. The Itinera site tells that larger story on [Why Itinera](https://itinera-dev.org/why/).

The next chapter writes the same checkout with Itinera, where every flow decision has one place to live, and answers each of these questions by reading one declaration.

## Outcomes

We have shown how flow decisions hide inside business code, so that answering a simple question about one order means reading every module it passes through.

| The usual way | With Itinera |
|---|---|
| Each function usually decides, inside its own code, what happens when something fails, so the flow is spread across every module it calls. | The flow is declared in one workflow, and [chapter 2](02-the-big-picture.md) answers every question of this chapter from it. |
| Each module often logs what its author thought worth logging, in its own way, so the log rarely tells the whole story. | One reporter attached to the workflow writes the log of every journey, in one format. |
