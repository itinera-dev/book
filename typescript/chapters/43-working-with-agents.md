# Working with agents

## What you want to do

The shop now works with coding agents, which change its steps and, sometimes, its workflow. You want an agent to change a business rule without changing how an order travels through the shop, and to know, without reading every line it wrote, whether the flow changed and how.

## The code

The program is [chapter 30](30-testing-a-whole-journey.md)'s module, with its approved paths, so it has none of what chapters 31 to 36 added to the shop. An agent was asked to cap discounts, and changed the pricing step alone:

```example
name: ch43_working_with_agents
region: price
```

Another agent proposes a change to the flow: a parcel the carrier won't collect should be given up at once, rather than tried again with the backup carrier. Before it replaces the workflow, the shop declares the proposal in its tests, beside the approved workflow of [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md), with `ship` and no retry budget, and only the main carrier:

```example
name: ch43_working_with_agents.test
region: proposed
```

The module now exports each piece the proposal names, as it exported what [chapter 30](30-testing-a-whole-journey.md)'s tests import. Those pieces are as [chapter 26](26-running-async-journeys-while-others-wait.md) shows the charge, `RenewCard`, the shipping, `Notify`, `FailedShipping`, `Payments`, `Carrier`, `Messages` and `Refunds`, [chapter 22](22-finishing-a-journey-early.md) the lookup, `AlreadyCompleted`, `OrderBook`, `ORDER_BOOK`, `Ledger` and the log, [chapter 21](21-deciding-where-a-steps-data-comes-from.md) `Parcel` and `PARCEL`, [chapter 20](20-an-optional-input.md) the audit, [chapter 17](17-giving-up-a-step-early.md) `Outage`, [chapter 7](07-giving-a-step-the-tools-it-needs.md) the tokens `PAYMENTS` and `CARRIER`, and [chapter 11](11-naming-a-journey.md) the desk.

[Chapter 30](30-testing-a-whole-journey.md)'s `pathOf` now makes its order with `orderOf`, and `proposedPathOf` runs the same order through the proposal, with the same fakes, `TestBank` and `TestCarrier`, as [chapter 30](30-testing-a-whole-journey.md) shows them:

```example
name: ch43_working_with_agents.test
region: paths
```

A test runs each approved order through both, and says which paths the proposal keeps, and what it makes of the others:

```example
name: ch43_working_with_agents.test
region: compared
```

`node --test` prints:

```test-run
name: ch43_working_with_agents
```

## How it works

1. **A business rule changes in a step, and the paths hold.** The pricing now caps the discount at half the price, and all four approved paths still pass: the step decides an amount, and the flow around it is the same. To change it, the agent read one step, which names its inputs by key and decides nothing about what comes next.
2. **A flow change shows in the paths it changes.** The proposal changes the path of the parcel no carrier collects, as intended: `ship given up: failure` after one attempt. It also changes the path of the parcel over the limit, which nobody asked for. That order was given up by `failedShipping`, in the hook it runs before a retry; with no retry budget there is no retry, so the hook never runs, and the journey now fails with `over-parcel-limit` instead of `no-carrier-can-take-it`. The test says so before any order takes that path.
3. **A path is the flow a person agreed.** An approved path changes only when a person agrees to the new one. A failing path test is not a test to fix: it is a flow change waiting for that decision, here two of them.
4. **One rule for every change to a step: it decides nothing about the flow.** A step that tries its work again, calls another service when the first fails, or tells anyone anything has taken a decision that belongs elsewhere, as [chapter 40](40-recognising-a-decision-in-the-wrong-place.md) showed. A reviewer, a person or an agent, checks every change to a step for that, and every change to a workflow, a policy or a provider against the paths.

The shop gives its agents this paragraph in its `AGENTS.md`:

> Business rules live in steps; the flow lives in the workflows, their policies and their providers. A change to a step never decides anything about the flow: it does not try its work again, fall back to another service or tell anyone, and it reports what happened as an outcome. A change to a workflow, a policy or a provider is a flow change: say so in the pull request, show which approved paths it changes, and never edit an approved path to make a test pass. A person approves every new path.

This is where the book ends, for now. Its workflows run one after the other, from start to end, in one program. Later tiers of Itinera will add moving a journey to another workflow, running a workflow from inside another, and journeys that survive a crash and resume, each with a part of this book of its own.

## Outcomes

We have shown how an agent changes a business rule without touching the flow, and how a proposed change to the flow shows in the paths it changes.

| The usual way | With Itinera |
|---|---|
| Whether a change altered the flow is often found by reading the whole diff, or in production. | A business rule changes in a step, and the approved paths say whether the flow changed. |
| A side effect of a flow change, such as a check that no longer runs, often goes unnoticed. | The paths it changes are listed by a test, for a person to approve. |
