# Giving a step the tools it needs

## What you want to do

The shop's steps talk to the outside world: the charge to a payment client, the reservation and the shipping to the warehouse's stock store, the shipping to a carrier. You want each step to ask for the tool it needs and use it, while the workflow decides which tool it gets and how long that tool lasts. The stock store must be the same for the reservation and the shipping, so that the shipping finds what was reserved; the carrier must be fresh for every attempt.

## The code

Each tool is a trait that says what it can do, and nothing more. These traits are all a step knows of them. Each is `Send + Sync`, as `itinera` asks of a service's trait.

```example
name: ch12_services
region: services
```

Each step asks for the tools it uses as parameters of `run` marked `#[service]`, next to its contributor. The reservation asks for the stock store; the charge for the payment client; the shipping for the stock store and the carrier.

```example
name: ch12_services
region: steps
```

The workflow lists the steps, and provides each tool from a method of its own `impl` block. The stock store is made once for the journey and serves only the reservation and the shipping; the payment client is made once for the journey; the carrier is made anew for every attempt.

```example
name: ch12_services
region: workflow
```

You run the journey as before.

```example
name: ch12_services
region: run
```

It prints what each tool does, in order: the warehouse opens one stock store, which reserves the order; the bank charges 40 to the customer's card; the shop books the carrier for the shipping's attempt, the carrier collects the order, and the same stock store hands it over. The journey succeeds, with the receipt `R-ann-card-40` and the tracking number `T-A-1`.

```output
name: ch12_services
```

## How it works

1. **A service is a tool, not data.** A payment client, a stock store or a carrier is an object a step uses to do its business. It isn't a value: it never enters the data bag, an event or the result. In Rust it is a trait object, such as `dyn StockStore`.
2. **A step asks for a service by a key.** As it asks for an input, a step asks for a service by a key and by what it must be able to do: the parameter `#[service] stock: &dyn StockStore` asks for the key `stock`, the parameter's name, and the trait `StockStore`. `#[service("key")]` names another key. A step always asks for a trait, never for a concrete type such as a struct: a `#[service]` parameter of a concrete type does not compile, so a step never depends on one implementation. A service is always required. The step uses what it gets without knowing which implementation it is, or where it came from.
3. **The workflow provides each service.** For each key, the workflow has a provider: part of the workflow, which makes the service when a step needs it. A provider is a method marked `#[provider]` in the workflow's `#[workflow]` block, and its key is the method's name, unless `key = "..."` names it. It returns `Result<Box<dyn StockStore>, Error>`. Since it takes `&self`, it can use the workflow's own fields, such as a connection pool.
4. **A missing provider is caught before any journey.** A workflow that lists a step asking for a key it doesn't provide is refused before any journey, with the violation `service not provided`: a violation is a mistake in the workflow's declaration, found before any journey can start. With the macros, Rust catches it earlier still: the program doesn't compile. Nor do two providers of one key serving the same step, or a provider whose type differs from what a step asks for.
5. **The provider decides how long a service lasts.** A provider makes either one instance for the whole journey, or a fresh one for every attempt:
   1. **For the whole journey,** with `#[provider]`: the provider is called the first time a step asks for the key, and every later attempt of every step that asks for it receives that same instance. The journey's stock store is opened once, used by the reservation and then by the shipping, which finds the order the reservation made. A journey that never asks for a key never makes its service.
   2. **For every attempt,** with `#[provider(per_attempt)]`: the provider is called for each attempt of each step that asks for the key, retries included, and that instance serves that attempt only. Each attempt of the shipping gets a fresh booking with the carrier, so a failed attempt's booking is never reused. [Chapter 13](13-trying-again.md) uses this to give the second attempt the backup carrier.

   Each instance is dropped when the journey or the attempt it serves ends, so a client that must close its connection does so in its `Drop`.
6. **A provider can serve only some steps.** The stock store's provider says `steps = ["reserve", "ship"]`, so it serves those two steps only. No other step of this workflow asks for the stock store, so here the attachment only makes plain who uses it. It matters when a workflow has both a provider attached to some steps and one for every step, under the same key: the attached one takes the place of the other for its steps.
7. **The step has the service only for its attempt.** The service is lent to the step for its attempt, as its contributor is. In Rust it's a reference that borrows the attempt, so the step can't keep it once the attempt ends, and the step's struct needs no lifetime. When the attempt ends, the provider's decision about the service's lifetime holds.
8. **What a service answers is the step's to handle.** What a step does with a service is its business, and the executor doesn't record it. Here each step turns its service's answer into an outcome, and for Ann's order every answer is a success; [chapter 13](13-trying-again.md) runs an order whose carrier can't take the parcel, which the shipping reports as a failure that might work next time. An error from a service that the step lets escape with `?` is an abnormal termination, as [chapter 11](11-when-something-goes-wrong-inside-a-step.md) shows. A provider that returns `Err`, on the other hand, means the step can't be built, and the journey is aborted with `step could not be built`.
9. **Only steps ask for services.** Policies act on the outside world through roles instead, as [chapter 18](18-letting-policies-act-on-the-outside-world.md) shows.

## Remember

1. A step asks for a service by key and uses it without knowing which implementation it got; which one, and how long it lasts, are the workflow's decisions.
2. A service is never a value: it never goes in the data bag, and what a step learns from it is contributed as values.
3. A step uses a service only during its attempt, and never keeps it.
4. Every service a step asks for has a provider in the workflow, or the workflow is refused before any journey.
