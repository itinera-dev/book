# Letting policies act on the outside world

## What you want to do

In chapter 17 the message for the customer was only written into the journey's data. Now you want it sent. How it is sent (by email today, perhaps by text message tomorrow) is the shop's business, not the policy's: the policy should only say "tell the customer this".

## The code

What the shop can do for its policies is a workflow role, or role for short: an interface of operations that the workflow implements and offers to its policies, here `Messages`, with one operation, `tell`.

```example
name: ch18_roles
region: role
asynchronous: ch18_roles_async
```

The `notify` policy asks for the role, as it asks for data, and tells the customer how the order went. It does not know whether the message goes by email.

```example
name: ch18_roles
region: policy
asynchronous: ch18_roles_async
```

The workflow provides the role. Its own code implements `tell`, using what the shop configured when it created the instance: here, the channel the shop uses to reach its customers. In this example, sending an email is printing a line.

```example
name: ch18_roles
region: workflow
asynchronous: ch18_roles_async
```

Each instance is created with the shop's channel, then run.

```example
name: ch18_roles
region: run
asynchronous: ch18_roles_async
```

It prints:

```output
name: ch18_roles
```

In the asynchronous form, the hooks are asynchronous, and the role can be shared across the points where they wait.

```note
name: ch18_roles
```

## How it works

1. **A role is what a workflow offers its policies.** It is an interface of operations that the workflow's own code implements, such as "tell the customer". A policy that needs to act on the outside world asks for a role, never for the workflow itself.
2. **Roles carry operations, never data.** The customer and the order come to the hook as data from the workflow, read from the data bag. The role only does something with them.
3. **The policy works with any workflow that provides its roles.** `notify` can be attached to any workflow that provides `Messages`. Attaching it to a workflow that does not is the violation `role not provided`, caught before any journey; chapter 27 shows the message.
4. **Only policies use roles.** Step policies and workflow policies may ask for them. Steps may not: a step does its business with what it is given, and never reaches the workflow.
5. **Role calls are the workflow's business.** The executor does not record them in the events. A hook that wants a call on record emits an event of its own.
6. **A role operation that fails fails the hook.** A hook cannot report an error and carry on, so an error from a role operation it calls aborts the journey with `hook failed`. A role that might fail, such as a mail server that is down, is best written to handle its own trouble, by queueing the message for later, for example.

## Remember

1. A policy reaches the outside world only through the roles it asks for.
2. A role carries operations; the data a hook needs comes from the data bag.
3. Steps never ask for roles.
4. A policy attached to a workflow that does not provide one of its roles is refused before any journey.
5. An error from a role operation aborts the journey, so a role handles its own trouble where it can.
