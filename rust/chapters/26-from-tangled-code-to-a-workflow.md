# From tangled code to a workflow

## What you want to do

You have code like the checkout of [chapter 1](01-but-why-itinera.md): business rules, retries, fallbacks, audits, messages and logging, all woven together. You want to turn it into a workflow, one move at a time, so that you can do the same with your own code. This chapter rewrites that checkout, and gathers what each "Without Itinera" section of the book showed.

## The code

### 1. Sort what the code does

Read the checkout and put each thing it does in one of a few kinds. The kind says where it goes.

| What the checkout does | Kind | Where it goes |
|---|---|---|
| Reserves the goods | business rule | the step `reserve` |
| Charges the price, less any discount | business rule | the step `charge` |
| Hands the parcel to a carrier | business rule | the step `ship` |
| Reserves, then charges, then ships | flow | the order of the workflow's steps |
| Gives up when the card is refused | flow | the default: a step that fails fails the journey |
| Tries the backup carrier | flow | the retry budget of `ship`, and a provider that picks the carrier |
| Renews an expired card and charges once more | repair | a rescue step of `charge` |
| Audits every successful charge | reaction to an outcome | a step policy, `audit` |
| Tells the customer how it went | reaction to the end | a workflow policy, `notify`, through a role |
| Writes log lines | observation | a reporter |
| Talks to the bank, the warehouse and the carriers | tools | services, supplied by providers |

### 2. Turn each business rule into a step

Here is the payment code of [chapter 1](01-but-why-itinera.md) again. It holds one business rule, charging the card, and three other things: a repair, an audit and a log line.

```example
name: ch01_checkout
region: payment
```

As steps, the business rules keep only the business. The charge works out the amount and calls the bank. When the bank refuses, it doesn't renew the card, log or give up: it reports a failure with a reason, `Self::fail("expired-card")` or `Self::fail("declined")`. The ship step does the same with the carrier: when the carrier can't take the parcel, it reports a failure that might work next time, `Self::fail_retriably("carrier-down")`, and doesn't call another carrier.

For now the steps still reach the faked bank, warehouse and carrier by themselves, through constants such as `BANK`, as the tangled code did. Move 7 takes those away.

```example
name: ch26_workflow
region: steps
```

### 3. Write the order down

The order of the work, and how many times shipping may be tried, go in the workflow's declaration: one `#[with_step(...)]` line per step, in order, with `retry_budget = 1` on `ship`.

```example
name: ch26_workflow
region: workflow
```

### 4. Move the reactions into policies

The audit reacts to a successful charge, so it becomes a step policy attached to `charge`. Telling the customer reacts to the end of the journey, so it becomes a workflow policy, which reaches the customer through the `Messages` role that the workflow provides.

```example
name: ch26_workflow
region: policies
```

### 5. Replace the log lines with a reporter

The steps and policies no longer log anything. One reporter receives every event of every journey and writes, in one format, the failures and the decisions that the modules of [chapter 1](01-but-why-itinera.md) each logged in their own way. It's a plain implementation of the `Reporter` trait, listed on the workflow with `reporters = [Log]`, and the workflow makes one with `Default` for every journey. `EventBody` is `#[non_exhaustive]`, so the reporter's `match` ends with an arm for the events it doesn't handle.

```example
name: ch26_workflow
region: reporter
```

### 6. Run what you have

The same four orders as in [chapter 1](01-but-why-itinera.md):

```example
name: ch26_workflow
region: run
```

It prints:

```output
name: ch26_workflow
```

Ann's order goes through. The others fail, and the log says exactly why, without a line of logging in the steps. Ben's and Dan's charges fail with `expired-card`, because nothing renews a card yet. Cat's parcel is tried twice with the main carrier, because nothing picks the backup carrier yet. Each move of a rewrite leaves a program that compiles and runs, and its events show you what hasn't moved yet.

### 7. Give the steps their tools

The steps still reach the bank, the warehouse and the carrier by themselves. Instead, each step asks for what it uses as a service, by a key: `payments`, `stock` or `carrier`. Each service is a trait, so the step never knows which implementation it was given.

From this move on, the program is the shop of [chapter 3](03-read-the-workflow-know-what-happens.md).

```example
name: ch03_shop
region: services
```

The ship step now uses the carrier it is given, and the stock store, which it shares with `reserve`:

```example
name: ch03_shop
region: ship
```

The workflow supplies each service through a provider. The payment client and the stock store last for the whole journey. The carrier is made afresh for every attempt: the main carrier on the first, the backup carrier after. The fallback of [chapter 1](01-but-why-itinera.md) is now these few lines, together with the retry budget of `ship`.

