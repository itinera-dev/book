# Working with agents

## What you want to do

The shop now works with coding agents, which change its steps and, sometimes, its workflow. You want an agent to change a business rule without changing how an order travels through the shop, and to know, without reading every line it wrote, whether the flow changed and how.

## The code

The program is [chapter 30](30-testing-a-whole-journey.md)'s, with its approved paths, so it has none of what chapters 31 to 35 added to the shop. An agent was asked to cap discounts, and changed the pricing step alone:

```example
name: ch42_working_with_agents
region: price
asynchronous: ch42_working_with_agents_async
```

Another agent proposes a change to the flow: a parcel the carrier won't collect should be given up at once, rather than tried again with the backup carrier. Before it replaces the workflow, the shop declares the proposal in its tests, beside the approved workflow of [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md), with `ship` and no retry budget:

```example
name: ch42_working_with_agents
region: proposed
asynchronous: ch42_working_with_agents_async
```

What it names is as earlier chapters show it: the lookup, `already_completed` and the log in [chapter 22](22-finishing-a-journey-early.md), the charge and the audit in [chapter 20](20-an-optional-input.md), `RenewCard` in [chapter 14](14-repairing-then-trying-once-more.md), the shipping and `outage` in [chapter 17](17-giving-up-a-step-early.md), `failed_shipping` in [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md), `notify` in [chapter 19](19-letting-policies-act-on-the-outside-world.md) and the desk in [chapter 11](11-naming-a-journey.md).

A test runs each approved order through both, and says which paths the proposal keeps, and what it makes of the others:

```example
name: ch42_working_with_agents
region: compared
asynchronous: ch42_working_with_agents_async
```

In the asynchronous form, the pricing's `run` is an `async fn`, and the test is an `async fn` marked `#[tokio::test]`, which awaits each path. The tests print:

```test-run
name: ch42_working_with_agents
asynchronous: ch42_working_with_agents_async
```

## How it works

1. **A business rule changes in a step, and the paths hold.** The pricing now caps the discount at half the price, and all four approved paths still pass: the step decides an amount, and the flow around it is the same. To change it, the agent read one step, which names its inputs by key and decides nothing about what comes next.
2. **A flow change shows in the paths it changes.** The proposal changes the path of the parcel no carrier collects, as intended: `ship given up: failure` after one attempt. It also changes the path of the parcel over the limit, which nobody asked for. That order was given up by `failed_shipping`, in the hook it runs before a retry; with no retry budget there is no retry, so the hook never runs, and the journey now fails with `over-parcel-limit` instead of `no-carrier-can-take-it`. The test says so before any order takes that path.
3. **A path is the flow a person agreed.** An approved path changes only when a person agrees to the new one. A failing path test is not a test to fix: it is a flow change waiting for that decision, here two of them.
4. **One rule for every change to a step: it decides nothing about the flow.** A step that tries its work again, calls another service when the first fails, or tells anyone anything has taken a decision that belongs elsewhere, as [chapter 39](39-deciding-where-a-decision-belongs.md) showed. A reviewer, a person or an agent, checks every change to a step for that, and every change to a workflow, a policy or a provider against the paths.

The shop gives its agents this paragraph in its `AGENTS.md`:

> Business rules live in steps; the flow lives in the workflows, their policies and their providers. A change to a step never decides anything about the flow: it does not try its work again, fall back to another service or tell anyone, and it reports what happened as an outcome. A change to a workflow, a policy or a provider is a flow change: say so in the pull request, show which approved paths it changes, and never edit an approved path to make a test pass. A person approves every new path.

This is where the book ends, for now. Its workflows run one after the other, from start to end, in one program. Later tiers of Itinera will add moving a journey to another workflow, running a workflow from inside another, and journeys that survive a crash and resume, each with a part of this book of its own.

## Outcomes

We have shown how an agent changes a business rule without touching the flow, and how a proposed change to the flow shows in the paths it changes.

| The usual way | With Itinera |
|---|---|
| Whether a change altered the flow is often found by reading the whole diff, or in production. | A business rule changes in a step, and the approved paths say whether the flow changed. |
| A side effect of a flow change, such as a check that no longer runs, often goes unnoticed. | The paths it changes are listed by a test, for a person to approve. |
