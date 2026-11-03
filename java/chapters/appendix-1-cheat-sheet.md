# Cheat sheet

Everything you declare with Itinera, in the order the book teaches it, with the chapter to read for more. After the table comes how each line is written in Java.

## What you declare

| What you declare | What it does | Chapter |
|---|---|---|
| A step | holds one business rule, runs once per attempt and reports an outcome | [4](04-a-first-step.md) |
| A step's inputs, required or optional | the data a step needs, which the workflow supplies, each read as its parameter's type | [4](04-a-first-step.md), [20](20-an-optional-input.md) |
| An outcome | success, failure with a reason, or skipped | [4](04-a-first-step.md), [16](16-skipping-a-step.md) |
| A workflow and its steps | the plan: the steps in order, each under its name | [5](05-a-first-journey.md), [6](06-passing-data-on.md) |
| A workflow instance and its initial data | one journey's starting point | [5](05-a-first-journey.md), [6](06-passing-data-on.md), [24](24-running-many-journeys.md) |
| An executor, and running an instance | runs one journey at a time and returns its result, or refuses it, keeping nothing between journeys | [5](05-a-first-journey.md), [24](24-running-many-journeys.md) |
| Reading the result and its output | how the journey ended and what explains it, and its output, read typed, unless it was aborted | [5](05-a-first-journey.md), [6](06-passing-data-on.md), [25](25-reading-the-result.md) |
| A contributor | how a step adds data to the data bag | [6](06-passing-data-on.md) |
| A service and its provider | a tool a step asks for by a key, and the part of the workflow that supplies it, per journey or per attempt | [7](07-giving-a-step-the-tools-it-needs.md), [13](13-a-carrier-for-each-attempt.md) |
| A reporter | receives every event of a journey | [8](08-seeing-what-happened.md), [10](10-more-than-one-reporter.md) |
| A step's own events | what only the step knows, emitted as `step_info`, `step_warning` or `step_error` | [9](09-saying-what-only-a-step-knows.md) |
| A journey ID generator | names each journey, for example after the order | [11](11-naming-a-journey.md) |
| A retry budget | how many times a step may be tried again | [12](12-trying-again.md) |
| An abnormal termination | an exception that escapes a running step, which spends the retry budget unless the step's line says otherwise | [12](12-trying-again.md) |
| A rescue step | a step that repairs the cause of a failure before the step runs once more | [14](14-repairing-then-trying-once-more.md) |
| A step policy and its hooks | reacts to a step's outcome, such as `on step success` | [15](15-reacting-to-what-a-step-did.md) |
| Giving a step up early | `on step retry` or `on step abnormal termination` returning the lifecycle `FailWorkflow` | [17](17-giving-up-a-step-early.md) |
| What a hook requests | data from the step or the workflow, the step's name, the reason a step failed with, a role, a contributor | [15](15-reacting-to-what-a-step-did.md), [17](17-giving-up-a-step-early.md), [18](18-doing-something-when-the-journey-ends.md), [19](19-letting-policies-act-on-the-outside-world.md) |
| A workflow policy and its hooks | reacts to how the journey ended: `on workflow success`, `on workflow failure` | [18](18-doing-something-when-the-journey-ends.md) |
| A role | operations the workflow provides to policies | [19](19-letting-policies-act-on-the-outside-world.md) |
| An input adapter | decides where a step's inputs come from | [21](21-deciding-where-a-steps-data-comes-from.md) |
| A value of your own type | a record of yours, held in the data bag like any other value, and read back as its class | [21](21-deciding-where-a-steps-data-comes-from.md) |
| Finishing a journey early | `on step success` returning the lifecycle `FinishWorkflow`, which ends the journey as succeeded | [22](22-finishing-a-journey-early.md) |
| Undoing a step's work when it is given up | `on step failure`, called once the executor gives a step up by its own rule, and the hook that gives it up earlier, both calling the policy's own code | [23](23-undoing-a-charge-when-the-shipping-fails.md) |
| Journeys that run at the same time | each journey on a thread of its own, with an executor of its own, while a step or a hook waits for a service or a role | [26](26-running-async-journeys-while-others-wait.md) |
| A step tested alone | a step built from its inputs, and run with fakes of its services and recorders lending its contributor and reporter | [27](27-testing-a-step-alone.md) |
| A hook or an input adapter tested alone | called as a method, with the data it asks for and fakes of its roles | [28](28-testing-a-hook-or-an-adapter-alone.md) |
| A scripted run | in a test, a journey in which a script says how the steps' attempts end, in place of running them | [29](29-testing-the-flow-with-scripted-steps.md) |
| A fake provider and a journey's path | in a test, a lambda in place of a workflow's provider, and the journey's outcomes and decisions as lines a test compares | [30](30-testing-a-whole-journey.md) |
| A reporter that fails | a reporter whose report fails aborts the journey with `reporter failed`, and receives nothing more | [31](31-stopping-a-journey-when-its-log-fails.md) |
| A reporter that prints every event | a reporter that comes with Itinera, prints each event as it is, and never fails | [32](32-seeing-every-event-of-a-journey.md) |
| Global metadata | a key and a value every reporter of a journey receives, set by the instance before the first event or by a step while it runs | [33](33-giving-every-reporter-what-it-needs.md) |
| What only a hook can know | the journey's ID, the attempt a step hook is about, and the error the step ended in | [34](34-saying-which-attempt-took-the-money.md) |
| A key of its own, and a provider for some steps | an input or a service found under a key other than its field's or parameter's name; a provider attached to some steps only, handing over a client it didn't make, with data saying which it is | [35](35-giving-the-rescue-step-a-service-of-its-own.md) |
| An order stopped before its journey | an instance not created when its journey ID can't be produced, and a journey refused when a reporter can't take its global metadata | [36](36-orders-refused-before-they-start.md) |
| The listing, and the violations of admission | how a workflow is put together, read without running anything, and the mistakes caught before any journey | [37](37-mistakes-caught-before-a-journey.md) |
| A workflow declared by hand | the long form of everything above, for a declaration put together when the program starts, and the violations `build()` throws | [38](38-declaring-by-hand.md) |

