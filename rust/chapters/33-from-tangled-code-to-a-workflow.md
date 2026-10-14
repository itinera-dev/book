# From tangled code to a workflow

## What you want to do

You have a system that works, and grew the way systems do: business rules, retries, fallbacks, audits, messages and logging, woven together across modules, and nobody can say for sure how a request travels through it. You don't want to rewrite it. You want to bring Itinera in one route at a time, starting with the route that hurts most, keeping the clients and services you already have, with a program that runs after every move.

This chapter does that with the shop's checkout of [chapter 1](01-but-why-itinera.md), the shop's one route, and gathers what each "Without Itinera" section of the book showed. Your system has more routes; each one is adopted the same way, while the others keep running as they are.

## The code

### 1. Sort what the code does

Pick one route: here, the checkout. Read it and put each thing it does in one of a few kinds. The kind says where it goes.

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

As steps, the business rules keep only the business. The charge works out the amount and calls the bank. When the bank refuses, it doesn't renew the card, log or give up: it reports a failure with a reason, `Self::fail("expired-card")` or `Self::fail("declined")`. The ship step does the same with the carrier: when the carrier can't take the parcel, it reports a failure, `Self::fail("parcel-refused")`, and doesn't call another carrier: the workflow's retry budget decides whether it is tried again.

For now the steps still reach the faked bank, warehouse and carrier by themselves, through constants such as `BANK`, as the tangled code did. Move 7 takes those away.

```example
name: ch33_workflow
region: steps
```

### 3. Write the order down

The order of the work, and how many times shipping may be tried, go in the workflow's declaration: one `#[with_step(...)]` line per step, in order, with `retry_budget = 1` on `ship`.

```example
name: ch33_workflow
region: workflow
```

### 4. Move the reactions into policies

The audit reacts to a successful charge, so it becomes a step policy attached to `charge`, which reports each charge as an event of the journey. Telling the customer reacts to the end of the journey, so it becomes a workflow policy, which reaches the customer through the `Messages` role that the workflow provides.

```example
name: ch33_workflow
region: policies
```

### 5. Replace the log lines with a reporter

The steps and policies no longer write log lines. One reporter receives every event of every journey and writes, in one format, the failures, the decisions and the audit that the modules of [chapter 1](01-but-why-itinera.md) each logged in their own way. It's a plain implementation of the `Reporter` trait, listed on the workflow with `reporters = [Log]`, and the workflow makes one with `Default` for every journey. `EventBody` is `#[non_exhaustive]`, so the reporter's `match` ends with an arm for the events it doesn't handle.

```example
name: ch33_workflow
region: reporter
```

### 6. Run what you have

The same four orders as in [chapter 1](01-but-why-itinera.md):

```example
name: ch33_workflow
region: run
```

It prints:

```output
name: ch33_workflow
```

Ann's order goes through. The others fail, and the log says exactly why, without a line of logging in the steps. Ben's and Dan's charges fail with `expired-card`, because nothing renews a card yet. Cat's parcel is tried twice with the main carrier, because nothing picks the backup carrier yet. Each move of a rewrite leaves a program that compiles and runs, and its events show you what hasn't moved yet.

### 7. Give the steps their tools

The steps still reach the bank, the warehouse and the carrier by themselves. Instead, each step asks for what it uses as a service, by a key: `payments`, `stock` or `carrier`. Each service is a trait, so the step never knows which implementation it was given. In your system, those traits wrap the clients you already have, such as your payment gateway's client, rather than replacing them: the client stays, and only who decides which one a step gets moves into the workflow.

From this move on, the program is the shop of [chapter 2](02-the-big-picture.md).

```example
name: ch02_big_picture
region: services
```

The ship step now uses the carrier it is given, and the stock store, which it shares with `reserve`:

```example
name: ch02_big_picture
region: ship
```

The workflow supplies each service through a provider. The payment client and the stock store last for the whole journey. The carrier is made afresh for every attempt: the main carrier on the first, the backup carrier after. The fallback of [chapter 1](01-but-why-itinera.md) is now these few lines, together with the retry budget of `ship`.