```example
name: ch03_shop
region: providers
```

### 8. Turn the repair into a rescue step

The charge takes its payment client as a service, and its business rule does not change:

```example
name: ch03_shop
region: charge
```

The card renewal becomes a step of its own, the charge's rescue step. It reads why the charge failed, asks the bank for the new card only when the card has expired, and contributes it as `card`, which the charge reads when it runs once more.

```example
name: ch03_shop
region: rescue
```

The workflow declares the rescue step on `charge`, and the rewrite is complete:

```example
name: ch03_shop
region: workflow
```

Moves 7 and 8 use this syntax:

- A step asks for a service with a parameter of `run` marked `#[service]`, such as `#[service] carrier: &dyn Carrier`, borrowed for the attempt only.
- A provider is a method of the workflow's `#[workflow]` block marked `#[provider]`, for one instance per journey, or `#[provider(per_attempt)]`, for one per attempt. It returns a `Result<Box<dyn Trait>, Error>`, such as `Box<dyn Carrier>` for the carrier, and may request the attempt number with `#[attempt]`.
- A rescue step derives `RescueStep` instead of `Step`, and its `run` is in a `#[rescue_step]` block. Its fields can be its inputs, marked `#[input]`, data from the failed attempt of the step it rescues, marked `#[from_rescued_step]`, the reason of that failure, marked `#[reason]`, and its error, marked `#[error]`. The reason is an `Option<Reason>`, because a charge that ends in an abnormal termination has no reason, only an error. Its `run` returns `Result<RescueOutcome, Error>`, built with `Self::succeed()`, `Self::fail(...)` or `Self::fail_retriably(...)`: there's no way to skip, so a rescue step that skips can't be written in Rust.
- The workflow attaches the rescue step with `rescue = RenewCard` on the step's `#[with_step(...)]` line. A `rescue` together with a `retry_budget` above 0 on the same line won't compile.

When it runs, the orders go as they did in [chapter 1](01-but-why-itinera.md). Ben's charge fails with `expired-card`, `charge_rescue` gets his new card, and the charge's attempt 2 succeeds. Cat's parcel goes with the backup carrier on the second attempt of `ship`. Dan's charge is rescued too, but the bank refuses his new card, so the charge's attempt 2 fails and so does the journey. Each of those decisions is now in the events, and each has one place in the declaration:

```output
name: ch03_shop
```

## How it works

The rewrite follows one rule: every decision goes where its kind belongs.

1. **Business rules become steps that report.** A step does its work and reports an outcome with a reason. It never calls the next step, never tries itself again, never tells the customer and never logs what the flow did. That is what the "Without Itinera" section of [part 3](part-3-without-itinera.md) showed: a function that calls the next component itself, catches its error and decides to retry is making flow decisions in business code.
2. **The caller knows only the journey.** As the section of [part 2](part-2-without-itinera.md) showed, a checkout that calls the charge and then the shipping has to know both functions: their order, what each takes and how each fails. Now the code that takes an order creates a workflow instance, runs the journey and reads its result, however many steps the workflow has.
3. **Flow goes into the declaration.** The order of the steps, every retry budget and every rescue step are in the workflow, where you read them all at once. The section of [part 4](part-4-without-itinera.md) showed retry strategies multiplying across a code base, each with its own limits; here there is one budget per step, declared next to it.
4. **Choices about the outside world go into providers.** Which carrier ships the second attempt is a decision of the workflow, not of the ship step, so the ship step never knows there are two carriers. Part 4 showed the same fallback written inside the shipping code.
5. **Repairs become rescue steps.** The renewal is no longer hidden in a branch of the charge; it is a step of its own, and the events say when a rescue happened and how it went.
6. **Reactions become policies.** Auditing and telling the customer react to outcomes, so they are hooks, attached where they apply, and they reach the outside world through roles.
7. **Observation becomes a reporter.** As the section of [part 5](part-5-without-itinera.md) showed, logging written by hand depends on who wrote each module. Every journey now reports the same events, and a reporter decides where they go. That same stream lets you test the journey by its events, and each step and policy alone, without mocking carriers or counting calls.

Do the rewrite in moves, as this chapter did. After each move the program runs, and the events of a journey show plainly what is still missing.

## Remember

1. A step keeps its business rule and reports its outcome; it never calls the next step, tries itself again or tells anyone.
2. Every retry budget and every rescue step is declared in the workflow, next to its step.
3. A step reaches the outside world only through the services the workflow provides, and a policy only through the roles the workflow provides.
4. Logging belongs to a reporter, not to the steps and policies it observes.
