# Letting policies act on the outside world

## What you want to do

The message for each customer is written, but nobody sends it. You want the notify policy to send it, while how it is sent, by email today and perhaps by text message tomorrow, stays the shop's business: the policy should only say "tell this customer this".

## The code

What the policy needs from the shop is a trait of its own, with one operation: tell a customer something.

```example
name: ch19_letting_policies_act_on_the_outside_world
region: role
```

The notify policy's two hooks now ask for it, as a parameter marked `#[role]`, and for the customer's name from the data bag, and tell the customer instead of contributing the message:

```example
name: ch19_letting_policies_act_on_the_outside_world
region: policy
```

The workflow implements the trait, in an `impl` block marked `#[role]`, and sends through the channel it is configured with, a field of the workflow's struct. Here the channel is faked, and prints what it sends.

```example
name: ch19_letting_policies_act_on_the_outside_world
region: workflow
```

Each order now starts with its customer's name, and each instance is created with the channel, `email`:

```example
name: ch19_letting_policies_act_on_the_outside_world
region: create
```

The rest is [chapter 18](18-doing-something-when-the-journey-ends.md)'s shop, and it runs the same five orders. It prints:

```output
name: ch19_letting_policies_act_on_the_outside_world
```

## How it works

1. **A role is what a workflow offers its policies.** It is a set of operations the workflow's own code carries out, such as "tell a customer". A policy that needs to act on the outside world asks for a role, never for the workflow itself. In Rust, `#[role]` on `impl Messages for Orders` makes the workflow lend itself, as a `&dyn Messages`, to every hook that asks for that role.
2. **A role carries operations, never data.** The customer's name and the order come to the hook as data from the workflow, from the data bag. The role only does something with them.
3. **A workflow's fields are its configuration.** `channel` is set on the builder of each instance, with `.channel("email")`, and is never in the data bag, so no step or hook can read it as data. `create()` doesn't compile until it is set.
4. **The policy knows only the role.** `Notify` tells a customer through whatever `Messages` it is lent, so sending by text message would change the workflow's role, and no policy. **A policy acts on the outside world only through the roles its workflow offers.**
5. **Only policies ask for roles.** Steps never do: a step does its business with its inputs and its services, and never reaches the workflow.
6. **A role's work is the workflow's business.** The executor doesn't record what a role does: the log shows `hook_called` once the hook has returned, after the email was sent, and nothing in between.

So every customer is told how their order went. The charge, though, still charges the full price of every order, while the shop gives some customers a discount that nothing in the shop applies.

## Outcomes

We have shown how a policy acts on the outside world through a role its workflow offers.

| The usual way | With Itinera |
|---|---|
| Code that sends a message usually holds the mail client itself, so changing how customers are told changes that code. | The policy asks for a role, and only the workflow, which implements it, knows how the message is sent. |
