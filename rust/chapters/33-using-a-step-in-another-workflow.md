# Using a step in another workflow

## What you want to do

The shop starts letting customers collect their orders at the counter. A collected order is reserved and charged exactly as a delivered one is, but nothing is shipped: the order is put aside in the back room, the card is charged at the counter's terminal, and an expired card isn't renewed, since the customer is standing there and can pay another way. You want a second workflow that uses the same reserve and charge steps, unchanged, and decides everything around them differently.

## The code

These are the parts both workflows share: the reserve step, the charge step and the audit policy. Nothing in them says which workflow they're in. The charge asks for a payment client, and the reservation for a stock store, by their keys; which one each gets is the workflow's decision.

```example
name: ch33_two_workflows
region: shared
```

The delivery workflow is the shop you know. Its stock comes from the warehouse, its charge goes through the card network and has a rescue step that renews an expired card, and its last step ships the order. Its stock, payments and issuer providers name the key they serve with `key = "..."`, so each provider's name can say where the service comes from, such as `warehouse`. The card's bank is lent only to the rescue step, through a provider attached to `charge_rescue`.

```example
name: ch33_two_workflows
region: deliveries
```

The collection workflow lists the same `Reserve` and `Charge`, under the same names, with the same `Audit` policy. Everything else is its own: the back room's stock, the counter's card terminal, no rescue step, `PutAside` instead of `Ship`, and a `notify` policy that tells the customer their order is waiting.

```example
name: ch33_two_workflows
region: collections
```

The program prints each workflow's listing, then runs two orders through each. Ann's card is fine; Ben's has expired.

```example
name: ch33_two_workflows
region: run
```

It prints:

```output
name: ch33_two_workflows
```

The listings show much of the two plans side by side, before anything runs: the same steps, served by different providers, with a rescue step in one and not the other. Then the journeys follow those plans. Ben's delivery renews his card and goes on; his collection fails at the charge with `expired-card`, and the counter's message asks him to pay with another card.

## How it works

1. **A step belongs to no workflow.** `Reserve` and `Charge` say what they need, inputs and services, by key, and what they report. A workflow lists them with `#[with_step(...)]` and decides the rest: their names, their order, their retry budgets, their rescue steps and the policies attached to them. Listing a step in a second workflow changes nothing in the step.
2. **The flow is the workflow's, so it can differ.** The charge has a rescue step in deliveries and none in collections. Same step, same failure, `expired-card`; what happens next is decided by the workflow that lists it, and you read it in that workflow alone.
3. **So are the services.** The charge asks for `payments` and gets the card network in one workflow and the counter's terminal in the other. The reservation asks for `stock` and gets the warehouse or the back room. Each workflow provides every service its steps ask for, or it is refused before any journey with `service not provided`, as [chapter 31](31-mistakes-caught-before-a-journey.md) shows.
4. **A rescue step's services are its own.** The bank's provider is attached to `charge_rescue`, so only the rescue step can use it. A workflow without that rescue step doesn't need the bank at all, and the collection workflow doesn't provide it.
5. **Policies are shared the same way.** `Audit` is attached to the charge in both workflows. `Delivered` and `ReadyToCollect` are two workflow policies with the same name, `notify`, each attached to one workflow; they tell the customer different things through the same `Messages` role, which each workflow provides.
6. **The listings show much of what each workflow decided.** As [chapter 2](02-the-big-picture.md) showed, a workflow's listing gives one entry per step, in order: its name, its step policies, the services it asks for with the provider that serves each, and its rescue step with that step's services. Here the two listings differ in their providers, their last step and the charge's rescue step. The rest, such as the retry budget of `ship` and each workflow's `notify` policy, is read in the two declarations.

## Remember

1. A step names what it needs and never which workflow it's in; any workflow that provides what it needs can list it.
2. Retry budgets, rescue steps, policies and providers are decided by each workflow that lists a step, never by the step.
3. Each workflow provides every service its steps and rescue steps ask for, or it is refused before any journey.
4. To compare two workflows, compare their listings, then their declarations for retry budgets and workflow policies.