## In Java

With the annotations, each line of the table is written as follows. Each name is imported from the package of its concept under `org.itineradev.itinera`, as in `import org.itineradev.itinera.step.Step`, and every item's documentation, which your editor shows, has the rest. A parameter that itinera reads by its name, such as an input, needs the program compiled with `-parameters`, as [chapter 3](03-installing-itinera.md) says.

| What you declare | How you write it | Chapter |
|---|---|---|
| A step | a class marked `@Step` that implements `ItineraStep`, whose public `run` returns an `Outcome` | 4 |
| A step's inputs | a constructor parameter marked `@Input`, such as `@Input long price`; with the type `Optional<Long>`, the input is optional, and empty when it is absent | 4, 20 |
| An outcome | `succeed()`, `fail("code")`, `skip()` or `skipBecause("code")`, which `ItineraStep` gives the class | 4, 16 |
| A workflow and its steps | a class marked `@Workflow(name = "orders")`, then one `@WithStep(step = Charge.class, name = "charge")` line per step, in order, implementing `ItineraWorkflow` | 5, 6 |
| A workflow instance and its initial data | `new Orders().instance().data("price", price).create()`, or `.initialData(order)` with the order as a record | 5, 6, 24 |
| An executor, and running an instance | `var executor = new LocalExecutor()`, then `executor.run(instance)` for each instance, which returns a `JourneyResult`, from `org.itineradev.itinera.journey`, or throws a `Refusal`, from `org.itineradev.itinera.executor` | 5, 24 |
| Reading the result and its output | `result.isSucceeded()` or `result.isFailed()`, or `instanceof` over `result.status()`, a `JourneyStatus.Succeeded`, `Failed`, with its `failure()`, or `Aborted`, with its `abort()`; and `result.output(TypeToken.of(Order.class))` | 5, 6, 25 |
| A contributor | a parameter of `run` of type `Contributor`, then `contributor.contribute("key", value)` | 6 |
| A service and its provider | an interface such as `Payments`; a parameter of `run` marked `@Service`; a public method of the workflow's class marked `@Provider(key = "payments")`, or with `perAttempt = true` for a new object for each attempt, returning `provide(new Bank())`, typed `Provided<Payments>`; a provider for each attempt may ask for `Attempt attempt` | 7, 13 |
| A reporter | a class implementing `Reporter`, from `org.itineradev.itinera.report`, whose `report(Event event)` reads `event.getKind()`, listed with `reporters = {Log.class, Desk.class}` in `@Workflow(...)` | 8, 10 |
| A step's own events | a parameter of `run` of type `StepReporter`, then `reporter.info("...")`, `.warning(...)` or `.error(...)` | 9 |
| A journey ID generator | a public method of the workflow's class marked `@IdGenerator`, asking for `DataBag data` and returning a `String`, such as `data.read("order", TypeToken.of(String.class)).orNull()`, throwing when it is `null` | 11 |
| A retry budget | `retryBudget = 1` in the step's `@WithStep` | 12 |
| An abnormal termination | `run` throws an exception; `noRetryOnAbnormalTermination = true` in the step's `@WithStep` keeps it from spending the retry budget | 12 |
| A rescue step | a class marked `@RescueStep` that implements `ItineraRescueStep`, whose `run` returns a `RescueOutcome` and whose constructor may ask for `Reason reason`; attached with `rescue = RenewCard.class` in the step's `@WithStep` | 14 |
| A step policy and its hooks | a class marked `@StepPolicy(name = "audit")` that implements `ItineraStepPolicy`, with a public method marked with its hook, such as `@OnStepSuccess`, returning an `OnSuccess` made with `keepDefault()`; attached with `policies = {Audit.class}` in a step's `@WithStep` | 15 |
| Giving a step up early | a method marked `@OnStepRetry` or `@OnStepAbnormalTermination`, returning an `OnFailure`, with `failWorkflow("code")` or `keepDefault()` | 17 |
| What a hook requests | parameters marked `@FromStep` or `@FromWorkflow`, such as `@FromStep String receipt`, and parameters asked for by their type alone: `StepName step`, `Optional<Reason> reason`, `Contributor contributor`; a role is marked `@Role`, as `@Role Messages messages` | 15, 17, 18, 19 |
| A workflow policy and its hooks | a class marked `@WorkflowPolicy(name = "notify")` that implements `ItineraWorkflowPolicy`, with methods marked `@OnWorkflowSuccess` or `@OnWorkflowFailure`, returning `void`; attached with `policies = {Notify.class}` in `@Workflow(...)` | 18 |
| A role | an interface marked `@Role`, such as `Messages`, which the workflow's class implements | 19 |
| An input adapter | a public method of the workflow's class marked `@InputAdapter(steps = {"ship"})`, asking for `@Key String key`, and returning `supply(value)` or `fromDataBag()`, both `Supplied` | 21 |
| A value of your own type | a record, such as `record Parcel(long weight, List<Long> size) {}`, whose components are values | 21 |
| Finishing a journey early | `finishWorkflow()`, returned by a method marked `@OnStepSuccess` | 22 |
| Undoing a step's work when it is given up | a method marked `@OnStepFailure`, returning an `OnFailure`, which may ask for `FailureCause cause`; and a private method of the policy, without a mark, that its hooks call | 23 |
| Journeys that run at the same time | one `new LocalExecutor()` for each journey running at the same time, each journey started with `CompletableFuture.supplyAsync` on a pool from `Executors.newFixedThreadPool`, and gathered with `join()` | 26 |
| A step tested alone | in a test file, `new Charge(...).run(TestBank.CHARGES, contributions.getContributor())`, with `Contributions` from `org.itineradev.itinera.journey.testing` and `StepEventReporter` from `org.itineradev.itinera.step.testing`, the outcome compared with `assertEquals`, and what it contributed read with `contributions.read("receipt", TypeToken.of(String.class)).orNull()` | 27 |
| A hook or an input adapter tested alone | in a test file, `new FailedShipping().beforeRetrying(...)`, whose lifecycle says what it decided, as `decision.getFailsWorkflow().code()`, or `new Orders("email").parcels("weight", parcel)`, compared with `new Supplied.Value(60L)` | 28 |
| A scripted run | in a test file, `Scripted.scripted(instance).step("charge").willFail("code").onAttempt(2).willSucceed().andStep("charge_rescue").willSucceed().run()`, with `Scripted` from `org.itineradev.itinera.executor.testing`, which returns a `ScriptedRun` | 29 |
| A fake provider and a journey's path | in a test file, an instance made with `TestInstanceBuilder.of(new Orders("email"))`, from `org.itineradev.itinera.instance.testing`, then `.fakeProvider("bank", Payments.class, () -> new Provided.Instance<>(bank))` and `.reporter(recorder)` for a `RecordingReporter`, from `org.itineradev.itinera.report.testing`, then `recorder.getPath()` | 30 |
| A reporter that fails | `report` throwing, which aborts the journey; the result's `abort().errorMessage()` holds the exception's message, and `abort().error()` the exception | 31 |
| A reporter that prints every event | `DebugReporter`, from `org.itineradev.itinera.report`, in `reporters = {DebugReporter.class, ...}` | 32 |
| Global metadata | `.reportersGlobalMetadata("customer", order.customer())` when the instance is created, beside `initialData`, or `reporter.setGlobalMetadata("receipt", receipt)` in a step, received by a reporter's `setGlobalMetadata(key, value)` | 33 |
| What only a hook can know | parameters `JourneyId journey` and `Attempt attempt`, and `Throwable error` on `on step abnormal termination`, or `Optional<Throwable> error` on `on step retry` and `on step failure` | 34 |
| A key of its own, and a provider for some steps | `@Input("card") String expired` or `@Service("cards") Cards registry`; `@Provider(steps = {"charge_rescue"})`, returning `provide(client, "eu")` | 35 |
| An order stopped before its journey | `create()` throwing an `IllegalStateException` when the ID generator throws; `run` throwing a `Refusal` whose `getRefused()` is a `Refused.GlobalMetadataNotSet` when a reporter's `setGlobalMetadata` throws | 36 |
| The listing, and the violations of admission | `WorkflowDescriptor.of(Orders.class).listing()`, giving a `ListedStep` for each step; with itinera's checker as an annotation processor, a violation is an error of the build; some mistakes don't compile at all | 37 |
| A workflow declared by hand | `WorkflowDescriptor.builder(Orders.class, "orders").step(StepDescriptor.builder("ship", Ship.class).retryBudget(1).build()).role(Messages.class).build()`, which throws a `Violations`; a step without annotations is `StepDescriptor.builder("price", got -> ...)` with a `request(...)` for each request made with `Requests` | 38 |
