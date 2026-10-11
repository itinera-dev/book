With the `macros` feature, each line of the table is written as follows. Paths are those applications write, through `itinera`.

| What you declare | How you write it | Chapter |
|---|---|---|
| A step | `#[derive(Step)]` on its struct, and `#[step]` on the block holding `fn run(self, ...) -> Result<Outcome, Error>` | 6 |
| A step's inputs | a field marked `#[input]`, or `#[input("key")]`; an `Option<T>` field is optional, and `#[input(required)]` makes its key required; `#[default]` marks a field that is not an input | 8 |
| A contributor | a parameter of `run` marked `#[contributor]`, of type `Contributor<'_>`, then `contributor.contribute("key", value)` | 9 |
| An outcome | `Self::succeed()`, `Self::fail("code")`, `Self::fail_retriably("code")`, `Self::skip()` or `Self::skip_because("code")` | 10 |
| An abnormal termination | `run` returns `Err(error)`, often through `?` | 11 |
| A service and its provider (planned) | a parameter of `run` marked `#[service]`, of type `&dyn Trait`; a method of the workflow marked `#[provider]` or `#[provider(per_attempt)]`, optionally with `steps = ["..."]` | 12 |
| A workflow and its steps | `#[workflow(name = "orders")]` on its struct, then one `#[with_step(Charge, name = "charge")]` line per step, in order | 6, 7 |
| A retry budget | `retry_budget = 1` on the step's `#[with_step]` line, and `abnormal_termination_retriable` to retry an abnormal termination too | 13 |
| A rescue step (planned) | `#[derive(RescueStep)]` on its struct, and `#[rescue_step]` on the block holding `fn run(self, ...) -> Result<RescueOutcome, Error>`; attached with `rescue = RenewCard` on the step's `#[with_step]` line | 14 |
| A step policy and its hooks | `#[step_policy(name = "audit")]` on its block, and a method marked `#[on_step_success]`, `#[on_step_failure]`, `#[on_step_retry]` or `#[on_step_abnormal_termination]`; attached with `policies = [Audit]` on a `#[with_step]` line | 15 |
| What a hook requests | parameters marked `#[from_step]`, `#[from_workflow]`, `#[reason]`, `#[cause]`, `#[error]`, `#[step_name]`, `#[attempt]`, `#[journey_id]`, `#[role]`, `#[contributor]` or `#[reporter]` | 15 |
| A lifecycle | `Self::carry_on()`, `Self::fail_workflow("code")`, or, from `on step success` only, `Self::finish_workflow()` | 16 |
| A workflow policy and its hooks | `#[workflow_policy(name = "notify")]` on its block, and a method marked `#[on_workflow_success]` or `#[on_workflow_failure]`; attached with `policies = [Notify]` on `#[workflow(...)]` | 17 |
| A role | a trait, and `#[role]` on the workflow's implementation of it; a hook requests it as `#[role] messages: &dyn Messages` | 18 |
| An input adapter | a method of the workflow's `#[workflow] impl` block, marked `#[input_adapter(steps = ["charge"])]`, returning `Result<Option<AnyValue>, Error>` | 19 |
| A journey ID generator | a method of the same block marked `#[id_generator]`, returning `Result<String, Error>` | 20 |
| A workflow instance and its initial data | `#[derive(InitialData)]` on a struct, then `Orders::builder().initial_data(order).create()?` | 21 |
| An executor, and running an instance | `LocalExecutor::new().run(instance)`, which returns `Result<JourneyResult, Refusal>` | 21 |
| Reading the result and its output | `result.status`, and `#[derive(Output)]` on a struct read with `result.output::<Receipt>()?` | 22 |
| A reporter | `impl Reporter for Printer`, listed with `reporters = [Printer]` on `#[workflow(...)]`, or added with `.reporter(...)` on the instance's builder | 23 |
| A step's own events | a parameter of `run` marked `#[reporter]`, of type `StepReporter<'_>`, then `reporter.info("...")?`, `.warning(...)` or `.error(...)`; a hook's `#[reporter]` emits the `journey_*` events the same way | 23 |
| A policy that may fail while it is built | a function of its block marked `#[factory]`, returning `Result<Self, Error>` | 27 |
| A workflow declared by hand | `WorkflowDescriptor::builder("orders").step(StepDescriptor::new(step_name!("charge"), Charging)).build()?` | 28 |

For the asynchronous form, turn on the `async` feature, write `async fn run` and `async fn` hooks, add `asynchronous` to `#[workflow(...)]`, list asynchronous reporters with `async_reporters = [...]`, and run on `AsyncLocalExecutor`.
