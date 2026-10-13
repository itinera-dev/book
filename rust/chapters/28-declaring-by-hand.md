# Declaring by hand

## What you want to do

So far you've declared the shop in the short form: the attributes and derives that write most of the declaration for you. You want to see what that short form stands for, write the same workflow in the long form, and know when the long form is the one you need.

## The code

### The shop in the short form

Here is the shop in the short form you've used so far: a charge and a ship step, the `audit` and `notify` policies, the `Messages` role, and the workflow that puts them together.

```example
name: ch28_short_form
region: step
```

```example
name: ch28_short_form
region: policies
```

```example
name: ch28_short_form
region: workflow
```

```example
name: ch28_short_form
region: data
```

It lists the workflow's steps, then runs one order:

```example
name: ch28_short_form
region: run
```

It prints:

```output
name: ch28_short_form
```

### The same shop by hand

Now the same shop, by hand. A step is two things: the step itself, which runs once and implements `Step`, and what builds it for each attempt, which implements `StepFactory`. The factory says what the step needs, as a `StepNeeds`, and builds the step from what the workflow resolved. A step that needs nothing can be a plain function.

```example
name: ch28_by_hand
region: step
asynchronous: ch28_by_hand_async
```

A policy implements the trait of each of its hooks, such as `OnStepSuccess<Orders>`: it says what the hook requests, and reads what it requested when it is called. The role is a trait the workflow implements, and the workflow says it provides it by implementing `Provides<dyn Messages>`; a hook then asks for it by its type.

```example
name: ch28_by_hand
region: policies
asynchronous: ch28_by_hand_async
```

The workflow is declared part by part: each step under its name, with its retry budget and its policies, then the workflow's own policies. Building it checks the declaration, and gives you either the workflow or every violation found.

```example
name: ch28_by_hand
region: workflow
asynchronous: ch28_by_hand_async
```

Reading the output by hand means implementing `Output` yourself, reading each key from the data bag.

```example
name: ch28_by_hand
region: data
```

Running it is the same, except that the instance comes from the descriptor, and its initial data is given key by key.

```example
name: ch28_by_hand
region: run
asynchronous: ch28_by_hand_async
```

The asynchronous form changes the step, the hooks and the executor to their asynchronous kinds, and declares the workflow asynchronous; everything else is the same. The step implements `AsyncStep` and its factory `AsyncStepFactory`, the hooks implement `AsyncOnStepSuccess` and `AsyncOnWorkflowSuccess`, the descriptors are made with `new_async` and `WorkflowDescriptor::async_builder`, and the instance runs on `AsyncLocalExecutor`. An asynchronous hook's future must be `Send`, and the role it borrows goes into that future, which is why `Messages` is declared `trait Messages: Sync` there.

It prints exactly what the short form printed:

```output
name: ch28_by_hand
```

### A workflow put together wrongly

This declaration of the shop has three mistakes: the ship step is listed twice under the same name, two policies attached to the charge both react to its success, and the pricing adapter is attached to `charging`, a step the workflow doesn't have.

Declared with `#[workflow(...)]`, these mistakes wouldn't compile, as [chapter 27](27-mistakes-caught-before-a-journey.md) shows. Declared by hand, the program compiles, and the mistakes appear as data when the workflow is built. Its steps and policies are declared with the attributes and go into the builder through their `descriptor`, as item 7 below explains, and `pricing_for` makes the `InputAdapterDescriptor` of [chapter 20](20-deciding-where-a-steps-data-comes-from.md)'s pricing adapter, attached to the step it is given.

```example
name: ch28_violations
region: wrong
```

Building it doesn't give you a workflow. The builder's `build()` returns `Result<WorkflowDescriptor<W>, Violations>`, and the `Err` holds every violation found, which you can read one by one.

```example
name: ch28_violations
region: report
```

`Violations` holds the violations in a fixed order, and each `Violation` displays as the specification names it, followed by what it concerns. When your code needs to tell them apart, `kind()` gives its `ViolationKind`.

Two mistakes never become a violation in the long form. Two input adapters of one name make the builder panic while the workflow is declared, with a message saying what was wrong. An empty step name doesn't compile, since `step_name!` checks it.

Fixed, the declaration lists each step once, keeps one policy for the charge's success, and attaches the adapter to `charge`.

```example
name: ch28_violations
region: fixed
```

It prints:

```output
name: ch28_violations
```

