# Testing a hook or an adapter alone

## What you want to do

Some of the shop's rules live outside its steps: `failed_shipping` deciding that no retry can help and refunding the charge, the audit line, the customer's message, and the adapter taking the weight out of the parcel. You want to check each of them on its own, as [chapter 27](27-testing-a-step-alone.md) checked the steps.

## The code

The program is [chapter 25](25-reading-the-result.md)'s shop, with its tests in a module that only a test build compiles, as in [chapter 27](27-testing-a-step-alone.md). The `notify` policy tells customers through the role `Messages`, and `failed_shipping` gives money back through the role `Refunds`, so the tests bring messages and refunds of their own, which keep what they are asked to do:

```example
name: ch28_testing_a_hook_or_an_adapter_alone
region: fakes
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async
```

`failed_shipping`'s hook `before_retrying` is called for Ned's shipping, with the reason each test chooses, and the tests read the decision it returns and the refunds it asked for. Its hook `given_up` is called with the cause `retries exhausted`, as for Oli's shipping, and the test also reads the refund it contributed:

```example
name: ch28_testing_a_hook_or_an_adapter_alone
region: failed_shipping
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async
```

The audit's hook is called with the step's name, made with `step_name!`, the amount, the receipt, and a contributor lent by `Contributions`:

```example
name: ch28_testing_a_hook_or_an_adapter_alone
region: audit
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async
```

Both hooks of `notify` are called with the test's messages, for Kim's gift card and Lee's failed order:

```example
name: ch28_testing_a_hook_or_an_adapter_alone
region: notify
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async
```

The adapter is a method of the workflow, so the tests make the workflow's value, with its channel, and ask it for two of the shipping's inputs:

```example
name: ch28_testing_a_hook_or_an_adapter_alone
region: adapter
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async
```

In the asynchronous form, which tests [chapter 26](26-running-async-journeys-while-others-wait.md)'s shop, each test of a hook is an `async fn` marked `#[tokio::test]` that awaits the hook, and the fakes implement the roles with `#[async_trait]` and `async fn`, keeping what they are asked in a `Mutex` in place of a `RefCell`.

`cargo test` prints the same for both forms:

```test-run
name: ch28_testing_a_hook_or_an_adapter_alone
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async
```

## How it works

1. **A hook is a method.** The test calls it with what its parameters ask for. Their attributes, such as `#[reason]`, `#[from_workflow]` or `#[role]`, only tell the executor what to give them in a journey. Nothing else changes: the policies are the shop's own.
2. **A hook is tested through its decision.** `before_retrying` returns an `Option<FailWorkflow>`, and `given_up` one too. The trait `ReturnedLifecycle`, from `itinera::policy`, which the tests bring in with `use`, reads it: `keeps_default` says the hook returned nothing, so the executor does what it would have done without the policy, and `fails_workflow` gives the reason the hook fails the journey with, if it does. The tests give the hook every case it has to handle, including no reason at all, which is what it gets after an attempt that ended in an error.
3. **A role is faked like a service.** `notify` asks for `&dyn Messages`, and `failed_shipping` for `&dyn Refunds`, so the tests pass their own `TestMessages` and `TestRefunds`, and read what the hooks told whom and what they gave back. `refund`, the policy's own method, is tested through the two hooks that call it. A role's methods take `&self`, so each fake keeps what it is asked in a `RefCell`, Rust's way of letting a value change behind a shared reference. In the asynchronous form, the roles are `Sync`, and a `RefCell` can't be shared between threads, so the fakes use a `Mutex`, from `std::sync`, which lets several threads share a value one at a time. Its `lock` and `into_inner` fail only if a thread panicked while holding it, which the fakes never do.
4. **A hook's contributions are read as a step's.** The audit and `given_up` contribute through the contributor `Contributions` lends, and the tests read it back with `read`.
5. **An adapter is tested through what it supplies.** `parcels` returns `Some` of an `AnyValue` for the weight, which `downcast_ref` reads as the type the shipping asks for, and `None` for every other input, which leaves it to the data bag. The test gives it the parcel itself, as a journey would give it from the data bag.

So each rule of the shop's policies and its workflow is checked alone too. But nothing yet checks how the workflow puts them together: that a declined card is rescued, that Ned's shipping is given up after its first attempt, and that every customer is told. A journey checks that, and a journey runs the steps, with their bank and their carriers.

## Outcomes

We have shown how to test a policy's hooks and a workflow's input adapter alone, by calling them as methods.

| The usual way | With Itinera |
|---|---|
| Logic that runs around a call, such as a retry rule or an audit line, is often tested only by running the whole call. | A hook is a method, called by the test with the data it asks for, and its decision is a value the test reads. |
| Code that converts data for a function often sits inside it, tested only with it. | An input adapter is a method of the workflow, tested on its own with the data it reads. |
