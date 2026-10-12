# Steps, policies and workflows

## What you want to do

You want to pull apart the two kinds of decision that [chapter 1](01-but-why-itinera.md) mixed together. The charge should only charge. Stopping a suspiciously large charge before the order ships should be something that reacts to a successful charge, not a check hidden at the end of the payment code. And one place should say what runs, in what order, and how many times it may be tried.

Itinera gives each of those its own home: a step, a policy and a workflow.

## The code

A **step** holds one business rule. In Rust it is a struct and an `impl` block. This is the charge: it works out the amount, fails when there is nothing to charge, and otherwise records what it charged.

```example
name: ch02_pieces
region: step
```

A **policy** holds flow decisions. In Rust it is a type and an `impl` block, whose methods are its hooks. This one, `review`, defines a hook that runs after every successful attempt of the step it is attached to. It compares what the step charged with the shop's review limit, which the order brings in its initial data. Above the limit, it fails the journey, so nothing is shipped until a person has looked; otherwise it lets the journey carry on.

```example
name: ch02_pieces
region: policy
```

A **workflow** is the plan. It lists the steps in the order they run, each under a name, with the policies attached to it and how many times it may be tried again.

```example
name: ch02_pieces
region: workflow
```

The program runs the workflow for three orders, each with a review limit of 500. The second has a discount as large as its price, so there is nothing to charge. The third costs 900.

```example
name: ch02_pieces
region: run
```

It prints:

```output
name: ch02_pieces
```

## How it works

1. **A step holds one business rule, and nothing else.** The charge declares the data it needs, a price and an optional discount, and does its work. It **contributes** what it learned, the amount under the key `charged`, to the journey's data, which Itinera calls the **data bag**. The contribution is committed, that is written to the data bag, only when the step succeeds. Then it reports an **outcome**: success; failure, with a reason such as `nothing-to-charge`; or skipped, when it decides it has nothing to do. An outcome says what happened, never what to do next. The step does not know its own name, which workflow runs it, or what comes after it. It has no way to reach any of them, so it cannot start the next step or try itself again, even by mistake.

   In Rust, deriving `Step` makes each field marked `#[input]` an input, named after the field; a field of type `Option<T>`, such as `discount`, is an optional input. The `#[step]` attribute declares `run`, which takes the step by value and returns `Result<Outcome, Error>`, built with `Self::succeed()` or `Self::fail(...)`. Its parameter marked `#[contributor]` is how it contributes.
2. **A policy holds flow decisions, through hooks.** A **hook** is a function of a policy that Itinera calls at a fixed moment. Step hooks are called after a step's outcome: `on step success`, `on step failure`, `on step retry` and `on step abnormal termination`. Workflow hooks are called when the journey ends: `on workflow success` and `on workflow failure`. A hook may read what the step contributed and the journey's data, and may contribute data of its own. A step hook then returns what should happen next: nothing, which keeps the default, or a **lifecycle**, a decision to fail the journey or, after a success, to finish it at once. The `review` hook reads `charged` and `review_limit`, and either returns nothing, so the journey carries on, or fails it with the reason `needs-review`. A **step policy**, like `review`, is attached to steps; a **workflow policy** is attached to the workflow. A policy is reusable: `review` could be attached to any step that contributes `charged`, in any workflow.

   In Rust, a step policy is an `impl` block marked `#[step_policy(name = "review")]`, and each hook is a method marked with its name, such as `#[on_step_success]`. Each parameter of a hook says what it requests: `#[from_step]` reads what the step contributed under the parameter's name, and `#[from_workflow]` reads the journey's data under it, here the initial data. The `on step success` hook returns `Result<Option<OnSuccess>, Error>`. An `OnSuccess`, either `FinishWorkflow` or `FailWorkflow`, is a lifecycle: `Self::fail_workflow("needs-review")` returns `FailWorkflow` with its reason, and `Self::carry_on()`, which is `Ok(None)`, returns nothing and keeps the default.
3. **A workflow is the plan.** It lists the steps in the order they run: `charge`, then `ship`. Each step has a **retry budget**, the number of times it may be tried again after its first attempt, which is 0 unless the workflow says otherwise: `ship` may be tried twice in all, when its first attempt fails in a way that might work next time. The `review` policy is attached to `charge`. The workflow is fixed before anything runs, and nothing else decides what runs or in what order.

   In Rust, the workflow is a struct marked `#[workflow(name = "orders")]`, with one `#[with_step(...)]` line per step, in the order the steps run, giving its type, its name, and optionally `retry_budget` and `policies`. The macros check the declaration when it compiles: two steps with the same name, or two policies of one step defining the same hook, do not compile.
4. **One execution of the workflow for one order is a journey.** For the first order, the charge succeeds, `review` finds 35 within the limit, and `ship` runs. For the second, the charge fails with `nothing-to-charge`. The charge has no retry budget, and no hook decides otherwise, so the default applies: a step that failed, with no retry left, fails the journey. The `ship` step never runs, and `review` is not called, since there was no success to review. The charge did not decide any of that: it only reported that it failed. For the third, the charge succeeds and its contribution is committed, so `charged` reads 900; then `review` fails the journey with `needs-review`, and `ship` never runs. The charge did its work exactly as for the first order: the decision to stop came from the policy, and the program can tell which one decided.

The vocabulary used here comes from the [Itinera specification](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md), and the book keeps to it exactly: a step's outcome, a hook's lifecycle and a journey's result are three different things.

## Remember

1. A step holds one business rule and reports what happened, never what to do next.
2. A flow decision belongs in a policy's hook or in the workflow's declaration, never in a step.
3. The steps run in the order the workflow lists them, and nothing else changes that order.
4. A hook that returns nothing keeps the default: a step that fails and has no retry left fails the journey.
