# Cheat sheet

Everything you declare with Itinera, in the order the book teaches it, with the chapter to read for more. After the table comes how each line is written in Rust.

## What you declare

| What you declare | What it does | Chapter |
|---|---|---|
| A step | holds one business rule, runs once per attempt and reports an outcome | [4](04-a-first-step.md) |
| A step's inputs, required or optional | the data a step needs, which the workflow supplies | [4](04-a-first-step.md), [19](19-an-optional-input.md) |
| An outcome | success, failure with a reason, a failure that might work next time, or skipped | [4](04-a-first-step.md), [12](12-trying-again.md), [16](16-skipping-a-step.md) |
| A workflow and its steps | the plan: the steps in order, each under its name | [5](05-a-first-journey.md), [6](06-passing-data-on.md) |
| A workflow instance and its initial data | one journey's starting point | [5](05-a-first-journey.md), [21](21-running-many-journeys.md) |
| An executor, and running an instance | runs one journey and returns its result, or refuses it | [5](05-a-first-journey.md), [21](21-running-many-journeys.md) |
| Reading the result and its output | how the journey ended, and the data bag it left | [5](05-a-first-journey.md), [6](06-passing-data-on.md), [22](22-reading-the-result.md) |
| A contributor | how a step adds data to the data bag | [6](06-passing-data-on.md) |
| A service and its provider | a tool a step asks for by a key, and the part of the workflow that supplies it, per journey or per attempt | [7](07-giving-a-step-the-tools-it-needs.md), [13](13-a-carrier-for-each-attempt.md) |
| A reporter | receives every event of a journey | [8](08-seeing-what-happened.md), [10](10-more-than-one-reporter.md) |
| A step's own events | what only the step knows, emitted as `step_info`, `step_warning` or `step_error` | [9](09-saying-what-only-a-step-knows.md) |
| A journey ID generator | names each journey, for example after the order | [11](11-naming-a-journey.md) |
| A retry budget | how many times a step may be tried again | [12](12-trying-again.md) |
| A rescue step | a step that repairs the cause of a failure before the step runs once more | [14](14-repairing-then-trying-once-more.md) |
| A step policy and its hooks | reacts to a step's outcome, such as `on step success` | [15](15-reacting-to-what-a-step-did.md) |
| What a hook requests | data from the step or the workflow, the step's name, a role, a contributor | [15](15-reacting-to-what-a-step-did.md), [17](17-doing-something-when-the-journey-ends.md), [18](18-letting-policies-act-on-the-outside-world.md) |
| A workflow policy and its hooks | reacts to how the journey ended, such as `on workflow success` | [17](17-doing-something-when-the-journey-ends.md) |
| A role | operations the workflow provides to policies | [18](18-letting-policies-act-on-the-outside-world.md) |
| An input adapter | decides where a step's inputs come from | [20](20-deciding-where-a-steps-data-comes-from.md) |
| A policy that may fail while it is built | a policy built by a function of your own, which may fail; a workflow policy that fails is a refusal | [21](21-running-many-journeys.md) |
| A workflow declared by hand | the long form of everything above | [28](28-declaring-by-hand.md) |
| An abnormal termination | an error that escapes a running step | [When something goes wrong inside a step](11-when-something-goes-wrong-inside-a-step.md) |
| A lifecycle | what a hook returns to end the journey early: `FailWorkflow` or `FinishWorkflow` | [Ending a journey early](16-ending-a-journey-early.md) |

## In Rust

With the `macros` feature, which is on by default, each line of the table is written as follows. Types and attributes are imported through `itinera`, as in `use itinera::step::{Outcome, Step, step};`; the API documentation of `itinera` has every item.

