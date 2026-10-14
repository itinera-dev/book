# Cheat sheet

Everything you declare with Itinera, in the order the book teaches it, with the chapter to read for more. After the table comes how each line is written in Rust.

## What you declare

| What you declare | What it does | Chapter |
|---|---|---|
| A step | holds one business rule, runs once per attempt and reports an outcome | [4](04-a-first-step.md) |
| A step's inputs, required or optional | the data a step needs, which the workflow supplies | [4](04-a-first-step.md), [20](20-an-optional-input.md) |
| An outcome | success, failure with a reason, or skipped | [4](04-a-first-step.md), [16](16-skipping-a-step.md) |
| A workflow and its steps | the plan: the steps in order, each under its name | [5](05-a-first-journey.md), [6](06-passing-data-on.md) |
| A workflow instance and its initial data | one journey's starting point | [5](05-a-first-journey.md), [22](22-running-many-journeys.md) |
| An executor, and running an instance | runs one journey at a time and returns its result, or refuses it, keeping nothing between journeys | [5](05-a-first-journey.md), [22](22-running-many-journeys.md) |
| Reading the result and its output | how the journey ended and what explains it, and its output, read typed, unless it was aborted | [5](05-a-first-journey.md), [6](06-passing-data-on.md), [23](23-reading-the-result.md) |
| A contributor | how a step adds data to the data bag | [6](06-passing-data-on.md) |
| A service and its provider | a tool a step asks for by a key, and the part of the workflow that supplies it, per journey or per attempt | [7](07-giving-a-step-the-tools-it-needs.md), [13](13-a-carrier-for-each-attempt.md) |
| A reporter | receives every event of a journey | [8](08-seeing-what-happened.md), [10](10-more-than-one-reporter.md) |
| A step's own events | what only the step knows, emitted as `step_info`, `step_warning` or `step_error` | [9](09-saying-what-only-a-step-knows.md) |
| A journey ID generator | names each journey, for example after the order | [11](11-naming-a-journey.md) |
| A retry budget | how many times a step may be tried again | [12](12-trying-again.md) |
| An abnormal termination | an error that escapes a running step, which spends the retry budget unless the step's line says otherwise | [12](12-trying-again.md) |
| A rescue step | a step that repairs the cause of a failure before the step runs once more | [14](14-repairing-then-trying-once-more.md) |
| A step policy and its hooks | reacts to a step's outcome, such as `on step success` | [15](15-reacting-to-what-a-step-did.md) |
| Giving a step up early | `on step retry` or `on step abnormal termination` returning the lifecycle `FailWorkflow` | [17](17-giving-up-a-step-early.md) |
| What a hook requests | data from the step or the workflow, the step's name, the reason a step failed with, a role, a contributor | [15](15-reacting-to-what-a-step-did.md), [17](17-giving-up-a-step-early.md), [18](18-doing-something-when-the-journey-ends.md), [19](19-letting-policies-act-on-the-outside-world.md) |
| A workflow policy and its hooks | reacts to how the journey ended: `on workflow success`, `on workflow failure` | [18](18-doing-something-when-the-journey-ends.md) |
| A role | operations the workflow provides to policies | [19](19-letting-policies-act-on-the-outside-world.md) |
| An input adapter | decides where a step's inputs come from | [21](21-deciding-where-a-steps-data-comes-from.md) |
| A value of your own type | a struct of yours, held in the data bag like any other value | [21](21-deciding-where-a-steps-data-comes-from.md) |
| A step tested alone | a step built from its inputs, and run with fakes of its services and recorders lending its contributor and reporter | [24](24-testing-a-step-alone.md) |
| A hook or an input adapter tested alone | called as a method, with the data it asks for and fakes of its roles | [25](25-testing-a-hook-or-an-adapter-alone.md) |
| A scripted run | a journey in which a script says how the steps' attempts end, in place of running them | [26](26-testing-the-flow-with-scripted-steps.md) |
| A fake provider and a journey's path | a function in place of a workflow's provider, and the journey's outcomes and decisions as lines a test compares | [27](27-testing-a-whole-journey.md) |
| The asynchronous execution mode | steps, hooks and services that wait without holding the thread, so journeys run at the same time | [28](28-running-journeys-while-others-wait.md) |
| A workflow declared by hand | the long form of everything above | [32](32-declaring-by-hand.md) |
| A lifecycle | what a hook returns to end the journey early: `FailWorkflow` or `FinishWorkflow` | [17](17-giving-up-a-step-early.md), [Ending a journey early](16-ending-a-journey-early.md) |

## In Rust

With the `macros` feature, which is on by default, each line of the table is written as follows. Types and attributes are imported through `itinera`, as in `use itinera::step::{Outcome, Step, step};`; the API documentation of `itinera` has every item.

