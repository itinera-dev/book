# Cheat sheet

Everything you declare with Itinera, in the order the book teaches it, with the chapter to read for more. After the table comes how each line is written in Rust.

## What you declare

| What you declare | What it does | Chapter |
|---|---|---|
| A step | holds one business rule, runs once per attempt and reports an outcome | [6](06-a-workflow-of-one-step.md) |
| A workflow and its steps | the plan: the steps in order, each under its name | [6](06-a-workflow-of-one-step.md), [7](07-adding-a-step.md) |
| A step's inputs, required or optional | the data a step needs, which the workflow supplies | [8](08-giving-a-step-what-it-needs.md) |
| A contributor | how a step adds data to the data bag | [9](09-passing-data-on.md) |
| Reading the result and its output | how the journey ended, and the data bag it left | [9](09-passing-data-on.md), [22](22-reading-the-result.md) |
| An outcome | success, failure with a reason, a failure that might work next time, or skipped | [10](10-saying-how-it-went.md) |
| An abnormal termination | an error that escapes a running step | [11](11-when-something-goes-wrong-inside-a-step.md) |
| A service and its provider | a tool a step asks for by a key, and the part of the workflow that supplies it, per journey or per attempt | [12](12-giving-a-step-the-tools-it-needs.md) |
| A retry budget | how many times a step may be tried again | [13](13-trying-again.md) |
| A rescue step | a step that repairs the cause of a failure before the step runs once more | [14](14-repairing-then-trying-once-more.md) |
| A step policy and its hooks | reacts to a step's outcome: `on step success`, `on step failure`, `on step retry`, `on step abnormal termination` | [15](15-reacting-to-what-a-step-did.md) |
| What a hook requests | data from the step or the workflow, the reason, the error, the step's name, the attempt, the journey ID, a role, a contributor, a reporter | [15](15-reacting-to-what-a-step-did.md) |
| A lifecycle | what a hook returns to end the journey early: `FailWorkflow` or `FinishWorkflow` | [16](16-ending-a-journey-early.md) |
| A workflow policy and its hooks | reacts to how the journey ended: `on workflow success`, `on workflow failure` | [17](17-doing-something-when-the-journey-ends.md) |
| A role | operations the workflow provides to policies | [18](18-letting-policies-act-on-the-outside-world.md) |
| An input adapter | decides where a step's inputs come from | [19](19-deciding-where-a-steps-data-comes-from.md) |
| A journey ID generator | names each journey, for example after the order | [20](20-naming-your-journeys.md) |
| A workflow instance and its initial data | one journey's starting point | [21](21-running-many-journeys.md) |
| An executor, and running an instance | runs one journey and returns its result, or refuses it | [21](21-running-many-journeys.md) |
| A policy that may fail while it is built | a policy built by a function of your own, which may fail; a workflow policy that fails is a refusal | [21](21-running-many-journeys.md) |
| A reporter | receives every event of a journey | [23](23-seeing-what-happened.md) |
| A step's own events | what only the step knows, emitted as `step_info`, `step_warning` or `step_error`; a hook emits `journey_info`, `journey_warning` or `journey_error` | [23](23-seeing-what-happened.md) |
| A workflow declared by hand | the long form of everything above | [28](28-declaring-by-hand.md) |

## In Rust

With the `macros` feature, which is on by default, each line of the table is written as follows. Types and attributes are imported through `itinera`, as in `use itinera::step::{Outcome, Step, step};`; the API documentation of `itinera` has every item.

