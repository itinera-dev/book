# Letting policies act on the outside world

## What you want to do

In [chapter 17](17-doing-something-when-the-journey-ends.md) the message for the customer was only written into the journey's data. Now you want it sent. How it's sent (by email today, perhaps by text message tomorrow) is the shop's business, not the policy's: the policy should only say "tell the customer this".

## The code

What the shop can do for its policies is a workflow role, or role for short: an interface of operations that the workflow implements and offers to its policies. In Rust a role is a trait, here `Messages`, with one operation, `tell`.

```example
name: ch18_roles
region: role
asynchronous: ch18_roles_async
```

In the asynchronous form, the trait is declared `trait Messages: Sync`, because an asynchronous hook's future must be `Send`, and the role it borrows goes into that future.

The `notify` policy asks for the role with a parameter marked `#[role]`, of type `&dyn Messages`, just as it asks for data with `#[from_workflow]`, and tells the customer how the order went. It doesn't know whether the message goes by email.

```example
name: ch18_roles
region: policy
asynchronous: ch18_roles_async
```

In the asynchronous form, the hooks are `async fn`.

The workflow provides the role by implementing the trait, with `#[role]` on the `impl` block. Its own code implements `tell`, using what the shop configured when it created the instance: here, the channel the shop uses to reach its customers, a field of the workflow's struct. In this example, sending an email is printing a line.

```example
name: ch18_roles
region: workflow
asynchronous: ch18_roles_async
```

The asynchronous form declares the workflow `asynchronous`, as every workflow with asynchronous policies is.

Each instance is created with the shop's channel, then run.

```example
name: ch18_roles
region: run
asynchronous: ch18_roles_async
```

The asynchronous form runs the same journeys on `AsyncLocalExecutor`, awaiting each.

It prints:

```output
name: ch18_roles
```

## How it works

1. **A role is what a workflow offers its policies.** It's an interface of operations that the workflow's own code implements, such as "tell the customer". A policy that needs to act on the outside world asks for a role, never for the workflow itself. In Rust, `#[role]` on `impl Messages for Orders` makes the workflow hand itself, as a `&dyn Messages`, to every hook that asks for that role, which is why the trait has to be dyn compatible.
2. **Roles carry operations, never data.** The customer and the order come to the hook as data from the workflow, read from the data bag. The role only does something with them.
3. **The workflow's fields are its configuration.** `channel` is the workflow's own configuration, the same for every journey the shop starts with it, never journey data. The instance builder has one setter for each field, such as `.channel("email")`, and `create` doesn't compile until every field that is neither an `Option` nor marked `#[default]` has been set.
4. **The policy works with any workflow that provides its roles.** `notify` can be attached to any workflow that provides `Messages`. Attaching it to a workflow that doesn't is the violation `role not provided`; in Rust it doesn't even compile, and [chapter 27](27-mistakes-caught-before-a-journey.md) shows the compiler's message.
5. **Only policies use roles.** Step policies and workflow policies may ask for them. Steps may not: a step does its business with what it is given, and never reaches the workflow.
6. **Role calls are the workflow's business.** The executor doesn't record them in the events. A hook that wants a call on record emits an event of its own.
7. **A role operation that fails fails the hook.** A hook can't report an error and carry on, so an error from a role operation that the hook passes on, with `?` for example, aborts the journey with `hook failed`. A role that might fail, such as a mail server that is down, is best written to handle its own trouble, by queueing the message for later, for example.

## Remember

1. A policy reaches the outside world only through the roles it asks for.
2. A role carries operations; the data a hook needs comes from the data bag.
3. Steps never ask for roles.
4. A policy attached to a workflow that doesn't provide one of its roles doesn't compile.
5. An error from a role operation aborts the journey, so a role handles its own trouble where it can.
