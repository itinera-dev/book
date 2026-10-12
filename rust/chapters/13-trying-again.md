# Trying again

## What you want to do

Carriers have bad days. When the shop's main carrier is down, you want shipping to be tried once more, through the backup carrier, before the order is given up. When an order can't be shipped at all, because it has no address, trying again wouldn't help, and you want it given up at once.

## The code

The ship step ships through the carrier it is given. An order without an address is a failure that trying again wouldn't fix, so the step returns `Self::fail("no-address")`. A carrier that is down is a failure that might go away on the next attempt, so the step marks it retriable with `Self::fail_retriably("carrier-down")`.

```example
name: ch13_trying_again
region: step
```

In this first version, the step isn't lent a carrier as in [chapter 12](12-giving-a-step-the-tools-it-needs.md): it reads the carrier's name from the journey's data, through the optional input `carrier`, an `Option<String>`. No order starts with one, so the first attempt uses the main carrier. The backup carrier is named by a step policy, `fallback`: an object attached to the step, whose hooks the executor calls after the step's outcome. [Chapter 15](15-reacting-to-what-a-step-did.md) says more about policies. In Rust, a step policy is a type whose `impl` block carries `#[step_policy(name = "...")]`. This one has a single hook, a method marked `#[on_step_retry]`, called when the ship step is about to be tried again. The executor builds the policy with `Default`, so `Fallback` derives it.

```example
name: ch13_trying_again
region: policy
```

The workflow gives the ship step a retry budget of 1, and attaches the policy to it, both on the step's `#[with_step(...)]` line.

```example
name: ch13_trying_again
region: workflow
```

Two orders go through: one whose main carrier is down, and one with no address. A small reporter prints each event as it happens; [chapter 23](23-seeing-what-happened.md) shows how to write one.

```example
name: ch13_trying_again
region: run
```

It prints:

```output
name: ch13_trying_again
```

The step can know even less. With services, the ship step asks for a carrier with `#[service] carrier: &dyn Carrier` and uses whatever it is lent, for its attempt only.

```example
name: ch13_backup_carrier
region: step
```

The workflow lends the carrier through a provider marked `#[provider(per_attempt)]`, which makes a fresh carrier for every attempt. It asks for the attempt number with `#[attempt] attempt: NonZeroU32`, and picks the carrier from it: the main carrier on attempt 1, the backup carrier after. The ship step never knows there are two.

```example
name: ch13_backup_carrier
region: workflow
```

The program's main carrier is down, so the first attempt fails and the second, with the backup carrier, ships the order:

```output
name: ch13_backup_carrier
```

## How it works

1. **An attempt is one execution of a step.** The first is attempt 1, which is why the attempt number is a `NonZeroU32`, and every attempt after it is a retry, except the attempt that follows a rescue step, which [chapter 14](14-repairing-then-trying-once-more.md) shows. Each attempt builds a fresh step from the data of the moment, so nothing carries over from a failed attempt but what was committed to the data bag.
2. **The retry budget is how many retries a step allows.** It is declared for each step in the workflow, with `retry_budget = N` on the step's line, and is 0 unless you say otherwise. With a budget of 1, the ship step runs at most twice.
3. **Only a retriable failure is retried.** The step says whether trying again might help, and the workflow decides whether it will. The order without an address fails through `Self::fail`, without the retriable flag: the step is given up at once, with the cause `failure`, even though its budget is untouched.
4. **An abnormal termination isn't retried, unless the step's line allows it.** An error the step didn't expect may already have done something, so by default an `Err` from `run` ends the step at once. Adding `abnormal_termination_retriable` to the step's `#[with_step(...)]` line declares that it may be retried, and it then uses the same budget as a retriable failure.
5. **The `on step retry` hook runs before the next attempt.** A hook is a function of a policy that the executor calls after an outcome. Here, `fallback` contributes `carrier` with the value `backup`, through the contributor it asks for as a parameter. A hook's contributions are committed as soon as it returns, so the second attempt is built with them. The retry hook returns `Result<Option<FailWorkflow>, Error>`: `Self::carry_on()` lets the retry happen, as here, and `Self::fail_workflow(...)` would stop trying and fail the journey at once.
6. **The events tell the whole story.** The failed attempt is a fact, `step_failed`, with its reason and its retriable flag. Then comes the hook, then the decision, `step_retrying`, then the next attempt. When no retry is left, the decision is `step_given_up` with the cause `retries exhausted`, and `on step retry` isn't called, since nothing would be tried again.
7. **Retries happen at once.** The executor never waits between attempts. A step that needs to wait for a carrier waits before it reports its failure.
8. **With a provider, the choice belongs to the workflow.** The provider is part of the workflow, like the step's retry budget, and is called for each attempt. It can read the attempt number, as here, or data that a retry hook contributed. Its carriers are the workflow's own fields, `main` and `backup`, which the instance builder sets. Either way, the ship step only ships.

Try giving the ship step a retry budget of 0: the first order is given up after its first attempt, with the cause `retries exhausted`, the policy is never called, and the journey fails.

## Remember

1. A step says whether a failure is retriable; the workflow's retry budget decides whether it is retried.
2. A step with a retry budget of N runs at most N + 1 times, or twice when it has a rescue step, since the attempt after a rescue isn't a retry; the budget is 0 unless the workflow declares it.
3. A failure that isn't retriable, from `Self::fail`, is never retried, whatever the budget.
4. An abnormal termination is retried only when the step's line says `abnormal_termination_retriable`, and it then uses the same budget.
5. Every attempt builds a fresh step: a failed attempt's contributions never reach the next one, and a retry hook's contributions do.
6. `on step retry` is called only when another attempt will be made.
