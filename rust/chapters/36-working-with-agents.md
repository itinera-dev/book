# Working with agents

## What you want to do

You work with coding agents, or they work for you. You want an agent to change the shop's business rules without changing how an order travels through it, and you want to know, without reading every line it wrote, whether it did.

## The code

An Itinera codebase splits every change into one of two kinds, and each kind lives in its own place:

1. **A change to a business rule** is a change to a step: what it needs, what it does, what it contributes and how it reports what happened. The charge, for instance, can learn to apply a discount without anything else moving.
2. **A change to the flow** is a change to a workflow and its policies: the order of the steps, their retry budgets, their rescue steps, their providers, and what happens after each outcome.

The shop makes the second kind visible with a few tests that pin its **paths**: for a given order, the outcomes and decisions its journey goes through, in order. A person approves each path once, and writes it down:

```example
name: ch35_approved_paths
region: paths
```

`RecordingReporter`, from [chapter 28](28-testing-a-whole-journey.md), keeps a journey's events, and `on_the_path` keeps the ones on its path: each outcome of a step, each decision about one, and how the journey ended. Everything else, such as which data was committed, is left out, because it belongs to the steps.

```example
name: ch35_approved_paths
region: recorder
```

Each check runs one order with faked services and compares the path its journey went through with the approved one.

```example
name: ch35_approved_paths
region: checks
```

It prints:

```output
name: ch35_approved_paths
```

A change to a business rule, such as charging the price less a discount, leaves both paths as they are, so both checks still pass: the steps changed, the paths did not. A retry budget of 0 for `ship` in the workflow's declaration changes the second path: `ship` is given up after its first failure and the journey fails, so the second check fails and prints that path instead. The flow changed, and the test says so.

## How it works

1. **What an agent needs to change a step is the step.** A step names its inputs and services by key, and reports an outcome. It never calls another step and never decides what happens next, so an agent changing it reads the step, and the step's own tests check it alone, as [chapter 25](25-testing-a-step-alone.md) shows.
2. **What an agent needs to change the flow is one workflow.** Every flow decision is in the workflow's declaration, its policies and its providers, or is a default that applies because the declaration says nothing. Its listing, which [chapter 2](02-the-big-picture.md) prints, shows the steps in order, their step policies, their services and providers, and their rescue steps, without running anything; retry budgets and workflow policies are read in the declaration.
3. **Neither change needs the other.** A business rule changes without touching the workflow, and the flow changes without touching a step. When a change touches both, it is two changes, and a reviewer can ask for them separately.
4. **One rule to check on every change: a step decides nothing about the flow.** A step that tries its work again inside itself, calls a second carrier when the first fails, or decides which step comes next has taken a flow decision. A reviewer, a person or an agent, looks for those in every change to a step. [Chapter 30](30-step-rescue-step-or-policy.md) lists what belongs where.
5. **Paths make flow changes hard to miss.** A path test fails when a flow change alters the path of an order it runs, and passes for a change to a business rule that keeps the steps' outcomes. It sees only the orders it runs: the retry budget of `ship` raised from 1 to 2 leaves both paths above unchanged, so pin a path for every case that matters to the business. A failing path test is not a broken test to fix: it is a flow change waiting for a person to approve the new path.
6. **With the default dispatcher, every reporter receives the same events, in order.** The path is read from events the executor sends, so the test needs nothing from the steps: no logging, no counters, no mocks that know how the code is built.

The shop gives its agents a paragraph like this one in its `AGENTS.md`:

> Business rules live in steps; the flow lives in the workflows and their policies. A change to a step never takes a flow decision: it does not try its work again, fall back to another service or decide which step comes next, and it reports what happened as an outcome. A change to a workflow, a policy or a provider is a flow change: say so in the pull request, and never edit an approved path to make a test pass. A path test that fails means the flow changed, which a person approves.

## Remember

1. A business rule changes in a step; the flow changes in a workflow and its policies. A change that touches both is two changes.
2. A step decides nothing about the flow.
3. An approved path changes only when a person approves the new flow.
