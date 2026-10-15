# Cheat sheet

Everything you declare with Itinera, in the order the book teaches it, with the chapter to read for more. After the table comes how each line is written in Rust.

## What you declare

| What you declare | What it does | Chapter |
|---|---|---|
| A step | holds one business rule, runs once per attempt and reports an outcome | [4](04-a-first-step.md) |
| A step's inputs, required or optional | the data a step needs, which the workflow supplies | [4](04-a-first-step.md), [20](20-an-optional-input.md) |
| An outcome | success, failure with a reason, or skipped | [4](04-a-first-step.md), [16](16-skipping-a-step.md) |
| A workflow and its steps | the plan: the steps in order, each under its name | [5](05-a-first-journey.md), [6](06-passing-data-on.md) |
| A workflow instance and its initial data | one journey's starting point | [5](05-a-first-journey.md), [24](24-running-many-journeys.md) |
| An executor, and running an instance | runs one journey at a time and returns its result, or refuses it, keeping nothing between journeys | [5](05-a-first-journey.md), [24](24-running-many-journeys.md) |
| Reading the result and its output | how the journey ended and what explains it, and its output, read typed, unless it was aborted | [5](05-a-first-journey.md), [6](06-passing-data-on.md), [25](25-reading-the-result.md) |
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
| Finishing a journey early | `on step success` returning the lifecycle `FinishWorkflow`, which ends the journey as succeeded | [22](22-finishing-a-journey-early.md) |
| Undoing a step's work when it is given up | `on step failure`, called once the executor gives a step up by its own rule, and the hook that gives it up earlier, both calling the policy's own code | [23](23-undoing-a-charge-when-the-shipping-fails.md) |
| The asynchronous execution mode | steps, hooks and services that wait without holding the thread, so journeys run at the same time | [26](26-running-async-journeys-while-others-wait.md) |
| A step tested alone | a step built from its inputs, and run with fakes of its services and recorders lending its contributor and reporter | [27](27-testing-a-step-alone.md) |
| A hook or an input adapter tested alone | called as a method, with the data it asks for and fakes of its roles | [28](28-testing-a-hook-or-an-adapter-alone.md) |
| A scripted run | in a test, a journey in which a script says how the steps' attempts end, in place of running them | [29](29-testing-the-flow-with-scripted-steps.md) |
| A fake provider and a journey's path | in a test, a function in place of a workflow's provider, and the journey's outcomes and decisions as lines a test compares | [30](30-testing-a-whole-journey.md) |
| A reporter that fails | a reporter whose report fails aborts the journey with `reporter failed`, and receives nothing more | [31](31-stopping-a-journey-when-its-log-fails.md) |
| A reporter that prints every event | a reporter that comes with Itinera, prints each event as it is, and never fails | [32](32-seeing-every-event-of-a-journey.md) |
| Global metadata | a key and a value every reporter of a journey receives, set by the instance before the first event or by a step while it runs | [33](33-giving-every-reporter-what-it-needs.md) |
| What only a hook can know | the journey's ID, the attempt a step hook is about, and the error the step ended in | [34](34-saying-which-attempt-took-the-money.md) |
| A key of its own, and a provider for some steps | an input or a service found under a key other than its field's name; a provider attached to some steps only, handing over a service already boxed, with data saying which it is | [35](35-giving-the-rescue-step-a-service-of-its-own.md) |
| An order stopped before its journey | an instance not created when its journey ID can't be produced, and a journey refused when a reporter can't take its global metadata | [36](36-orders-refused-before-they-start.md) |
| The listing, and the violations of admission | how a workflow is put together, read without running anything, and the mistakes caught before any journey | [37](37-mistakes-caught-before-a-journey.md) |
| A workflow declared by hand | the long form of everything above, and the violations `build` returns | [38](38-declaring-by-hand.md) |

## In Rust

With the `macros` feature, which is on by default, each line of the table is written as follows. Types and attributes are imported through `itinera`, as in `use itinera::step::{Outcome, Step, step};`; the API documentation of `itinera` has every item.

