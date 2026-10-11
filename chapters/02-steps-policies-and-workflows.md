# Steps, policies and workflows

## What you want to do

You want to pull apart the two kinds of decision that chapter 1 mixed together. The charge should only charge. Auditing a charge should be something that reacts to a successful charge, not a line hidden at the end of the payment code. And one place should say what runs, in what order, and how many times it may be tried.

Itinera gives each of those its own home: a step, a policy and a workflow.

## The code

A **step** holds one business rule. This is the charge: it works out the amount, fails when there is nothing to charge, and otherwise records what it charged.

```example
name: ch02_pieces
region: step
```

A **policy** holds flow decisions. This one, `audit`, defines a hook that runs after every successful attempt of the step it is attached to. It reads what the step charged, records it, and lets the journey carry on.

```example
name: ch02_pieces
region: policy
```

A **workflow** is the plan. It lists the steps in the order they run, each under a name, with the policies attached to it and how many times it may be tried again.

```example
name: ch02_pieces
region: workflow
```

The program runs the workflow for two orders. The second has a discount as large as its price, so there is nothing to charge.

```example
name: ch02_pieces
region: run
```

It prints:

```output
name: ch02_pieces
```

```note
name: ch02_pieces
```

## How it works

1. **A step holds one business rule, and nothing else.** The charge declares the data it needs, a price and an optional discount, and does its work. It **contributes** what it learned, the amount under the key `charged`: it adds it to the journey's data, which Itinera calls the **data bag**. Then it reports an **outcome**: success; failure, with a reason such as `nothing-to-charge`; or skipped, when it decides it has nothing to do. An outcome says what happened, never what to do next. The step does not know its own name, which workflow runs it, or what comes after it. It has no way to reach any of them, so it cannot start the next step or try itself again, even by mistake.
2. **A policy holds flow decisions, through hooks.** A **hook** is a function of a policy that Itinera calls at a fixed moment. Step hooks are called after a step's outcome: `on step success`, `on step failure`, `on step retry` and `on step abnormal termination`. Workflow hooks are called when the journey ends: `on workflow success` and `on workflow failure`. A hook may read what the step contributed and contribute data of its own. A step hook then returns what should happen next: nothing, which keeps the default, or a **lifecycle**, a decision to fail the journey or, after a success, to finish it at once. The `audit` hook reads `charged`, records it under `audit`, and returns nothing, so the journey carries on. A **step policy**, like `audit`, is attached to steps; a **workflow policy** is attached to the workflow. A policy is reusable: `audit` could be attached to any step that contributes `charged`, in any workflow.
3. **A workflow is the plan.** It lists the steps in the order they run: `charge`, then `ship`. Each step has a **retry budget**, the number of times it may be tried again after its first attempt, which is 0 unless the workflow says otherwise: `ship` may be tried twice in all. The `audit` policy is attached to `charge`. The workflow is fixed before anything runs, and nothing else decides what runs or in what order.
4. **One run of the workflow for one order is a journey.** For the first order, the charge succeeds, `audit` records it, and `ship` runs. For the second, the charge fails with `nothing-to-charge`. No hook decides otherwise, so the default applies: a step that failed fails the journey. The `ship` step never runs, and `audit` is not called, since there was no success to audit. The charge did not decide any of that: it only reported that it failed.

The vocabulary used here comes from the [Itinera specification](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md), and the book keeps to it exactly: a step's outcome, a hook's decision and a journey's result are three different things.

## Remember

1. A step holds one business rule and reports what happened, never what to do next.
2. A flow decision belongs in a policy's hook or in the workflow's declaration, never in a step.
3. The steps run in the order the workflow lists them, and nothing else changes that order.
4. A hook that returns nothing keeps the default: after a failure, the journey fails.
