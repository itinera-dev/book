# Cheat sheet

Everything you declare with Itinera, in the order the book teaches it, with the chapter to read for more. After the table comes how each line is written in TypeScript.

## What you declare

| What you declare | What it does | Chapter |
|---|---|---|
| A step | holds one business rule, runs once per attempt and reports an outcome | [4](04-a-first-step.md) |
| A step's inputs, required or optional | the data a step needs, which the workflow supplies, each checked by its type token | [4](04-a-first-step.md), [20](20-an-optional-input.md) |
| An outcome | success, failure with a reason, or skipped | [4](04-a-first-step.md), [16](16-skipping-a-step.md) |
| A workflow and its steps | the plan: the steps in order, each under its name | [5](05-a-first-journey.md), [6](06-passing-data-on.md) |
| A workflow instance and its initial data | one journey's starting point | [5](05-a-first-journey.md), [24](24-running-many-journeys.md) |
| An executor, and running an instance | runs one journey at a time and resolves to its result, or refuses it, keeping nothing between journeys | [5](05-a-first-journey.md), [24](24-running-many-journeys.md) |
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
| A value of your own type | a plain object of yours, held in the data bag like any other value, and read back with a type token made for it | [21](21-deciding-where-a-steps-data-comes-from.md) |
| Finishing a journey early | `on step success` returning the lifecycle `FinishWorkflow`, which ends the journey as succeeded | [22](22-finishing-a-journey-early.md) |
| Undoing a step's work when it is given up | `on step failure`, called once the executor gives a step up by its own rule, and the hook that gives it up earlier, both calling the policy's own code | [23](23-undoing-a-charge-when-the-shipping-fails.md) |
| Steps and hooks that wait | a step or a hook that awaits a service or a role, so journeys run at the same time, each with an executor of its own | [26](26-running-async-journeys-while-others-wait.md) |
| A step tested alone | a step built from its inputs, and run with fakes of its services and recorders lending its contributor and reporter | [27](27-testing-a-step-alone.md) |
| A hook or an input adapter tested alone | called as a method, with the data it asks for and fakes of its roles | [28](28-testing-a-hook-or-an-adapter-alone.md) |
| A scripted run | in a test, a journey in which a script says how the steps' attempts end, in place of running them | [29](29-testing-the-flow-with-scripted-steps.md) |
| A fake provider and a journey's path | in a test, a function in place of a workflow's provider, and the journey's outcomes and decisions as lines a test compares | [30](30-testing-a-whole-journey.md) |
| A reporter that fails | a reporter whose report fails aborts the journey with `reporter failed`, and receives nothing more | [31](31-stopping-a-journey-when-its-log-fails.md) |
| A reporter that prints every event | a reporter that comes with Itinera, prints each event as it is, and never fails | [32](32-seeing-every-event-of-a-journey.md) |
| Global metadata | a key and a value every reporter of a journey receives, set by the instance before the first event or by a step while it runs | [33](33-giving-every-reporter-what-it-needs.md) |
| What only a hook can know | the journey's ID, the attempt a step hook is about, and the error the step ended in | [34](34-saying-which-attempt-took-the-money.md) |
| A key of its own, and a provider for some steps | an input or a service found under a key other than its field's or parameter's name; a provider attached to some steps only, handing over a client it didn't make, with data saying which it is | [35](35-giving-the-rescue-step-a-service-of-its-own.md) |
| An order stopped before its journey | an instance not created when its journey ID can't be produced, and a journey refused when a reporter can't take its global metadata | [36](36-orders-refused-before-they-start.md) |
| The listing, and the violations of admission | how a workflow is put together, read without running anything, and the mistakes caught before any journey | [37](37-mistakes-caught-before-a-journey.md) |
| A workflow declared by hand | the long form of everything above, for a compiler that records no decorator metadata, and the violations `build()` throws | [38](38-declaring-by-hand.md) |

## In TypeScript

With the decorators, each line of the table is written as follows. Each name is imported from the subpath of `itinera` its concept has, as in `import { input, type Outcome, Step, step } from "itinera/step";`, and every item's documentation, which your editor shows, has the rest.

