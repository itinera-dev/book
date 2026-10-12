# But why Itinera?

## What you want to do

Fair question. You already know how to write a checkout, so why would you hand its flow to a framework? Let's not argue about it in the abstract. Let's look at code you have probably written yourself, and see what it costs.

You run a small online shop. When a customer checks out, the shop reserves the goods, charges the card, ships the parcel and tells the customer how it went. You want to answer simple questions about any order: what the shop tried, in what order, what it did when something failed, and who was told.

This chapter looks at a checkout written the usual way, in plain Rust, by a competent developer, and asks it those questions. No Itinera yet: just the usual code, and four orders to try it on.

## The code

The checkout is split the way most code bases split it: a module for payments, one for shipping, one for the checkout itself, and small modules for the log and the audit trail. The modules are written inline, as `mod payments { ... }`, so that the whole program fits in one file; in a real project each would be a file of its own. The bank, the warehouse, the carriers and the customer messages are faked inside the program, so it prints the same thing every time it runs. Errors travel the way they usually do in Rust: as `Result` values, passed up with the `?` operator.

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

Shipping hands the parcel to the main carrier, and to a backup carrier when the main one cannot take it. It returns an `Option`: the tracking number, or nothing:

```example
name: ch01_checkout
region: shipping
```

The program sets up the fakes and checks out four orders. Each card ends with the year it expires, and the bank refuses a card that expired before this year, and any charge above 1,000. The main carrier, Rapid, takes parcels up to 20 kg; the backup carrier, Steady, up to 50 kg. So the orders' own data decides what happens to them: Ann's goes through without trouble. Ben's card expired in 2024, and the bank's new card for him expires in 2029. Cat's parcel weighs 30 kg, too heavy for the main carrier. Dan's card has expired too, and his order of 1,200 is above the bank's limit, so the bank refuses the new card as well.

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

1. **In what order does a checkout do its work?** This one is easy: the function that reserves, charges and ships says it in three lines. But each of those lines hides decisions of its own, and nothing in the checkout shows them.
2. **What happens to an order when the card is declined twice?** The checkout does not say. You follow the charge into the payments module. There, `take_payment` matches on the bank's answer: a card declined because it has expired is renewed with the bank and charged once more. If the bank refuses that second charge, the refusal travels silently back up to the checkout, and a single character carries it: the `?` at the end of the second `bank.charge(...)` returns it from `take_payment`, and the `?` after `take_payment` in `reserve_charge_and_ship` passes it on, turned into a `CheckoutError` by its `From` implementation. The checkout then logs a warning and tells the customer. A card refused for any other reason is never tried again: the last arm of the `match` returns the refusal at once. To learn this you read three functions in two modules, and the bank's `Declined` enum, its list of reasons for refusing a charge. Dan's order shows the answer, but its log does not say that it was the new card the bank refused.
3. **What happens when the main carrier cannot take the parcel?** The shipping module tries the backup carrier. If the backup cannot take the parcel either, its error is thrown away: `.ok()` turns the backup's `Result` into an `Option`, and the error goes with it. The log shows only the main carrier's trouble, and the checkout reports "no carrier could take it", without saying why.
4. **Is every successful charge audited?** Only the payments module tells you. The audit happens at the end of the payment function, after the renewal. Reading the checkout, you would never know that charges are audited at all.
5. **When is the customer told, and what?** At the end of the checkout, whether the order went through or not. On a failure, the customer is told only that the order could not be completed. Why it failed is in the log, where each module writes what its author thought worth writing, in its own way: the checkout logs failures, the payments module logs renewals, the shipping module logs the switch to the backup carrier.

The code mixes two kinds of decision. **Business rules** decide what the shop does: an order is charged its price less any discount, goods are reserved before they are shipped, a parcel goes to a carrier. **Flow control** decides what happens around that work: renew the card and try once more, fall back to another carrier, give up, audit, tell the customer. Logging is woven through both. Every flow decision lives inside a business function, so the only way to find it is to read that function, and the one it calls, and the one after.

That is the answer to "why Itinera?": the hardest part of understanding a system like this is knowing what is going on. Answering one question means reading every module, and a decision is easy to miss: the audit tucked into the payment function, the backup carrier's error dropped, the second refusal that travels up without a word. Changing a decision is harder still, because you first have to find every place that makes it.

The shop is small, and you can still read all of it. A system that has grown for years, across teams and repositories, has the same problem with no one left who has read everything, and agents changing it faster than anyone can follow. The Itinera site tells that larger story on [Why Itinera](https://itinera-dev.github.io/why/).

The next chapters write the same shop with Itinera, where every flow decision has one place to live, and you can answer each of these questions by reading one declaration.

## Remember

1. A business rule says what the business does; a flow decision says what happens around it: try again, fall back, give up, tell someone. Tell them apart before you change either.
2. A flow decision made inside a business function is invisible to the code that calls it.
3. A log says only what its author chose to say. Answer questions about the flow from the code, not from the log.