## How it works

1. **The short form writes descriptors.** Whatever form you write in, a workflow is a workflow descriptor: its name, a step descriptor for each step in order, its workflow policies, its input adapters and the roles it provides. The short form writes the descriptors from your types and their attributes; in the long form you write them yourself. The executor doesn't know or care which form made them, and the short-form and long-form shops above behave the same and print the same.
2. **Each attribute stands for public traits and builders.** The short form is the attributes and derives of the `macros` feature, which is on by default, and the long form is what they expand to:
   - `#[derive(Step)]` implements `FromInputs`: what the step needs, as a `StepNeeds`, and how it is built from a `Resolved`. `#[step]` implements `DescribedStep`, whose `descriptor(name)` makes the `StepDescriptor`, and turns each handle marked in `run` into a need, such as `.contributor()`. By hand, a `StepFactory` does both, and the step implements `Step`. A closure or function returning `Result<Outcome, Error>` is already a factory of a step that needs nothing.
   - `#[step_policy]` and `#[workflow_policy]` implement the trait of each hook, such as `OnStepSuccess<W>`, whose `needs()` lists what the hook requests and whose method reads it from `Requested`. They also implement `StepPolicy<W>` or `WorkflowPolicy<W>`, whose `descriptor()` is a `StepPolicyDescriptor` or `WorkflowPolicyDescriptor` declaring each hook. A role requested by a hook is a bound on the workflow, `Provides<dyn Messages>`, which `#[role]` implements.
   - `#[workflow(...)]` builds a `WorkflowDescriptor` with `WorkflowDescriptor::builder` the first time it is asked for, keeps it in a static, and gives the workflow `builder()` for its instances.
   - `#[derive(InitialData)]` and `#[derive(Output)]` implement `InitialData` and `Output`. By hand, the instance's builder takes each key with `.data(key, value)`, and `Output::from_data` reads each key from the `DataBag`.
3. **A step descriptor** holds the step's name, how to build the step, its retry budget, whether an abnormal termination may be retried, the step policies attached to it, its input adapter, if any, and its rescue step, if any. How to build the step means saying what it needs, its inputs, its services and its handles, which are its contributor and step reporter, and building it, for each attempt, from what was resolved. Nothing carries over from one attempt to the next, because each attempt gets a new step. That's also why the step in the long form borrows its contributor with a lifetime, `Charge<'a>`: it lives for one attempt only.
4. **A policy descriptor** holds the policy's name, the hooks it defines, what each hook requests, and how to build an instance of it, here the closure `|| Audit`. Workflow policies are built once per journey, step policies for every attempt.
5. **Building the workflow is the first half of admission.** In the long form you see it happen: `build()` checks the declaration, and returns every violation found as `Violations`, as the shop with three mistakes above shows; [chapter 27](27-mistakes-caught-before-a-journey.md) lists every violation and how to fix it. The short form runs the same checks while the program compiles, so a workflow declared with `#[workflow(...)]` that holds a violation doesn't compile.
6. **The listing reads the descriptor.** Both shops, short and long, list the workflow's steps, in order, with their policies, without running anything. A workflow is fixed before any journey, whichever form declared it.
7. **The two forms mix.** A step or policy declared in the short form goes into a builder through its `descriptor`, as `Charge::descriptor(step_name!("charge"))` and `Audit::descriptor()`. You can write most of a workflow in the short form and only the part that needs it in the long form.
8. **When you need the long form:**
   - a declaration put together when the program starts, from the shop's configuration, for example adding a gift-wrapping step only in the shops that offer it. It's still fixed before any journey starts;
   - seeing every violation as data when the workflow is built, rather than as a compile error;
   - a step, policy or workflow that is generic, which the attributes refuse;
   - one policy attached to several steps and reading a different key on each, such as an audit that reads `charged` on the charge and `refunded` on a refund, with `on_step_success_needing`;
   - code that would rather not depend on the macros at all, built with the `macros` feature turned off.

   Otherwise, the short form says the same thing in fewer lines, and is easier to read as the plan of the business.

## Remember

1. Both forms make the same workflow descriptor, and the executor runs either the same way.
2. A step is built afresh for every attempt, from what it declares it needs; keep nothing in what builds it that changes between attempts or journeys.
3. Build the workflow once, before any journey, and handle its violations there; never change it while journeys run.
4. Use the long form only where the short form can't say what you need.
