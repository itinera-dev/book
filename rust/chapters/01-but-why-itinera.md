# But why Itinera?

## What you want to do

Fair question. You already know how to write a checkout, so why would you hand its flow to a framework? Let's not argue about it in the abstract. Let's look at code you have probably written yourself, and see what it costs.

You run a small online shop. When a customer checks out, the shop reserves the goods, charges the card, ships the parcel and tells the customer how it went. You want to answer simple questions about any order: what the shop tried, in what order, what it did when something failed, and who was told.

This chapter looks at a checkout written the usual way, by a competent developer, and asks it those questions. No Itinera yet: just the usual code, and four orders to try it on.

## The code

The checkout is split the way most code bases split it: a module for payments, one for shipping, one for the checkout itself, and small modules for the log and the audit trail. The bank, the warehouse, the carriers and the customer messages are faked inside the program, so it prints the same thing every time it runs.

The checkout reserves, charges and ships, then tells the customer how it went:

```example
name: ch01_checkout
region: checkout
```

Taking the payment charges the card. When the card has expired, it asks the bank for the new card and charges that one. It also records every successful charge in the audit trail:

```example
name: ch01_checkout
region: payment
```

Shipping hands the parcel to the main carrier, and to a backup carrier when the main one cannot take it:

```example
name: ch01_checkout
region: shipping
```

The program checks out four orders. Ann's goes through without trouble. Ben's card has expired. The main carrier cannot take Cat's parcel. Dan's card has expired, and the bank refuses the new one too.

```example
name: ch01_checkout
region: main
```

It prints:

```output
name: ch01_checkout
```

The usual checkout is plain Rust, without itinera. Its modules are written inline so that the whole program fits in one file; in a real project each would be a file of its own. Errors travel as `Result` values, and the `?` operator passes them up: that single character, at the end of the second `bank.charge(...)` in `take_payment`, is what sends a second refusal up through `reserve_charge_and_ship` to `checkout`. The backup carrier's error is dropped by `.ok()`, which turns the `Result` into an `Option`.

## How it works

Every piece of this code is reasonable. We would have written it this way too. Now try to answer some questions about it.

1. **In what order does a checkout do its work?** This one is easy: the function that reserves, charges and ships says it in three lines. But each of those lines hides decisions of its own, and nothing in the checkout shows them.
2. **What happens to an order when the card is declined twice?** The checkout does not say. You follow the charge into the payments module. There, a card declined because it has expired is renewed with the bank and charged once more. If the bank refuses that second charge, the refusal travels silently back up through two functions to the checkout, which logs a warning and tells the customer. A card refused for any other reason is never tried again. To learn this you read three functions in two modules, and the bank's list of reasons for refusing a charge. Dan's order shows the answer, but its log does not say that it was the new card the bank refused.
3. **What happens when the main carrier cannot take the parcel?** The shipping module tries the backup carrier. If the backup cannot take the parcel either, its error is thrown away: the log shows only the main carrier's trouble, and the checkout reports "no carrier could take it", without saying why.
4. **Is every successful charge audited?** Only the payments module tells you. The audit happens at the end of the payment function, after the renewal. Reading the checkout, you would never know that charges are audited at all.
5. **When is the customer told, and what?** At the end of the checkout, whether the order went through or not. On a failure, the customer is told only that the order could not be completed. Why it failed is in the log, where each module writes what its author thought worth writing, in its own way: the checkout logs failures, the payments module logs renewals, the shipping module logs the switch to the backup carrier.

The code mixes two kinds of decision. **Business rules** decide what the shop does: an order is charged its price less any discount, goods are reserved before they are shipped, a parcel goes to a carrier. **Flow control** decides what happens around that work: renew the card and try once more, fall back to another carrier, give up, audit, tell the customer. Logging is woven through both. Every flow decision lives inside a business function, so the only way to find it is to read that function, and the one it calls, and the one after.

That is the answer to "why Itinera?": the hardest part of understanding a system like this is knowing what is going on. Answering one question means reading every module, and a decision is easy to miss: the audit tucked into the payment function, the backup carrier's error dropped, the second refusal that travels up without a word. Changing a decision is harder still, because you first have to find every place that makes it.

The next chapters write the same shop with Itinera, where every flow decision has one place to live, and you can answer each of these questions by reading one declaration.

## Remember

1. A business rule says what the business does; a flow decision says what happens around it: try again, fall back, give up, tell someone. Tell them apart before you change either.
2. A flow decision made inside a business function is invisible to the code that calls it.
3. A log says only what its author chose to say. Answer questions about the flow from the code, not from the log.
