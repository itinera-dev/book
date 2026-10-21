# Letting policies act on the outside world

## What you want to do

The message for each customer is written, but nobody sends it. You want the notify policy to send it, while how it is sent, by email today and perhaps by text message tomorrow, stays the shop's business: the policy should only say "tell this customer this".

## The code

What the policy needs from the shop is an interface of its own, with one operation: tell a customer something.

```example
name: ch19_letting_policies_act_on_the_outside_world
region: role
```

The notify policy now names it as the role it asks for, with `WorkflowPolicy<Messages>`. Its two hooks ask for it, as a parameter marked `@role<Messages>()`, from `itinera/policy`, and for the customer's name from the data bag, and tell the customer instead of contributing the message:

```example
name: ch19_letting_policies_act_on_the_outside_world
region: policy
```

The workflow class implements the interface, with `implements Messages`, and sends through the channel it is configured with, a field its constructor sets. Here the channel is faked, and prints what it sends.

```example
name: ch19_letting_policies_act_on_the_outside_world
region: workflow
```

The steps, policies and reporters it names are as earlier chapters show them: the charge and the audit in [chapter 15](15-reacting-to-what-a-step-did.md), `RenewCard` in [chapter 14](14-repairing-then-trying-once-more.md), the shipping, `Outage` and `HeavyParcels` in [chapter 17](17-giving-up-a-step-early.md), the log in [chapter 18](18-doing-something-when-the-journey-ends.md) and the desk in [chapter 11](11-naming-a-journey.md). The providers are [chapter 14](14-repairing-then-trying-once-more.md)'s, with the tokens `PAYMENTS` and `CARRIER` of [chapter 7](07-giving-a-step-the-tools-it-needs.md); the interfaces `Payments` and `Carrier` are [chapter 14](14-repairing-then-trying-once-more.md)'s and [chapter 12](12-trying-again.md)'s.

The workflow object is made with the channel, `Orders.build("email")`. Each order's initial data now holds its customer's name, so each order is given whole to its instance, and the output no longer holds a message:

```example
name: ch19_letting_policies_act_on_the_outside_world
region: run
```

The rest is [chapter 18](18-doing-something-when-the-journey-ends.md)'s shop, and it runs the same five orders. It prints:

```output
name: ch19_letting_policies_act_on_the_outside_world
```

## How it works

1. **A role is what a workflow offers its policies.** It is a set of operations the workflow's own code carries out, such as "tell a customer". A policy that needs to act on the outside world asks for a role, never for the workflow itself. In TypeScript, a role is an interface the workflow class implements, and a hook's parameter marked `@role<Messages>()` is given the workflow object, seen only as a `Messages`.
2. **A policy says which roles it needs.** `WorkflowPolicy<Messages>` names them, so a hook of `Notify` can ask for `Messages`, and a workflow that lists `Notify` must implement `Messages`: TypeScript checks it where `@workflow(...)` lists the policy.
3. **A role carries operations, never data.** The customer's name and the order come to the hook as data from the workflow, from the data bag. The role only does something with them.
4. **A workflow's fields are its configuration.** `channel` is given to the workflow's constructor, through `Orders.build("email")`, and is never in the data bag, so no step or hook can read it as data. `Orders.build` takes what the constructor takes, so the shop can't make the workflow object without a channel.
5. **The policy knows only the role.** `Notify` tells a customer through whatever `Messages` it is given, so sending by text message would change the workflow's `tell`, and no policy. **A policy acts on the outside world only through the roles its workflow offers.**
6. **Only policies ask for roles.** Steps never do: a step does its business with its inputs and its services, and never reaches the workflow.
7. **A role's work is the workflow's business.** The executor doesn't record what a role does: the log shows `hook_called` once the hook has returned, after the email was sent, and nothing in between.

So every customer is told how their order went. The charge, though, still charges the full price of every order, while the shop gives some customers a discount that nothing in the shop applies.

## Outcomes

We have shown how a policy acts on the outside world through a role its workflow offers.

| The usual way | With Itinera |
|---|---|
| Code that sends a message usually holds the mail client itself, so changing how customers are told changes that code. | The policy asks for a role, and only the workflow, which implements it, knows how the message is sent. |
