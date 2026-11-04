# Testing a hook or an adapter alone

## What you want to do

Some of the shop's rules live outside its steps: `failedShipping` deciding that no retry can help and refunding the charge, the audit line, the customer's message, and the adapter taking the weight out of the parcel. You want to check each of them on its own, as [chapter 27](27-testing-a-step-alone.md) checked the steps.

## The code

The program is [chapter 25](25-reading-the-result.md)'s shop, with its tests in a file beside it, as in [chapter 27](27-testing-a-step-alone.md). The `notify` policy tells customers through the role `Messages`, shown in [chapter 19](19-letting-policies-act-on-the-outside-world.md), and `failedShipping` gives money back through the role `Refunds`, shown in [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md), so the tests bring messages and refunds of their own, which keep what they are asked to do in a list:

```example
name: ch28_testing_a_hook_or_an_adapter_alone.test
region: fakes
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async.test
```

The policies and the adapter are the shop's own, unchanged since they were shown: the audit in [chapter 20](20-an-optional-input.md), `Notify` in [chapter 19](19-letting-policies-act-on-the-outside-world.md), `FailedShipping` in [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md), and the adapter of [chapter 21](21-deciding-where-a-steps-data-comes-from.md), a method of the workflow class as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) shows it, and `Parcel`, which chapter 21 also shows. This chapter shows only their tests. `FailedShipping`'s hook `beforeRetrying` is called for Ned's shipping, with the reason each test chooses, and the tests read the decision it returns and the refunds it asked for. Its hook `givenUp` is called with the cause `RETRIES_EXHAUSTED`, as for Oli's shipping, and the test also reads the refund it contributed:

```example
name: ch28_testing_a_hook_or_an_adapter_alone.test
region: failed_shipping
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async.test
```

The audit's hook is called with the step's name, made by `StepName.of`, from `org.itineradev.itinera.step`, the amount, the receipt, and a contributor lent by `Contributions`:

```example
name: ch28_testing_a_hook_or_an_adapter_alone.test
region: audit
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async.test
```

Both hooks of `Notify` are called with the test's messages, for Kim's gift card and Lee's failed order:

```example
name: ch28_testing_a_hook_or_an_adapter_alone.test
region: notify
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async.test
```

The adapter is a method of the workflow class, so the tests make the workflow, with its channel, and ask it for two of the shipping's inputs:

```example
name: ch28_testing_a_hook_or_an_adapter_alone.test
region: adapter
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async.test
```

In the asynchronous form, which tests [chapter 26](26-running-async-journeys-while-others-wait.md)'s shop, the fakes' methods are `suspend` functions, as the roles' are there, and each test runs its body in `runBlocking`, which waits for the hook or the adapter it calls.

JUnit's console launcher prints the same for both forms:

```test-run
name: ch28_testing_a_hook_or_an_adapter_alone
asynchronous: ch28_testing_a_hook_or_an_adapter_alone_async
```

## How it works

1. **A hook is a method.** The test makes the policy with its constructor, which takes nothing, and calls the hook with what its parameters ask for. Their types and annotations, such as `Reason?`, `@FromWorkflow` or `@Role`, only tell the executor what to give them in a journey. Nothing else changes: the policies are the shop's own.
2. **A hook is tested through its decision.** `beforeRetrying` returns an `OnFailure`, and `givenUp` one too. `OnFailure`'s own properties read it: `keepsDefault` says the hook kept the default, so the executor does what it would have done without the policy, and `failsWorkflow` gives the reason the hook fails the journey with, or `null` when it doesn't. The tests give the hook every case it has to handle, including no reason at all, which is what it gets after an attempt that ended in an error.
3. **A role is faked like a service.** `Notify` asks for `Messages`, and `FailedShipping` for `Refunds`, so the tests pass their own `TestMessages` and `TestRefunds`, and read what the hooks told whom and what they gave back. `refund`, the policy's own private method, is tested through the two hooks that call it.
4. **A hook's contributions are read as a step's.** The audit and `givenUp` contribute through the contributor `Contributions` lends, and the tests read it back with `read`, as in [chapter 27](27-testing-a-step-alone.md).
5. **An adapter is tested through what it supplies.** `parcels` returns what `supply(...)` makes for the weight, a `Supplied.Value` holding it, and what `fromDataBag()` makes for every other input, `Supplied.FromDataBag`, which leaves the input to the data bag. Both are values the test compares, from `org.itineradev.itinera.workflow`. The test gives the adapter the parcel itself, as a journey would give it from the data bag.

So each rule of the shop's policies and its workflow is checked alone too. But nothing yet checks how the workflow puts them together: that a declined card is rescued, that Ned's shipping is given up after its first attempt, and that every customer is told. A journey checks that, and a journey runs the steps, with their bank and their carriers.

## Outcomes

We have shown how to test a policy's hooks and a workflow's input adapter alone, by calling them as methods.

| The usual way | With Itinera |
|---|---|
| Logic that runs around a call, such as a retry rule or an audit line, is often tested only by running the whole call. | A hook is a method, called by the test with the data it asks for, and its decision is a value the test reads. |
| Code that converts data for a function often sits inside it, tested only with it. | An input adapter is a method of the workflow, tested on its own with the data it reads. |