| What you declare | How you write it | Chapter |
|---|---|---|
| A step | `#[derive(Step)]` on its struct, and `#[step]` on the block holding `fn run(self, ...) -> Result<Outcome>`, with `Result` from `itinera::error` | 4 |
| A step's inputs | a field marked `#[input]`; an `Option<T>` field is optional | 4, 20 |
| An outcome | `Self::succeed()`, `Self::fail("code")`, `Self::skip()` or `Self::skip_because("code")` | 4, 16 |
| A workflow and its steps | `#[workflow(name = "orders")]` on its struct, then one `#[with_step(Charge, name = "charge")]` line per step, in order | 5, 6 |
| A workflow instance and its initial data | `#[derive(InitialData)]` on a struct, then `Orders::builder().initial_data(order).create()?` | 5, 24 |
| An executor, and running an instance | `let mut executor = local_executor_for!(Orders);`, then `executor.run(instance)` for each instance, which returns `Result<JourneyResult, Refusal>` | 5, 24 |
| Reading the result and its output | a `match` on `result.status`: `JourneyStatus::Succeeded`, `Failed { failure, .. }` with `failure.cause()`, or `Aborted(abort)` with `abort.reason()`; and `#[derive(Output)]` on a struct read with `result.output::<Order>()?` | 5, 6, 25 |
| A contributor | a parameter of `run` of type `&mut Contributor`, then `contributor.contribute("key", value)` | 6 |
| A service and its provider | a parameter of `run` marked `#[service]`, of type `&dyn Trait`; a method of the workflow's `#[workflow] impl` block marked `#[provider]`, or `#[provider(key = "...", per_attempt)]` for a new instance for each attempt, returning `Result<Provided<dyn Trait>>`, made with `provide!(...)`; a per-attempt provider may request `attempt: Attempt` | 7, 13 |
| A reporter | `impl Reporter for Log`, listed with `reporters = [Log, Desk]` on `#[workflow(...)]` | 8, 10 |
| A step's own events | a parameter of `run` of type `StepReporter`, then `reporter.info("...")?`, `.warning(...)` or `.error(...)` | 9 |
| A journey ID generator | a method of the same block marked `#[id_generator]`, returning `Result<String>` | 11 |
| A retry budget | `retry_budget = 1` on the step's `#[with_step]` line | 12 |
| An abnormal termination | `run` returns `Err(error)`, often through `?`; `no_retry_on_abnormal_termination` on the step's `#[with_step]` line keeps it from spending the retry budget | 12 |
| A rescue step | `#[derive(RescueStep)]` on its struct, and `#[rescue_step]` on the block holding `fn run(self, ...) -> Result<RescueOutcome>`, whose struct may hold a field `reason: Reason`; attached with `rescue = RenewCard` on the step's `#[with_step]` line | 14 |
| A step policy and its hooks | `#[step_policy(name = "audit")]` on its block, and a method marked with its hook, such as `#[on_step_success]`; attached with `policies = [Audit]` on a `#[with_step]` line | 15 |
| Giving a step up early | a method marked `#[on_step_retry]` or `#[on_step_abnormal_termination]`, returning `Result<OnFailure>`, with `Self::fail_workflow("code")` or `Self::keep_default()` | 17 |
| What a hook requests | parameters marked `#[from_step]`, `#[from_workflow]` or `#[role]`, and parameters of type `StepName`, `Option<Reason>` or `&mut Contributor` | 15, 17, 18, 19 |
| A workflow policy and its hooks | `#[workflow_policy(name = "notify")]` on its block, and a method marked `#[on_workflow_success]` or `#[on_workflow_failure]`; attached with `policies = [Notify]` on `#[workflow(...)]` | 18 |
| A role | a trait, and `#[role]` on the workflow's implementation of it; a hook requests it as `#[role] messages: &dyn Messages` | 19 |
| An input adapter | a method of the workflow's `#[workflow] impl` block, marked `#[input_adapter(steps = ["ship"])]`, returning `Result<Supplied>`, made with `Self::supply(value)` or `Self::from_data_bag()` | 21 |
| A value of your own type | `#[derive(Clone, Serialize, Deserialize)]` on its struct, with the `serde` crate and its `derive` feature | 21 |
| Finishing a journey early | `Self::finish_workflow()`, returned by a method marked `#[on_step_success]` | 22 |
| Undoing a step's work when it is given up | a method marked `#[on_step_failure]`, returning `Result<OnFailure>`, which may ask for `cause: StepFailureCause`; and a method of the policy's own, without a mark, that its hooks call | 23 |
| The asynchronous execution mode | the `async` feature; `#[async_trait]`, from `itinera::mode`, on the trait of each service that waits, and each implementation; `async fn run` and `async fn` hooks; a role's trait that is `Sync`; `local_executor_for!(Orders).run(instance).await`, with an executor for each journey running at the same time | 26 |
| A step tested alone | with the `testing` feature under `[dev-dependencies]`, and only inside `#[cfg(test)] mod tests`, a `#[test]` function calling `Charge { ... }.run(&fake, contributions.contributor())`, with `Contributions` from `itinera::journey` and `StepEventReporter` from `itinera::step`; for an asynchronous step, an `async fn` marked `#[tokio::test]` that awaits `run`, with `events.async_reporter()` | 27 |
| A hook or an input adapter tested alone | `FailedShipping.before_retrying(Some(Reason::from("code")), receipt, amount, &refunds, contributions.contributor())?`, read with its `keeps_default()` or `fails_workflow()`, or `Orders { ... }.parcels("weight", parcel)?` | 28 |
| A scripted run | with the `testing` feature under `[dev-dependencies]`, and only inside `#[cfg(test)] mod tests`, `instance.scripted().step("charge").will_fail("code").on_attempt(2).will_succeed().and_step("charge_rescue").will_succeed().run()?`, which returns a `ScriptedRun` | 29 |
| A fake provider and a journey's path | with the `testing` feature under `[dev-dependencies]`, and only inside `#[cfg(test)] mod tests`, `.fake_provider::<dyn Payments>("bank", fake)` on the builder, where `fake` returns `Ok(provide!(...))`, and `.reporter(recorder.clone())` with a `RecordingReporter`, then `recorder.path()` | 30 |
| A reporter that fails | `report` returning `Err`, which aborts the journey; the result's `Abort::ReporterFailed(error)` holds the error | 31 |
| A reporter that prints every event | `DebugReporter`, from `itinera::report`, in `reporters = [DebugReporter, ...]` | 32 |
| Global metadata | `.reporters_global_metadata("customer", value)` on the instance builder, or `reporter.set_global_metadata("receipt", value)?` in a step with `mut reporter: StepReporter`, received by `Reporter::set_global_metadata(&mut self, key, value)` | 33 |
| What only a hook can know | parameters `journey: &JourneyId`, `attempt: Attempt` and `error: &Error` on `on step abnormal termination`, or `Option<&Error>` on `on step retry` and `on step failure` | 34 |
| A key of its own, and a provider for some steps | `#[input("card")] expired: String` or `#[service("cards")] registry: &dyn Cards`; `#[provider(steps = ["charge_rescue"])]`, returning `Provided::from_box(boxed).with_data(value)` | 35 |
| An order stopped before its journey | `create()` returning `Err` when the ID generator fails; `run` returning `Err(refusal)` when a reporter's `set_global_metadata` returns `Err` | 36 |
| The listing, and the violations of admission | `Orders::descriptor().listing()`, with `Workflow` from `itinera::workflow` in scope, giving a `ListedStep` for each step; with the attributes, a violation is a compile error | 37 |
| A workflow declared by hand | `WorkflowDescriptor::builder("orders").step(StepDescriptor::new(step_name!("price"), Pricing)).build()`, which returns `Result<WorkflowDescriptor<Orders>, Violations>`; a `StepFactory` and a `Step` for a step, a hook's trait such as `OnStepSuccess<Orders>` for a policy | 38 |

For the asynchronous form, as chapter 26 shows, turn on the `async` feature, mark the trait of each service that waits, and its implementations, `#[async_trait]`, write `async fn run`, which makes the workflow asynchronous and whose step reporter is `AsyncStepReporter<'_>`, with its lifetime written, and `async fn` hooks, keep a synchronous reporter under `reporters = [...]`, or declare an `AsyncReporter` under `async_reporters = [...]`, as chapter 31 shows, and await the `run` of the executor `local_executor_for!` makes, an `AsyncLocalExecutor`.