```example
name: ch02_big_picture
region: providers
```

### 8. Turn the repair into a rescue step

The charge takes its payment client as a service, and its business rule does not change:

```example
name: ch02_big_picture
region: charge
```

The card renewal becomes a step of its own, the charge's rescue step. It reads why the charge failed, asks the bank for the new card only when the card has expired, and contributes it as `card`, which the charge reads when it runs once more.

```example
name: ch02_big_picture
region: rescue
```

The workflow declares the rescue step on `charge`, and the rewrite is complete:

```example
name: ch02_big_picture
region: workflow
```

Move 7 uses the services and providers of [chapter 7](07-giving-a-step-the-tools-it-needs.md), with the per-attempt provider of [chapter 13](13-a-carrier-for-each-attempt.md), and move 8 the rescue step of [chapter 14](14-repairing-then-trying-once-more.md).

When it runs, the orders go as they did in [chapter 1](01-but-why-itinera.md). Ben's charge fails with `expired-card`, `charge_rescue` gets Ben's new card, and the charge's attempt 2 succeeds. Cat's parcel goes with the backup carrier on the second attempt of `ship`. Dan's charge is rescued too, but the bank refuses Dan's new card, because the order of 1,200 is above its limit, so the charge's attempt 2 fails and so does the journey. Every successful charge, Ben's after the rescue included, shows up as an audit line. Each of those decisions is now in the events, and each has one place in the declaration. The listing printed first shows the steps, the rescue step and the providers; the retry budget of `ship` is read in the declaration:

```output
name: ch02_big_picture
```

## How it works

The rewrite follows one rule: every decision goes where its kind belongs.

1. **One route at a time.** A route becomes a workflow while the rest of the system keeps calling what it called before: the code that took an order keeps its place, and now creates a workflow instance and runs a journey. Nothing else has to move with it, so you adopt the next route when you're ready, and each route you adopt can be read in one declaration.
2. **Business rules become steps that report.** A step does its work and reports an outcome with a reason. It never calls the next step, never tries itself again, never tells the customer and never logs what the flow did. A function that calls the next component itself, catches its error and decides to retry is making flow decisions in business code.
3. **The caller knows only the journey.** A checkout that calls the charge and then the shipping has to know both functions: their order, what each takes and how each fails. Now the code that takes an order creates a workflow instance, runs the journey and reads its result, however many steps the workflow has.
4. **Flow goes into the declaration.** The order of the steps, every retry budget and every rescue step are in the workflow, where you read them all at once. Here there is one budget per step, declared next to it.
5. **Choices about the outside world go into providers.** Which carrier ships the second attempt is a decision of the workflow, not of the ship step, so the ship step never knows there are two carriers.
6. **Repairs become rescue steps.** The renewal is no longer hidden in a branch of the charge; it is a step of its own, and the events say when a rescue happened and how it went.
7. **Reactions become policies.** Auditing and telling the customer react to outcomes, so they are hooks, attached where they apply, and they reach the outside world through roles.
8. **Observation becomes a reporter.** As the section of [part 4](part-5-without-itinera.md) showed, logging written by hand depends on who wrote each module. Every journey now reports its events the same way, whoever wrote each module, and a reporter decides where they go. That same stream lets you test the journey by its events, and each step and policy alone, without mocking carriers or counting calls.

Do the rewrite in moves, as this chapter did. After each move the program runs, and the events of a journey show plainly what is still missing.

## Remember

1. A step keeps its business rule and reports its outcome; it never calls the next step, tries itself again or tells anyone.
2. Every retry budget and every rescue step is declared in the workflow, next to its step.
3. A step reaches the outside world only through the services the workflow provides, and a policy only through the roles the workflow provides.
4. Logging belongs to a reporter, not to the steps and policies it observes.
5. Adopt one route at a time, keeping your clients behind services, and keep the program running after every move.