| What you declare | How you write it | Chapter |
|---|---|---|
| A step | `#[derive(Step)]` on its struct, and `#[step]` on the block holding `fn run(self, ...) -> Result<Outcome, Error>` | 6 |
| A workflow and its steps | `#[workflow(name = "orders")]` on its struct, then one `#[with_step(Charge, name = "charge")]` line per step, in order | 6, 7 |
| A step's inputs | a field marked `#[input]`, or `#[input("key")]`; an `Option<T>` field is optional | 8 |
| A contributor | a parameter of `run` marked `#[contributor]`, of type `Contributor<'_>`, then `contributor.contribute("key", value)` | 9 |
| Reading the result and its output | `result.status`, and `#[derive(Output)]` on a struct read with `result.output::<Receipt>()?` | 9, 22 |
| An outcome | `Self::succeed()`, `Self::fail("code")`, `Self::fail_retriably("code")`, `Self::skip()` or `Self::skip_because("code")` | 10 |
| An abnormal termination | `run` returns `Err(error)`, often through `?` | 11 |
| A service and its provider | a parameter of `run` marked `#[service]`, or `#[service("key")]`, of type `&dyn Trait`; a method of the workflow's `#[workflow] impl` block marked `#[provider]` or `#[provider(per_attempt)]`, optionally with any of `steps = ["..."]`, `key = "..."` and `name = "..."`, returning `Result<Box<dyn Trait>, Error>`; a per-attempt provider may request `#[step_name]` and `#[attempt]` | 12, 13 |
| A retry budget | `retry_budget = 1` on the step's `#[with_step]` line, and `abnormal_termination_retriable` to retry an abnormal termination too | 13 |
| A rescue step | `#[derive(RescueStep)]` on its struct, and `#[rescue_step]` on the block holding `fn run(self, ...) -> Result<RescueOutcome, Error>`; attached with `rescue = RenewCard` on the step's `#[with_step]` line; its own adapter and providers are attached to `charge_rescue` | 14 |
| A step policy and its hooks | `#[step_policy(name = "audit")]` on its block, and a method marked `#[on_step_success]`, `#[on_step_failure]`, `#[on_step_retry]` or `#[on_step_abnormal_termination]`; attached with `policies = [Audit]` on a `#[with_step]` line | 15 |
| What a hook requests | parameters marked `#[from_step]`, `#[from_workflow]`, `#[reason]`, `#[error]`, `#[step_name]`, `#[attempt]`, `#[journey_id]`, `#[role]`, `#[contributor]` or `#[reporter]` | 15 |
| A lifecycle | `Self::carry_on()`, `Self::fail_workflow("code")`, or, from `on step success` only, `Self::finish_workflow()` | 16 |
| A workflow policy and its hooks | `#[workflow_policy(name = "notify")]` on its block, and a method marked `#[on_workflow_success]` or `#[on_workflow_failure]`; attached with `policies = [Notify]` on `#[workflow(...)]` | 17 |
| A role | a trait, and `#[role]` on the workflow's implementation of it; a hook requests it as `#[role] messages: &dyn Messages` | 18 |
| An input adapter | a method of the workflow's `#[workflow] impl` block, marked `#[input_adapter(steps = ["charge"])]`, returning `Result<Option<AnyValue>, Error>` | 19 |
| A journey ID generator | a method of the same block marked `#[id_generator]`, returning `Result<String, Error>` | 20 |
| A workflow instance and its initial data | `#[derive(InitialData)]` on a struct, then `Orders::builder().initial_data(order).create()?` | 21 |
| An executor, and running an instance | `LocalExecutor::new().run(instance)`, which returns `Result<JourneyResult, Refusal>` | 21 |
| A policy that may fail while it is built | a function of its block marked `#[factory]`, returning `Result<Self, Error>` | 21 |
| A reporter | `impl Reporter for Log`, listed with `reporters = [Log]`, or the ready-made `DebugReporter` of chapter 4, on `#[workflow(...)]`, or added with `.reporter(...)` on the instance's builder | 23 |
| A step's own events | a parameter of `run` marked `#[reporter]`, of type `StepReporter<'_>`, then `reporter.info("...")?`, `.warning(...)` or `.error(...)`; a hook's `#[reporter]` emits the `journey_*` events the same way | 23 |
| A workflow declared by hand | `WorkflowDescriptor::builder("orders").step(StepDescriptor::new(step_name!("charge"), Charging)).build()?` | 28 |

For the asynchronous form, turn on the `async` feature, write `async fn run` and `async fn` hooks, add `asynchronous` to `#[workflow(...)]`, implement `AsyncReporter` for an asynchronous reporter and list it with `async_reporters = [...]`, or keep a synchronous one under `reporters = [...]`, and run on `AsyncLocalExecutor`, awaiting its `run`.
