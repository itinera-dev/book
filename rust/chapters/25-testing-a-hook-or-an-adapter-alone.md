# Testing a hook or an adapter alone

## What you want to do

Some of the shop's rules live outside its steps: `heavy_parcels` deciding that no retry can help, the audit line, the customer's message, and the adapter taking the weight out of the parcel. You want to check each of them on its own, as [chapter 24](24-testing-a-step-alone.md) checked the steps.

## The code

The program is [chapter 23](23-reading-the-result.md)'s shop, with checks in place of the orders. The `notify` policy tells customers through the role `Messages`, so the test brings messages of its own, which keep what they are asked to send:

```example
name: ch25_testing_a_hook_or_an_adapter_alone
region: messages
```

`heavy_parcels` is tested by calling its hook `before_retrying` with the reason the test chooses, and reading the decision it returns:

```example
name: ch25_testing_a_hook_or_an_adapter_alone
region: heavy_parcels
```

The audit's hook is called with the step's name, made with `step_name!`, the amount, the receipt, and a contributor lent by `Contributions`:

```example
name: ch25_testing_a_hook_or_an_adapter_alone
region: audit
```

Both hooks of `notify` are called with the test's messages, for Kim's gift card and Lee's failed order:

```example
name: ch25_testing_a_hook_or_an_adapter_alone
region: notify
```

The adapter is a method of the workflow, so the test makes the workflow's value, with its channel, and asks it for two of the shipping's inputs:

```example
name: ch25_testing_a_hook_or_an_adapter_alone
region: adapter
```

It prints:

```output
name: ch25_testing_a_hook_or_an_adapter_alone
```

## How it works

1. **A hook is a method.** The test calls it with what its parameters ask for. Their attributes, such as `#[reason]`, `#[from_workflow]` or `#[role]`, only tell the executor what to give them in a journey. Nothing else changes: the policies are the shop's own.
2. **A hook is tested through its decision.** `before_retrying` returns an `Option<FailWorkflow>`: `Some` gives the step up, and `None` carries on. `Reason::from` turns a `FailWorkflow` into its reason. The test gives the hook every case it has to handle, including no reason at all, which is what it gets after an attempt that ended in an error.
3. **A role is faked like a service.** `notify` asks for `&dyn Messages`, so the test passes its own `TestMessages`, and reads what the hooks told whom. A role's methods take `&self`, so the fake keeps its messages in a `RefCell`, Rust's way of letting a value change behind a shared reference.
4. **A hook's contributions are read as a step's.** The audit contributes its line through the contributor `Contributions` lends, and the test reads it back with `read`.
5. **An adapter is tested through what it supplies.** `parcels` returns `Some` of an `AnyValue` for the weight, which `downcast_ref` reads as the type the shipping asks for, and `None` for every other input, which leaves it to the data bag. The test gives it the parcel itself, as a journey would give it from the data bag.

So each rule of the shop's policies and its workflow is checked alone too. But nothing yet checks how the workflow puts them together: that a declined card is rescued, that Ned's shipping is given up after its first attempt, and that every customer is told. A journey checks that, and a journey runs the steps, with their bank and their carriers.

## Outcomes

We have shown how to test a policy's hooks and a workflow's input adapter alone, by calling them as methods.

| The usual way | With Itinera |
|---|---|
| Logic that runs around a call, such as a retry rule or an audit line, is often tested only by running the whole call. | A hook is a method, called by the test with the data it asks for, and its decision is a value the test reads. |
| Code that converts data for a function often sits inside it, tested only with it. | An input adapter is a method of the workflow, tested on its own with the data it reads. |
