# Giving a step the tools it needs

## What you want to do

The charge talks to the bank, and the shipping to the carrier. You want each step to ask for the tool it needs and use it, without knowing which bank or which carrier it is, while the workflow decides which one it gets.

## The code

Each tool is an interface that says what the tool can do, and nothing more: a payment client that charges, a carrier that books and collects. These interfaces are all a step knows of its tools.

```example
name: ch07_services
region: services
```

The shop's bank and carrier are still faked, now as classes implementing each interface, and print what they do as before. The steps are those of [chapter 6](06-passing-data-on.md), with one change: each asks for its tool as a parameter of `run` marked `@Service`, next to its contributor, and calls the tool instead of a function of its own.

```example
name: ch07_services
region: steps
```

The workflow lists the same steps, and its class gets a body: two methods marked `@Provider`, each making one tool. Each prints when the shop gets its tool.

```example
name: ch07_services
region: workflow
```

The shop runs Ann's and Gil's orders as in chapter 6, with its `NewOrder` and `Order` unchanged, and reads back the same output. It prints:

```output
name: ch07_services
```

## How it works

1. **A tool a step uses is a service.** A payment client or a carrier is something a step uses to do its business, and in Itinera it is a **service**. It isn't a value: it never goes into the data bag. In Kotlin it is an object, of the interface the step asks for.
2. **A step asks for a service by a key, and by its type.** The parameter `@Service payments: Payments` asks for the service under the key `payments`, the parameter's name, and says it must be a `Payments`. The step asks for the interface, never for the shop's bank itself: it uses what it gets without knowing which implementation it is, or where it came from.
3. **The workflow provides each service.** For every key a step asks for, the workflow has a **provider**: a method of the workflow's class marked `@Provider`, from `org.itineradev.itinera.service`, which makes the service. Its key is the method's name, as for `carrier`, unless `key` names another, as `bank` provides `payments`. A provider returns `provide(...)` with the object it made, a method that `ItineraWorkflow` gives the class, and its return type, `Provided<Payments>`, makes the compiler check that the object is a `Payments`. **A step decides what it does with a service; the workflow decides which service it gets.** To charge through another bank, you change the provider, and no step.
4. **A service is made once for each journey, the first time a step asks for it.** The output shows when. In Ann's journey, the bank is connected just before her charge, and the carrier called just before her shipping. Gil's journey connects to the bank and calls the carrier again: every journey gets services of its own, and lets them go when it ends.
5. **The step has the service only for its attempt.** The service is lent to the step for its attempt, as its contributor is. That is why it is a parameter of `run`, and never of the constructor: the step's properties hold its data, and nothing it borrows.
6. **What a service answers is the step's to handle.** The carrier says it won't collect Gil's parcel of 60 kg, and the shipping turns that answer into its outcome, a failure with the reason `too-heavy`, as it did in chapter 6. The executor never sees what a service answers, only the outcome the step reports.

So the steps no longer reach the bank or the carrier on their own, and the workflow says which ones they use. Now look at the output again. Every line of it comes from a `println` in the shop's own code: the steps, the faked tools and the providers print what they do, and `main` prints how each journey ended. What the executor did in between, such as when it ran each step and which step's failure ended Gil's journey, the shop sees only where its own code happens to print it.

## Outcomes

We have shown how a step asks for the tools it needs, and how the workflow provides them.

| The usual way | With Itinera |
|---|---|
| A function often creates its clients, or is handed them by whoever builds it, so which bank an order uses is decided wherever that happens to be. | A step asks for a service by the interface it needs, and the workflow's provider decides which one it gets. |
| A client is often opened and closed by hand, around the calls that use it. | Each journey gets services of its own, made the first time a step asks and let go when the journey ends. |