| What you declare | How you write it | Chapter |
|---|---|---|
| A step | `#[derive(Step)]` on its struct, and `#[step]` on the block holding `fn run(self, ...) -> Result<Outcome, Error>` | 4 |
| A step's inputs | a field marked `#[input]`; an `Option<T>` field is optional | 4, 20 |
| An outcome | `Self::succeed()`, `Self::fail("code")`, `Self::skip()` or `Self::skip_because("code")` | 4, 16 |
| A workflow and its steps | `#[workflow(name = "orders")]` on its struct, then one `#[with_step(Charge, name = "charge")]` line per step, in order | 5, 6 |
| A workflow instance and its initial data | `#[derive(InitialData)]` on a struct, then `Orders::builder().initial_data(order).create()?` | 5, 22 |
| An executor, and running an instance | `let mut executor = LocalExecutor::new();`, then `executor.run(instance)` for each instance, which returns `Result<JourneyResult, Refusal>` | 5, 22 |
| Reading the result and its output | a `match` on `result.status`: `JourneyStatus::Succeeded`, `Failed { failure, .. }` with `failure.cause()`, or `Aborted(abort)` with `abort.reason()`; and `#[derive(Output)]` on a struct read with `result.output::<Order>()?` | 5, 6, 23 |
| A contributor | a parameter of `run` marked `#[contributor]`, of type `Contributor<'_>`, then `contributor.contribute("key", value)` | 6 |
| A service and its provider | a parameter of `run` marked `#[service]`, of type `&dyn Trait`; a method of the workflow's `#[workflow] impl` block marked `#[provider]`, or `#[provider(key = "...", per_attempt)]` for a new instance for each attempt, returning `Result<Provided<dyn Trait>, Error>`, made with `provide!(...)`; a per-attempt provider may request `#[attempt]` | 7, 13 |
| A reporter | `impl Reporter for Log`, listed with `reporters = [Log, Desk]` on `#[workflow(...)]` | 8, 10 |
| A step's own events | a parameter of `run` marked `#[reporter]`, of type `StepReporter<'_>`, then `reporter.info("...")?`, `.warning(...)` or `.error(...)` | 9 |
| A journey ID generator | a method of the same block marked `#[id_generator]`, returning `Result<String, Error>` | 11 |
| A retry budget | `retry_budget = 1` on the step's `#[with_step]` line | 12 |
| An abnormal termination | `run` returns `Err(error)`, often through `?`; `no_retry_on_abnormal_termination` on the step's `#[with_step]` line keeps it from spending the retry budget | 12 |
| A rescue step | `#[derive(RescueStep)]` on its struct, and `#[rescue_step]` on the block holding `fn run(self, ...) -> Result<RescueOutcome, Error>`, which may request `#[reason]`; attached with `rescue = RenewCard` on the step's `#[with_step]` line | 14 |
| A step policy and its hooks | `#[step_policy(name = "audit")]` on its block, and a method marked with its hook, such as `#[on_step_success]`; attached with `policies = [Audit]` on a `#[with_step]` line | 15 |
| Giving a step up early | a method marked `#[on_step_retry]` or `#[on_step_abnormal_termination]`, returning `Result<Option<FailWorkflow>, Error>`, with `Self::fail_workflow("code")` or `Self::carry_on()` | 17 |
| What a hook requests | parameters marked `#[from_step]`, `#[from_workflow]`, `#[step_name]`, `#[reason]`, `#[role]` or `#[contributor]` | 15, 17, 18, 19 |
| A workflow policy and its hooks | `#[workflow_policy(name = "notify")]` on its block, and a method marked `#[on_workflow_success]` or `#[on_workflow_failure]`; attached with `policies = [Notify]` on `#[workflow(...)]` | 18 |
| A role | a trait, and `#[role]` on the workflow's implementation of it; a hook requests it as `#[role] messages: &dyn Messages` | 19 |
| An input adapter | a method of the workflow's `#[workflow] impl` block, marked `#[input_adapter(steps = ["ship"])]`, returning `Result<Option<AnyValue>, Error>`, made with `Self::supply(value)` or `Self::from_data_bag()` | 21 |
| A value of your own type | `#[derive(Clone, Serialize, Deserialize)]` on its struct, with the `serde` crate and its `derive` feature | 21 |
| A step tested alone | with the `testing` feature under `[dev-dependencies]`, `Charge { ... }.run(&fake, contributions.contributor())`, with `Contributions` from `itinera::journey` and `StepEventReporter` from `itinera::step` | 24 |
| A hook or an input adapter tested alone | `HeavyParcels.before_retrying(Some(Reason::from("code")))?` or `Orders { ... }.parcels("weight", parcel)?` | 25 |
| A scripted run | `instance.scripted().step("charge").will_fail("code").on_attempt(2).will_succeed().and_step("charge_rescue").will_succeed().run()?`, which returns a `ScriptedRun` | 26 |
| A fake provider and a journey's path | `.fake_provider::<dyn Payments>("bank", fake)` on the builder, where `fake` returns `Ok(provide!(...))`, and `.reporter(recorder.clone())` with a `RecordingReporter`, then `recorder.path()` | 27 |
| The asynchronous execution mode | the `async` feature; `#[async_trait]`, from `itinera::mode`, on a service's trait and each implementation; `async fn run` and `async fn` hooks; a role's trait that is `Sync`; `AsyncLocalExecutor::new().run(instance).await`, with an executor for each journey running at the same time | 28 |
| A workflow declared by hand | `WorkflowDescriptor::builder("orders").step(StepDescriptor::new(step_name!("charge"), Charging)).build()?` | 32 |
| A lifecycle | `Self::carry_on()`, `Self::fail_workflow("code")`, or, from `on step success` only, `Self::finish_workflow()` | 17, Ending a journey early |

For the asynchronous form, as chapter 28 shows, turn on the `async` feature, mark each service's trait and its implementations `#[async_trait]`, write `async fn run`, which makes the workflow asynchronous, and `async fn` hooks, keep a synchronous reporter under `reporters = [...]`, and run on `AsyncLocalExecutor`, awaiting its `run`.
