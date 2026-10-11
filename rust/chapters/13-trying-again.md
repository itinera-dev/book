# Trying again

## What you want to do

Carriers have bad days. When the shop's main carrier is down, you want shipping to be tried once more, through the backup carrier, before the order is given up. When an order cannot be shipped at all, because it has no address, trying again would not help, and you want it given up at once.

## The code

The ship step ships through the carrier it is given. An order without an address is a failure that trying again would not fix. A carrier that is down is a failure that might go away on the next attempt, so the step marks it retriable.

```example
name: ch13_trying_again
region: step
```

In this first version, the step is not lent a carrier as in chapter 12: it reads the carrier's name from the journey's data, under `carrier`. No order starts with one, so the first attempt uses the main carrier. The backup carrier is named by a step policy, `fallback`: an object attached to the step, whose hooks the executor calls after the step's outcome. Chapter 15 says more about policies. This one has a single hook, `on step retry`, called when the ship step is about to be tried again.

```example
name: ch13_trying_again
region: policy
```

The workflow gives the ship step a retry budget of 1, and attaches the policy to it.

```example
name: ch13_trying_again
region: workflow
```

Two orders go through: one whose main carrier is down, and one with no address. A small reporter prints each event as it happens; chapter 23 shows how to write one.

```example
name: ch13_trying_again
region: run
```

It prints:

```output
name: ch13_trying_again
```

The step can know even less. With services, the ship step asks for a carrier and uses whatever it is lent.

```example
name: ch13_backup_carrier
region: step
```

The workflow lends it through a provider that makes a fresh carrier for every attempt, and picks it from the attempt number: the main carrier on attempt 1, the backup carrier after. The ship step never knows there are two.

```example
name: ch13_backup_carrier
region: workflow
```

The retry budget is part of the step's line in the workflow: `#[with_step(Ship, name = "ship", retry_budget = 1)]`. Adding `abnormal_termination_retriable` to the same line lets an `Err` returned from `run` be retried within that budget. A step marks a failure retriable with `Self::fail_retriably(...)`; `Self::fail(...)` is never retried.

A step policy is a type whose `impl` block carries `#[step_policy(name = "...")]`, and each hook is a method marked with its name, here `#[on_step_retry]`. The retry hook returns `Result<Option<FailWorkflow>, Error>`: `Self::carry_on()` lets the retry happen, and `Self::fail_workflow(...)` stops it. The executor builds the policy with `Default`, so it derives `Default`. An optional input is a field of type `Option<T>`, which is `None` when the key is absent.

The second version of the ship step uses services, which `itinera` does not have yet, so it is not compiled. It follows the agreed design: the step takes `#[service] carrier: &dyn Carrier` as a parameter of `run`, borrowed for the attempt only, and the workflow supplies it from a method marked `#[provider(per_attempt)]`, which may take `#[attempt] attempt: NonZeroU32`.

## How it works

1. **An attempt is one run of a step.** The first is attempt 1, and every attempt after it is a retry, except the run that follows a rescue step, which chapter 14 shows. Each attempt builds a fresh step from the data of the moment, so nothing carries over from a failed attempt but what was committed to the data bag.
2. **The retry budget is how many retries a step allows.** It is declared for each step in the workflow, and is 0 unless you say otherwise. With a budget of 1, the ship step runs at most twice.
3. **Only a retriable failure is retried.** The step says whether trying again might help, and the workflow decides whether it will. The order without an address fails without the retriable flag: the step is given up at once, with the cause `failure`, even though its budget is untouched.
4. **An abnormal termination is not retried, unless the step's declaration allows it.** An error the step did not expect may already have done something, so by default it ends the step at once. When the workflow declares that a step's abnormal termination may be retried, it uses the same budget as a retriable failure.
5. **The `on step retry` hook runs before the next attempt.** A hook is a function of a policy that the executor calls after an outcome. Here, `fallback` contributes `carrier` with the value `backup`. A hook's contributions are committed as soon as it returns, so the second attempt is built with them. The hook could also return `FailWorkflow` to stop trying and fail the journey at once.
6. **The events tell the whole story.** The failed attempt is a fact, `step_failed`, with its reason and its retriable flag. Then comes the hook, then the decision, `step_retrying`, then the next attempt. When no retry is left, the decision is `step_given_up` with the cause `retries exhausted`, and `on step retry` is not called, since nothing would be tried again.
7. **Retries happen at once.** The executor never waits between attempts. A step that needs to wait for a carrier waits before it reports its failure.
8. **With a provider, the choice belongs to the workflow.** The provider is part of the workflow, like the step's retry budget, and is called for each attempt. It could read the attempt number, as here, or data that a retry hook contributed. Either way, the ship step only ships.

Try giving the ship step a retry budget of 0: the first order is given up after its first attempt, with the cause `retries exhausted`, the policy is never called, and the journey fails.

## Remember

1. A step says whether a failure is retriable; the workflow's retry budget decides whether it is retried.
2. A step with a retry budget of N runs at most N + 1 times, or twice when it has a rescue step, since the run after a rescue is not a retry; the budget is 0 unless the workflow declares it.
3. A failure that is not retriable is never retried, whatever the budget.
4. An abnormal termination is retried only when the step's declaration allows it, and it then uses the same budget.
5. Every attempt builds a fresh step: a failed attempt's contributions never reach the next one, and a retry hook's contributions do.
6. `on step retry` is called only when another attempt will be made.
