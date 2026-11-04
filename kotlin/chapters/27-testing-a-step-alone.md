# Testing a step alone

## What you want to do

The shop's business rules live in its steps: what a charge does when the bank declines, and which parcels the shipping refuses. You want to check each of those rules on its own, with no journey, no real bank and no real carrier.

## The code

The tests use Itinera's test support, an artifact of its own, `itinera-testing`. You add it for tests only, with no version, as [chapter 3](03-installing-itinera.md) added `itinera-kotlin`: with Gradle, `testImplementation("org.itinera-dev:itinera-testing")`, and with Maven, with `<scope>test</scope>`.

The program is [chapter 25](25-reading-the-result.md)'s shop, unchanged. Its tests are in a file of their own ending in `.test.kt`; in your project they would go in `src/test/kotlin`. They use JUnit Jupiter, the JVM's usual test framework, added for tests in the same way, and are in the shop's package, so they use its steps without importing them. Each test is a method of a class, here `StepsTest`, marked `@Test`:

```example
name: ch27_testing_a_step_alone.test
region: tests
asynchronous: ch27_testing_a_step_alone_async.test
```

The steps are the shop's own, unchanged, as shown: the charge in [chapter 20](20-an-optional-input.md), its rescue step in [chapter 14](14-repairing-then-trying-once-more.md) and the shipping in [chapter 17](17-giving-up-a-step-early.md). This chapter shows only their tests.

A step meets the outside world only through its services, so the tests give it fakes of their own, implementing `Payments` and `Carrier` as [chapter 14](14-repairing-then-trying-once-more.md) and [chapter 12](12-trying-again.md) declared them, and throwing [chapter 12](12-trying-again.md)'s errors. Each is an enum, whose constants answer as the test chooses:

```example
name: ch27_testing_a_step_alone.test
region: fakes
asynchronous: ch27_testing_a_step_alone_async.test
```

A test's name, in backticks, says the rule it checks, and the test fails when an assertion, such as `assertEquals`, doesn't hold. The charge is tested by building it with Ann's order and calling its `run`. `Contributions`, from `org.itineradev.itinera.journey.testing`, lends the contributor `run` asks for, and keeps what the step contributes. Its `read` works as the data bag's does in [chapter 11](11-naming-a-journey.md), with the type to read made by `typeToken<String>()`, from `org.itineradev.itinera.value.kotlin`:

```example
name: ch27_testing_a_step_alone.test
region: charge
asynchronous: ch27_testing_a_step_alone_async.test
```

The rescue step is built the same way, with the reason the test chooses:

```example
name: ch27_testing_a_step_alone.test
region: rescue
asynchronous: ch27_testing_a_step_alone_async.test
```

The shipping also asks for its step reporter. `StepEventReporter`, from `org.itineradev.itinera.step.testing`, lends one, and keeps what the step emits through it, in `emitted`:

```example
name: ch27_testing_a_step_alone.test
region: ship
asynchronous: ch27_testing_a_step_alone_async.test
```

Where it differs, each example is followed by its asynchronous form, which tests [chapter 26](26-running-async-journeys-while-others-wait.md)'s shop the same way: its fakes' methods are `suspend` functions, each test runs its body in `runBlocking`, and the shipping's step reporter is lent by `AsyncStepEventReporter`.

JUnit's console launcher runs the tests, and prints a line as each starts and ends, then a count. It prints the same for both forms:

```test-run
name: ch27_testing_a_step_alone
asynchronous: ch27_testing_a_step_alone_async
```

## How it works

1. **A step is an object with a method.** The test builds it with its constructor, naming its inputs, and calls `run` with what its parameters ask for: a service, a contributor, a reporter. Their types and annotations, such as `@Service`, tell the executor what to give them in a journey; in a test, you give them yourself.
2. **A step is tested through its outcome.** The test compares the outcome `run` returns with `Outcome.success()` or `Outcome.failure("card-expired")`, the values a step's `succeed()` and `fail(...)` return, or with `RescueOutcome`'s, for the rescue step. A declined card fails the charge with `card-expired`, and a parcel of 90 kg fails the shipping with `over-parcel-limit`. That fake carrier is down, so the test passing shows that the shipping refused the parcel without calling it.
3. **An error escapes as it would in a journey.** When the bank is down, the charge's `run` throws `NetworkDown`, which the executor would record as an abnormal termination, as [chapter 12](12-trying-again.md) showed. `assertThrows<NetworkDown>` checks that the call throws it; what happens next is the workflow's decision, not the step's.
4. **Recorders lend what the executor would.** In a journey, the executor makes a step's contributor and step reporter for each attempt; in a test, `Contributions` and `StepEventReporter` lend them and keep what they receive. A recorder only keeps what it is given; committing a successful attempt's contributions is the executor's work.
5. **Fakes come from the test, not from the workflow.** In a journey, the workflow's providers make the bank and the carriers. Here, the test passes a fake where `run` asks for the service.
6. **Test support stays in the tests.** `Contributions` and `StepEventReporter` are in `itinera-testing`, which only the tests have. So only the test file uses them, and the book builds this program without its tests too, as it would ship.

So each rule of the shop's steps is checked without running an order. But some rules live in its policies and its workflow: `FailedShipping`, shown in [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md), deciding that no retry can help and refunding the charge, or the adapter of [chapter 21](21-deciding-where-a-steps-data-comes-from.md), a method of the workflow class as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) shows it, taking the weight out of the parcel. These tests never reach them.

## Outcomes

We have shown how to test each step alone, with fakes of its services and recorders lending what the executor would.

| The usual way | With Itinera |
|---|---|
| Testing a business rule often means setting up the whole request around it, or mocking the code that calls it. | A step is built from its inputs and run alone, with fakes passed where it asks for services. |
| What a function did is often checked through what the mocks it called received. | A step's test reads the outcome it returned, and what it contributed and emitted. |