| What you declare | How you write it | Chapter |
|---|---|---|
| A step | `#[derive(Step)]` on its struct, and `#[step]` on the block holding `fn run(self, ...) -> Result<Outcome, Error>` | 4 |
| A step's inputs | a field marked `#[input]`; an `Option<T>` field is optional | 4, 19 |
| An outcome | `Self::succeed()`, `Self::fail("code")`, `Self::fail_retriably("code")`, `Self::skip()` or `Self::skip_because("code")` | 4, 12, 16 |
| A workflow and its steps | `#[workflow(name = "orders")]` on its struct, then one `#[with_step(Charge, name = "charge")]` line per step, in order | 5, 6 |
| A workflow instance and its initial data | `#[derive(InitialData)]` on a struct, then `Orders::builder().initial_data(order).create()?` | 5, 21 |
| An executor, and running an instance | `LocalExecutor::new().run(instance)`, which returns `Result<JourneyResult, Refusal>` | 5, 21 |
| Reading the result and its output | `result.status`, and `#[derive(Output)]` on a struct read with `result.output::<Receipt>()?` | 5, 6, 22 |
| A contributor | a parameter of `run` marked `#[contributor]`, of type `Contributor<'_>`, then `contributor.contribute("key", value)` | 6 |
| A service and its provider | a parameter of `run` marked `#[service]`, of type `&dyn Trait`; a method of the workflow's `#[workflow] impl` block marked `#[provider]`, or `#[provider(key = "...", per_attempt)]` for a new instance for each attempt, returning `Result<Provided<dyn Trait>, Error>`, made with `provide!(...)`; a per-attempt provider may request `#[attempt]` | 7, 13 |
| A reporter | `impl Reporter for Log`, listed with `reporters = [Log, Desk]` on `#[workflow(...)]` | 8, 10 |
| A step's own events | a parameter of `run` marked `#[reporter]`, of type `StepReporter<'_>`, then `reporter.info("...")?`, `.warning(...)` or `.error(...)` | 9 |
| A journey ID generator | a method of the same block marked `#[id_generator]`, returning `Result<String, Error>` | 11 |
| A retry budget | `retry_budget = 1` on the step's `#[with_step]` line | 12 |
| A rescue step | `#[derive(RescueStep)]` on its struct, and `#[rescue_step]` on the block holding `fn run(self, ...) -> Result<RescueOutcome, Error>`, which may request `#[reason]`; attached with `rescue = RenewCard` on the step's `#[with_step]` line | 14 |
| A step policy and its hooks | `#[step_policy(name = "audit")]` on its block, and a method marked with its hook, such as `#[on_step_success]`; attached with `policies = [Audit]` on a `#[with_step]` line | 15 |
| What a hook requests | parameters marked `#[from_step]`, `#[from_workflow]`, `#[step_name]`, `#[role]` or `#[contributor]` | 15, 17, 18 |
| A workflow policy and its hooks | `#[workflow_policy(name = "notify")]` on its block, and a method marked with its hook, such as `#[on_workflow_success]`; attached with `policies = [Notify]` on `#[workflow(...)]` | 17 |
| A role | a trait, and `#[role]` on the workflow's implementation of it; a hook requests it as `#[role] messages: &dyn Messages` | 18 |
| An input adapter | a method of the workflow's `#[workflow] impl` block, marked `#[input_adapter(steps = ["charge"])]`, returning `Result<Option<AnyValue>, Error>`, made with `Self::supply(value)` or `Self::from_data_bag()` | 20 |
| A policy that may fail while it is built | a function of its block marked `#[factory]`, returning `Result<Self, Error>` | 21 |
| A workflow declared by hand | `WorkflowDescriptor::builder("orders").step(StepDescriptor::new(step_name!("charge"), Charging)).build()?` | 28 |
| An abnormal termination | `run` returns `Err(error)`, often through `?`; `abnormal_termination_retriable` on the step's `#[with_step]` line lets it spend the retry budget | When something goes wrong inside a step |
| A lifecycle | `Self::carry_on()`, `Self::fail_workflow("code")`, or, from `on step success` only, `Self::finish_workflow()` | Ending a journey early |

For the asynchronous form, turn on the `async` feature, write `async fn run`, which makes the workflow asynchronous, and `async fn` hooks, implement `AsyncReporter` for an asynchronous reporter and list it with `async_reporters = [...]`, or keep a synchronous one under `reporters = [...]`, and run on `AsyncLocalExecutor`, awaiting its `run`.