| What you declare | How you write it | Chapter |
|---|---|---|
| A step | a class extending `Step`, marked `@step()`, whose `run` returns an `Outcome`, or a promise of one | 4, 9 |
| A step's inputs | a constructor parameter marked `@input("price", integer)`, with a type token from `itinera/value`, such as `integer` or `string`; with `optional(integer)`, the input is optional, and the parameter an `Option<number>` from `itinera/option` | 4, 20 |
| An outcome | `this.succeed()`, `this.fail("code")`, `this.skip()` or `this.skipBecause("code")` | 4, 16 |
| A workflow and its steps | a class extending `Workflow`, marked `@workflow({ name: "orders" })` on the top line, then one `@withStep(Charge, { name: "charge" })` line per step, in order | 5, 6 |
| A workflow instance and its initial data | `const shop = Orders.build()`, once, then `shop.instance(order)` for each order, with the order as a plain object | 5, 24 |
| An executor, and running an instance | `const executor = new LocalExecutor()`, then `await executor.run(instance)` for each instance, which resolves to a `JourneyResult`, or rejects with a `Refusal`, from `itinera/executor` | 5, 24 |
| Reading the result and its output | `result.isSucceeded()`, `result.isFailed()` or `result.isAborted()`, then `result.status.failure`, with its `cause` and `reason`, or `result.status.abort`, with its `reason`; and `result.output({ receipt: string, tracking: optional(string) })` | 5, 6, 25 |
| A contributor | a parameter of `run` marked `@contributor()`, of type `Contributor`, then `contributor.contribute("key", value)` | 6 |
| A service and its provider | an interface with a token, `serviceType<Payments>("Payments")`; a parameter of `run` marked `@service("payments", PAYMENTS)`; a method of the workflow's class marked `@provider(PAYMENTS, { key: "payments" })`, or with `perAttempt: true` for a new instance for each attempt, returning `this.provide(new Bank())`, typed `Provided<Payments>`; a provider for each attempt may ask for `@attempt() attempt: Attempt` | 7, 13 |
| A reporter | a class implementing `Reporter`, from `itinera/report`, whose `report(event)` reads `event.kind`, listed with `reporters: [Log, Desk]` in `@workflow(...)` | 8, 10 |
| A step's own events | a parameter of `run` marked `@stepReporter()`, of type `StepReporter`, then `await reporter.info("...")`, `.warning(...)` or `.error(...)` | 9 |
| A journey ID generator | a method of the workflow's class marked `@idGenerator()`, asking for `@dataBag() data: DataBag` and returning a `string` | 11 |
| A retry budget | `retryBudget: 1` in the step's `@withStep` options | 12 |
| An abnormal termination | `run` throws, or its promise rejects; `noRetryOnAbnormalTermination: true` in the step's `@withStep` options keeps it from spending the retry budget | 12 |
| A rescue step | a class extending `RescueStep`, marked `@rescueStep()`, whose `run` returns a `RescueOutcome` and whose constructor may ask for `@failureReason() reason: Reason`; attached with `rescue: RenewCard` in the step's `@withStep` options | 14 |
| A step policy and its hooks | a class extending `StepPolicy`, marked `@stepPolicy({ name: "audit" })`, with a method marked with its hook, such as `@onStepSuccess()`, returning an `OnSuccess` made with `this.keepDefault()`; attached with `policies: [Audit]` in a step's `@withStep` options | 15 |
| Giving a step up early | a method marked `@onStepRetry()` or `@onStepAbnormalTermination()`, returning an `OnFailure`, with `this.failWorkflow("code")` or `this.keepDefault()` | 17 |
| What a hook requests | parameters marked `@fromStep("receipt", string)`, `@fromWorkflow("price", integer)`, `@stepName()`, `@failureReason({ optional: true })`, `@role<Messages>()` or `@contributor()` | 15, 17, 18, 19 |
| A workflow policy and its hooks | a class extending `WorkflowPolicy`, marked `@workflowPolicy({ name: "notify" })`, with methods marked `@onWorkflowSuccess()` or `@onWorkflowFailure()`, returning `void`; attached with `policies: [Notify]` in `@workflow(...)` | 18 |
| A role | an interface, which the workflow's class implements, named by the policy as `WorkflowPolicy<Messages>`; a hook asks for it as `@role<Messages>() messages: Messages` | 19 |
| An input adapter | a method of the workflow's class marked `@inputAdapter({ steps: ["ship"] })`, asking for `@key() key: string`, and returning `this.supply(value)` or `this.fromDataBag()` | 21 |
| A value of your own type | an interface for a plain object, and a type token for it made with `objectOf({ weight: integer, size: listOf(integer) })` | 21 |
| Finishing a journey early | `this.finishWorkflow()`, returned by a method marked `@onStepSuccess()` | 22 |
| Undoing a step's work when it is given up | a method marked `@onStepFailure()`, returning an `OnFailure`, which may ask for `@failureCause() cause: StepFailureCause`; and a private method of the policy, without a mark, that its hooks call | 23 |
| Steps and hooks that wait | `async run` and `async` hooks, which await services and roles returning promises; one `new LocalExecutor()` for each journey running at the same time, and `await Promise.all(...)` | 26 |
| A step tested alone | in a test file, `await new Charge(...).run(fake, contributions.contributor)`, with `Contributions` and `StepEventReporter` from `itinera/testing`, the outcome compared with `assert.deepEqual`, and what it contributed read with `contributions.read("receipt", string).option()` | 27 |
| A hook or an input adapter tested alone | in a test file, `await new FailedShipping().beforeRetrying(...)`, compared with `{ kind: "KeepDefault" }`, or `Orders.build("email").parcels("weight", parcel)`, compared with `{ kind: "supplied", value: 60 }` | 28 |
| A scripted run | in a test file, `await scripted(instance).step("charge").willFail("code").onAttempt(2).willSucceed().andStep("charge_rescue").willSucceed().run()`, with `scripted` from `itinera/testing`, which resolves to a `ScriptedRun` | 29 |
| A fake provider and a journey's path | in a test file, `fakeProvider(instance, "bank", PAYMENTS, (provide) => provide(bank))`, and an instance made with `{ reporters: [recorder] }` for a `RecordingReporter`, both from `itinera/testing`, then `recorder.path` | 30 |
| A reporter that fails | `report` throwing, or returning a promise that rejects, which aborts the journey; the result's `abort.errorMessage` holds the error's message, and `abort.error` the error | 31 |
| A reporter that prints every event | `DebugReporter`, from `itinera/report`, in `reporters: [DebugReporter, ...]` | 32 |
| Global metadata | `{ reportersGlobalMetadata: { customer } }` as the second argument of `instance(...)`, or `await reporter.setGlobalMetadata("receipt", value)` in a step, received by a reporter's `receiveGlobalMetadata(key, value)` | 33 |
| What only a hook can know | parameters `@journeyId() journey: JourneyId`, `@attempt() attempt: Attempt`, and `@error() thrown: unknown` on `on step abnormal termination`, or `@error({ optional: true })` on `on step retry` and `on step failure` | 34 |
| A key of its own, and a provider for some steps | `@input("card", string) private readonly expired: string` or `@service("cards", CARDS) registry: Cards`; `@provider(CARDS, { key: "cards", steps: ["charge_rescue"] })`, returning `this.provide(client, "eu")` | 35 |
| An order stopped before its journey | `instance(...)` throwing what the ID generator threw; `run` rejecting with a `Refusal` whose `refused.kind` is `global metadata not set` when a reporter's `receiveGlobalMetadata` fails | 36 |
| The listing, and the violations of admission | `Orders.listing()`, giving a `ListedStep` for each step; with the decorators, a `Violations`, from `itinera/workflow`, thrown by `@workflow` as the class is defined; some mistakes don't compile | 37 |
| A workflow declared by hand | `Orders.builder().name("orders").step(Charge.builder("charge").retryBudget(1)).provider("bank", PAYMENTS, { key: "payments" }).constructWith("email").build()`, with each step's requests given by `needs(...)` and `run(...)`, which throws a `Violations` | 38 |
