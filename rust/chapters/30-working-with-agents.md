# Working with agents

## What you want to do

You work with coding agents, or they work for you. You want an agent to change the shop's business rules without changing how an order travels through it, and you want to know, without reading every line it wrote, whether it did.

## The code

An Itinera codebase splits every change into one of two kinds, and each kind lives in its own place:

1. **A change to a business rule** is a change to a step: what it needs, what it does, what it contributes and how it reports what happened. The charge, for instance, can learn to apply a discount without anything else moving.
2. **A change to the flow** is a change to a workflow and its policies: the order of the steps, their retry budgets, their rescue steps, their providers, and what happens after each outcome.

The shop makes the second kind visible with a few tests that pin its **routes**: for a given order, the order of outcomes and decisions its journey goes through. A person approves each route once, and writes it down:

```example
name: ch30_approved_routes
region: routes
```

A small reporter keeps the route of a journey: each outcome of a step, each decision about one, and how the journey ended. Everything else, such as which data was committed, is left out, because it belongs to the steps.

```example
name: ch30_approved_routes
region: recorder
```

Each check runs one order with faked services and compares the route it took with the approved one.

```example
name: ch30_approved_routes
region: checks
```

It prints:

```output
name: ch30_approved_routes
```

Try changing a business rule, such as charging the price less a discount, and both checks still pass: the steps changed, the routes did not. Now try giving `ship` a retry budget of 0 in the workflow's declaration: the second check fails, printing the route the journey took instead, in which `ship` is given up after its first failure and the journey fails. The flow changed, and the test says so.

## How it works

1. **What an agent needs to change a step is the step.** A step names its inputs and services by key, and reports an outcome. It never calls another step and never decides what happens next, so an agent changing it reads one file, and its tests check it alone, as [chapter 24](24-testing-your-workflows.md) shows.
2. **What an agent needs to change the flow is one workflow.** Every flow decision is in the workflow's declaration, its policies and its providers, or is a default that applies because the declaration says nothing. Its listing, which [chapter 29](29-using-a-step-in-another-workflow.md) prints, shows the steps, policies, services and rescue steps without running anything.
3. **Neither change needs the other.** A business rule changes without touching the workflow, and the flow changes without touching a step. When a change touches both, it is two changes, and a reviewer can ask for them separately.
4. **One rule to check on every change: a step decides nothing about the flow.** A step that retries inside itself, calls a second carrier when the first fails, or skips the rest of the journey has taken a flow decision. A reviewer, a person or an agent, looks for those in every diff to a step. [Chapter 25](25-step-rescue-step-or-policy.md) lists what belongs where.
5. **Routes make flow changes impossible to miss.** The route tests pass for every change to a business rule that keeps the steps' outcomes, and fail for a change to the flow. A failing route test is not a broken test to fix: it is a flow change waiting for a person to approve the new route.
6. **The events are the same for every journey.** The route is read from events the executor sends to every reporter, in a fixed order, so the test needs nothing from the steps: no logging, no counters, no mocks that know how the code is built.

The shop gives its agents a paragraph like this one in its `AGENTS.md`:

> Business rules live in steps; the flow lives in the workflows and their policies. A change to a step never takes a flow decision: it does not retry, fall back, skip the rest of the journey or call another step, and it reports what happened as an outcome. A change to a workflow, a policy or a provider is a flow change: say so in the pull request, and never edit an approved route to make a test pass. Route tests that fail mean the flow changed, which a person approves.

## Remember

1. A business rule changes in a step; the flow changes in a workflow and its policies. A change that touches both is two changes.
2. A step decides nothing about the flow.
3. An approved route changes only when a person approves the new flow.
