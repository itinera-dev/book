# Testing a hook or an adapter alone

## What you want to do

Some of the shop's rules live outside its steps: `failedShipping` deciding that no retry can help and refunding the charge, the audit line, the customer's message, and the adapter taking the weight out of the parcel. You want to check each of them on its own, as [chapter 27](27-testing-a-step-alone.md) checked the steps.

## The code

The program is [chapter 26](26-running-async-journeys-while-others-wait.md)'s shop as a module, with its tests in a file beside it that imports what it tests, as in [chapter 27](27-testing-a-step-alone.md). The `notify` policy tells customers through the role `Messages`, and `failedShipping` gives money back through the role `Refunds`, both returning promises as [chapter 26](26-running-async-journeys-while-others-wait.md) shows them, so the tests bring messages and refunds of their own, which keep what they are asked to do:

```example
name: ch28_testing_a_hook_or_an_adapter_alone.test
region: fakes
```

The policies and the adapter are the shop's own, unchanged since they were shown: the audit in [chapter 20](20-an-optional-input.md), `notify` and `failedShipping` in [chapter 26](26-running-async-journeys-while-others-wait.md), and the adapter in [chapter 21](21-deciding-where-a-steps-data-comes-from.md), which also shows `Parcel`, as the workflow class `Orders` still has it in [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md). This chapter shows only their tests. `failedShipping`'s hook `beforeRetrying` is called for Ned's shipping, with the reason each test chooses, and the tests read the decision it returns and the refunds it asked for. Its hook `givenUp` is called with the cause `retries exhausted`, as for Oli's shipping, and the test also reads the refund it contributed:

```example
name: ch28_testing_a_hook_or_an_adapter_alone.test
region: failed_shipping
```

The audit's hook is called with the step's name, the amount, the receipt, and a contributor lent by `Contributions`:

```example
name: ch28_testing_a_hook_or_an_adapter_alone.test
region: audit
```

Both hooks of `notify` are called with the test's messages, for Kim's gift card and Lee's failed order:

```example
name: ch28_testing_a_hook_or_an_adapter_alone.test
region: notify
```

The adapter is a method of the workflow, so the tests make the workflow object with its channel, as [chapter 19](19-letting-policies-act-on-the-outside-world.md) showed, and ask it for two of the shipping's inputs:

```example
name: ch28_testing_a_hook_or_an_adapter_alone.test
region: adapter
```

`node --test` prints:

```test-run
name: ch28_testing_a_hook_or_an_adapter_alone
```

## How it works

1. **A hook is a method.** The test makes the policy with `new` and calls the hook with what its parameters ask for, in order. Their decorators, such as `@failureReason({ optional: true })`, `@fromWorkflow(...)` or `@role<Refunds>()`, only tell the executor what to give them in a journey. Nothing else changes: the policies are the shop's own.
2. **A hook is tested through its decision.** `beforeRetrying` resolves to an `OnFailure`, and `givenUp` to one too. A decision is an object whose `kind` says which it is, as an outcome's does: `this.keepDefault()` makes `{ kind: "KeepDefault" }`, so the executor does what it would have done without the policy, and `this.failWorkflow(...)` makes a `FailWorkflow` that holds its `reason`. The tests give the hook every case it has to handle, including no reason at all, `None`, which is what it gets after an attempt that ended in an error.
3. **A role is faked like a service.** `notify` asks for a `Messages`, and `failedShipping` for a `Refunds`, so the tests pass their own `TestMessages` and `TestRefunds`, and read what the hooks told whom and what they gave back. `refund`, the policy's own private method, is tested through the two hooks that call it.
4. **A hook's contributions are read as a step's.** The audit and `givenUp` contribute through the contributor `Contributions` lends, and the tests read it back with `read`, as in [chapter 27](27-testing-a-step-alone.md).
5. **A name is checked where the workflow is declared.** The audit asks for the step's name, a `StepName`: a string Itinera checked when the workflow was declared, so TypeScript won't take a plain string for one. A test that calls the hook has no workflow to check it, so it writes `"charge" as StepName`, which tells TypeScript to take the string as one.
6. **An adapter is tested through what it supplies.** `parcels` returns what `this.supply(...)` makes for the weight, `{ kind: "supplied", value: 60 }`, the value the shipping gets, and what `this.fromDataBag()` makes for every other input, `{ kind: "fromDataBag" }`, which leaves it to the data bag. The test gives it the parcel itself, as a journey would give it from the data bag.
7. **A test awaits an asynchronous hook.** `failedShipping`'s and `notify`'s hooks return promises, so their tests are `async` and await them. The audit's hook is synchronous, so its test calls it and reads what it contributed at once.

So each rule of the shop's policies and its workflow is checked alone too. But nothing yet checks how the workflow puts them together: that a declined card is rescued, that Ned's shipping is given up after its first attempt, and that every customer is told. A journey checks that, and a journey runs the steps, with their bank and their carriers.

## Outcomes

We have shown how to test a policy's hooks and a workflow's input adapter alone, by calling them as methods.

| The usual way | With Itinera |
|---|---|
| Logic that runs around a call, such as a retry rule or an audit line, is often tested only by running the whole call. | A hook is a method, called by the test with the data it asks for, and its decision is a value the test reads. |
| Code that converts data for a function often sits inside it, tested only with it. | An input adapter is a method of the workflow, tested on its own with the data it reads. |
