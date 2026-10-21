# Testing a step alone

## What you want to do

The shop's business rules live in its steps: what a charge does when the bank declines, and which parcels the shipping refuses. You want to check each of those rules on its own, with no journey, no real bank and no real carrier.

## The code

The program is [chapter 26](26-running-async-journeys-while-others-wait.md)'s shop, with `export` before its workflow and each piece its tests use, and without its run of the day's orders, since importing a module runs its top-level code. Its tests sit beside it, in a file ending in `.test.ts`, as TypeScript projects usually keep them, and import the shop's pieces and the recorders of Itinera's test support, from `itinera/testing`. **Only tests import `itinera/testing`,** so the program you ship never uses it.

```example
name: ch27_testing_a_step_alone.test
region: imports
```

The steps are the shop's own, unchanged since [chapter 26](26-running-async-journeys-while-others-wait.md) showed the charge, its rescue step and the shipping. This chapter shows only their tests.

A step meets the outside world only through its services, so the tests bring fakes of their own, implementing `Payments` and `Carrier` as [chapter 26](26-running-async-journeys-while-others-wait.md) declared them, and throwing the errors of [chapter 12](12-trying-again.md). Each answers as its test chooses, and `implements` has TypeScript check that it has every method the service declares:

```example
name: ch27_testing_a_step_alone.test
region: fakes
```

Each test is a function given to `test`, from `node:test`, Node's own test runner, named after the rule it checks. It fails when it throws, as `assert`, from `node:assert/strict`, does when what it checks doesn't hold. The charge is tested by building it with Ann's order, as the executor would, and calling its `run`. `Contributions` lends the contributor `run` asks for, as `contributor`, and keeps what the step contributes:

```example
name: ch27_testing_a_step_alone.test
region: charge
```

The rescue step is built the same way, with the reason the test chooses:

```example
name: ch27_testing_a_step_alone.test
region: rescue
```

The shipping also asks for its step reporter. `StepEventReporter` lends one, as `reporter`, and keeps what the step emits through it, in `emitted`:

```example
name: ch27_testing_a_step_alone.test
region: ship
```

Once `tsc` has compiled them, `node --test` runs the tests, and prints a line for each, with how long it took:

```test-run
name: ch27_testing_a_step_alone
```

## How it works

1. **A step is an object with a method.** The test builds it with `new`, giving its constructor the inputs in order, and calls `run` with what its parameters ask for: a service, a contributor, a reporter. Their decorators tell the executor what to give them in a journey; in a test, you give them yourself. The rescue step's `@failureReason()` gets a `Reason` from the test, an object with a `code`.
2. **A step is tested through its outcome.** `run` resolves to the outcome, an object whose `kind` is `success`, `failure` or `skipped`, with its `reason` when it has one, which `assert.deepEqual` compares with the one expected. A declined card fails the charge with `card-expired`, and a parcel of 90 kg fails the shipping with `over-parcel-limit`. Its fake carrier is down, so the test passing shows that the shipping refused the parcel without calling it.
3. **What a step contributed is read like the data bag.** `Contributions`'s `read` works as the data bag's does in [chapter 11](11-naming-a-journey.md), and its `.option()` is compared with `Some` or `None`, from `itinera/option`, the two kinds of [chapter 6](06-passing-data-on.md)'s `Option`.
4. **An error escapes as it would in a journey.** When the bank is down, the charge's `run` rejects with `NetworkDown`, which the executor would record as an abnormal termination, as [chapter 12](12-trying-again.md) showed. `assert.rejects` checks that the promise rejects with an error of that class; what happens next is the workflow's decision, not the step's.
5. **The tests await the steps.** These steps are `async`, so each test is an `async` function that awaits `run`, and Node's test runner awaits the test. A synchronous step is tested the same way, without `await`.
6. **Recorders lend what the executor would.** In a journey, the executor makes a step's contributor and step reporter for each attempt; in a test, `Contributions` and `StepEventReporter` lend them and keep what they receive. A recorder only keeps what it is given: committing a successful attempt's contributions is the executor's work.
7. **Fakes come from the test, not from the workflow.** In a journey, the workflow's providers make the bank and the carriers. Here, the test passes a fake where `run` asks for the service, so the step never knows which payment client it got.

So each rule of the shop's steps is checked without running an order. But some rules live in the shop's policies and its workflow: `failedShipping`, taught in [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) and shown as it is now in [chapter 26](26-running-async-journeys-while-others-wait.md), deciding that no retry can help and refunding the charge, or the adapter of [chapter 21](21-deciding-where-a-steps-data-comes-from.md), a method of the workflow class as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) shows it, taking the weight out of the parcel. These tests never reach them.

## Outcomes

We have shown how to test each step alone, with fakes of its services and recorders lending what the executor would.

| The usual way | With Itinera |
|---|---|
| Testing a business rule often means setting up the whole request around it, or mocking the code that calls it. | A step is built from its inputs and run alone, with fakes passed where it asks for services. |
| What a function did is often checked through what the mocks it called received. | A step's test reads the outcome it returned, and what it contributed and emitted. |
