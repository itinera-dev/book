# Letting policies act on the outside world

## What you want to do

The message for each customer is written, but nobody sends it. You want the notify policy to send it, while how it is sent, by email today and perhaps by text message tomorrow, stays the shop's business: the policy should only say "tell this customer this".

## The code

What the policy needs from the shop is an interface of its own, with one operation: tell a customer something. It is marked `@Role`, from `org.itineradev.itinera.policy`:

```example
name: ch19_letting_policies_act_on_the_outside_world
region: role
```

The notify policy's two hooks now ask for it, as a parameter marked `@Role` too, and for the customer's name from the data bag, and tell the customer instead of contributing the message:

```example
name: ch19_letting_policies_act_on_the_outside_world
region: policy
```

The workflow class implements the interface, with `Messages` beside `ItineraWorkflow`, and sends through the channel it is configured with, a property its constructor sets. Here the channel is faked, and prints what it sends.

```example
name: ch19_letting_policies_act_on_the_outside_world
region: workflow
```

The steps, policies and reporters it names are as earlier chapters show them: the charge and the audit in [chapter 15](15-reacting-to-what-a-step-did.md), `RenewCard` in [chapter 14](14-repairing-then-trying-once-more.md), the shipping, `Outage` and `HeavyParcels` in [chapter 17](17-giving-up-a-step-early.md), the log in [chapter 18](18-doing-something-when-the-journey-ends.md), with the parts that page points to, and the desk in [chapter 11](11-naming-a-journey.md). The bank's provider is [chapter 12](12-trying-again.md)'s, the interface `Payments` it lends [chapter 14](14-repairing-then-trying-once-more.md)'s, and the carriers' provider [chapter 13](13-a-carrier-for-each-attempt.md)'s, with [chapter 12](12-trying-again.md)'s `Carrier`.

Each order's data now holds its customer's name, and the output no longer holds a message:

```example
name: ch19_letting_policies_act_on_the_outside_world
region: data
```

Since each order now names its customer, the shop runs a list of orders, and gives each one whole to its instance, made from a workflow object with the channel, `email`:

```example
name: ch19_letting_policies_act_on_the_outside_world
region: create
```

It runs the same five orders as [chapter 18](18-doing-something-when-the-journey-ends.md). It prints:

```output
name: ch19_letting_policies_act_on_the_outside_world
```

## How it works

1. **A role is what a workflow offers its policies.** It is a set of operations the workflow's own code carries out, such as "tell a customer". A policy that needs to act on the outside world asks for a role, never for the workflow itself. In Kotlin, a role is an interface marked `@Role` that the workflow class implements, and a hook's parameter marked `@Role` is given an object that implements only that interface and passes each call on to the workflow, so the hook reaches the workflow through the role alone.
2. **A role carries operations, never data.** The customer's name and the order come to the hook as data from the workflow, from the data bag. The role only does something with them.
3. **A workflow's properties are its configuration.** `channel` is given to the constructor of `Orders`, as `Orders(channel = "email")`, and is never in the data bag, so no step or hook can read it as data. The constructor has no default for it, so the shop can't make the workflow object without a channel: Kotlin's compiler refuses `Orders()`.
4. **The policy knows only the role.** `Notify` tells a customer through whatever `Messages` it is given, so sending by text message would change the workflow's `tell`, and no policy. **A policy acts on the outside world only through the roles its workflow offers.**
5. **Only policies ask for roles.** Steps never do: a step does its business with its inputs and its services, and never reaches the workflow.
6. **A role's work is the workflow's business.** The executor doesn't record what a role does: the log shows `hook_called` once the hook has returned, after the email was sent, and nothing in between.

So every customer is told how their order went. The charge, though, still charges the full price of every order, while the shop gives some customers a discount that nothing in the shop applies.

## Outcomes

We have shown how a policy acts on the outside world through a role its workflow offers.

| The usual way | With Itinera |
|---|---|
| Code that sends a message usually holds the mail client itself, so changing how customers are told changes that code. | The policy asks for a role, and only the workflow, which implements it, knows how the message is sent. |
