# Testing a step alone

## What you want to do

The shop's business rules live in its steps: what a charge does when the bank declines, and which parcels the shipping refuses. You want to check each of those rules on its own, with no journey, no real bank and no real carrier.

## The code

The tests use recorders that come with the `testing` feature of `itinera`. You turn it on for tests only, by naming `itinera` once more under `[dev-dependencies]` in `Cargo.toml`, with `features = ["testing"]`, so the program you ship never has them.

The program is [chapter 25](25-reading-the-result.md)'s shop, unchanged, with its tests at the end, in a module of their own. `#[cfg(test)]` makes it a module that only a test build compiles: the shop's own build leaves it out, so only the tests may use the `testing` feature, and `use super::*` gives them the shop's steps:

```example
name: ch27_testing_a_step_alone
region: module
asynchronous: ch27_testing_a_step_alone_async
```

The steps are the shop's own, unchanged since they were shown: the charge in [chapter 20](20-an-optional-input.md), its rescue step in [chapter 14](14-repairing-then-trying-once-more.md) and the shipping in [chapter 17](17-giving-up-a-step-early.md). This chapter shows only their tests.

A step meets the outside world only through its services, so the tests give it fakes of their own, inside the module, implementing the services `Payments` and `Carrier` as [chapter 14](14-repairing-then-trying-once-more.md) and [chapter 12](12-trying-again.md) declared them. These two answer as the test chooses:

```example
name: ch27_testing_a_step_alone
region: fakes
asynchronous: ch27_testing_a_step_alone_async
```

Each test is a function marked `#[test]`, named after the rule it checks, which fails when an `assert!` or `assert_eq!` doesn't hold. The charge is tested by building it with Ann's order, as the executor would, and calling its `run`. `Contributions`, from `itinera::journey`, lends the contributor `run` asks for, and keeps what the step contributes, read back with `read`:

```example
name: ch27_testing_a_step_alone
region: charge
asynchronous: ch27_testing_a_step_alone_async
```

The rescue step is built the same way, with the reason the test chooses in its `#[reason]` field:

```example
name: ch27_testing_a_step_alone
region: rescue
asynchronous: ch27_testing_a_step_alone_async
```

The shipping also asks for its step reporter. `StepEventReporter`, from `itinera::step`, lends one, and keeps what the step emits through it, read back with `emitted`:

```example
name: ch27_testing_a_step_alone
region: ship
asynchronous: ch27_testing_a_step_alone_async
```

Where it differs, each example is followed by its asynchronous form, which tests [chapter 26](26-running-async-journeys-while-others-wait.md)'s shop the same way: its fakes implement the services with `#[async_trait]` and `async fn`, and each test is an `async fn` marked `#[tokio::test]`, which runs it on tokio's runtime, awaits `run`, and has the shipping's step reporter lent by `async_reporter`.

`cargo test` runs every test, and prints a line for each, the same for both forms:

```test-run
name: ch27_testing_a_step_alone
asynchronous: ch27_testing_a_step_alone_async
```

## How it works

1. **A step is a value with a method.** The test builds it by naming its inputs, and calls `run` with what its parameters ask for: a service, a contributor, a reporter. The attributes on those parameters tell the executor what to give them in a journey; in a test, you give them yourself. The steps are the shop's own, unchanged.
2. **A step is tested through its outcome.** `run` returns the outcome, and `is_success` and `reason` read it. A declined card fails the charge with `card-expired`, and a parcel of 90 kg fails the shipping with `over-parcel-limit`. That fake carrier's booking system is down, so the shipping would have returned the carrier's error had it booked anything: it refused the parcel before calling the carrier.
3. **An error escapes as it would in a journey.** When the bank is down, the charge's `run` returns an `Err`, which the executor would record as an abnormal termination, as [chapter 12](12-trying-again.md) showed. The test checks that the error escapes; what happens next is the workflow's decision, not the step's.
4. **Recorders lend what the executor would.** In a journey, the executor makes a step's contributor and step reporter for each attempt. In a test, `Contributions` and `StepEventReporter` lend them and keep what they receive. A recorder only keeps what it is given: committing a successful attempt's contributions is the executor's work.
5. **Fakes come from the test, not from the workflow.** In a journey, the workflow's providers make the bank and the carriers. Here, the test passes a fake where `run` asks for the service, so the step never knows which payment client it got. The fakes are as simple as the tests need: one answer for each case the step has to handle.
6. **Test support stays in the tests.** `Contributions` and `StepEventReporter` exist only with the `testing` feature, which only a test build has. So everything that uses them sits inside `#[cfg(test)]`, and the shop you ship compiles without them. The book builds this program both ways: as it would ship, and with its tests.

So each rule of the shop's steps is checked without running an order. But some rules live in the shop's policies and its workflow: `failed_shipping`, shown in [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md), deciding that no retry can help and refunding the charge, or the adapter of [chapter 21](21-deciding-where-a-steps-data-comes-from.md) taking the weight out of the parcel. These tests never reach them.

## Outcomes

We have shown how to test each step alone, with fakes of its services and recorders lending what the executor would.

| The usual way | With Itinera |
|---|---|
| Testing a business rule often means setting up the whole request around it, or mocking the code that calls it. | A step is built from its inputs and run alone, with fakes passed where it asks for services. |
| What a function did is often checked through what the mocks it called received. | A step's test reads the outcome it returned, and what it contributed and emitted. |
