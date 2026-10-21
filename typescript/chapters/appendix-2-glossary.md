# Glossary

Every word of Itinera's vocabulary, in plain language, grouped as the specification groups them. The words are not synonyms of each other: each means exactly one thing, defined in [chapter 1 of the specification](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md), which each group links to. Where this glossary and the specification differ, the specification is right. Where it helps, an entry says what the word looks like in TypeScript; the documentation of each item of `itinera`, which your editor shows, has the rest.

## The idea

Defined in [1.1, The idea](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md#11-the-idea).

- **Business rule:** a decision about what the business does, such as whether an order can be charged. Business rules live in steps.
- **Flow control:** a decision about what happens around that work: try again, give up, finish early. Flow control lives in policies, whose hooks the executor calls after each outcome. A step cannot reach the workflow that runs it, so the two kinds of decision stay apart by construction, not by discipline.

## Workflows and steps

Defined in [1.2, Workflows and steps](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md#12-workflows-and-steps).

- **Workflow:** the plan. The steps in the order they run, with the policies, input adapters, service providers, roles and reporters attached to it. Everything in it is fixed before anything runs, and it can be listed without running it. In the shop: look the order up, price it, charge it, then ship it. In TypeScript, a class extending `Workflow`, marked `@workflow(...)`, with a `@withStep(...)` line for each step.
- **Step:** one business unit, such as charging an order. It has a name, unique within its workflow, though the step itself never knows that name. It declares the data and services it needs, does its work, contributes data back and reports an outcome. It knows nothing of the workflow, the executor or the other steps. In TypeScript, a class extending `Step`, marked `@step()`, with its `run`.
- **Workflow descriptor:** the workflow's declaration: its name, its step descriptors, its workflow policy descriptors, its input adapters and its service providers. It never changes once made, and every instance of the workflow shares it. In TypeScript, the decorators record it from your classes, and the builder takes it part by part.
- **Step descriptor:** what the workflow holds for one step: its name, how to build it, its retry budget, whether an abnormal termination may be retried, the step policies attached to it, its input adapter, if any, and its rescue step, if any. What it needs includes the keys and types of the services the step asks for.
- **Policy descriptor:** the description of one policy: its name, the hooks it defines, and how to build an instance of it.
- **Outcome:** what a step reports when it runs to the end, in TypeScript an `Outcome` that `run` returns, or resolves to. It says what happened, never what to do next. It is one of:
  - **Success:** the step did its work.
  - **Failure:** the step could not do its work. A failure carries a **reason**, made of a code such as `card-expired`, an optional message and optional details. Whether the step is tried again is never its own decision: its retry budget, and the policies attached to it, decide.
  - **Skipped:** the step decided, from what it knows of its domain, not to do its work, as the shipping does for a gift card. A skipped step leaves the flow.
- **Abnormal termination:** a recoverable failure that escapes a running step, instead of an outcome the step chose. In TypeScript, anything a step's `run` throws, or a rejection of the promise it returns. It ends the attempt as a failure, and spends the retry budget as a failure does, unless the step descriptor turns that off.
- **Attempt:** one execution of a step. The first is attempt 1.
- **Retry:** any attempt after the first, except the attempt that follows a rescue step.
- **Retry budget:** how many retries a step allows. Shipping with a retry budget of 1 is tried at most twice.
- **Rescue step:** a step that fixes the cause of another step's failure, after which the failed step runs once more. A step has at most one, and a step that has one has no retry budget. It is named after the step it rescues, such as `charge_rescue`, and runs when that step fails or ends in an abnormal termination. It inherits nothing from that step: an input adapter or service provider of its own is attached to its own name. If it succeeds, the failed step runs once more; a step is rescued at most once per journey. If it fails, the journey fails, and the failed step never runs again. The specification aborts a journey whose rescue step skips, with `rescue skipped`, but in TypeScript you can't write one: a `RescueOutcome` is never a skip.
- **Service:** an object of operations a step uses to do its business, such as a payment client or a carrier, asked for by a key and a type. It is not a value, so it never enters the data bag, an event or the result. Services carry operations for steps as roles carry operations for policies. In TypeScript, a step asks for one with a parameter of its `run` marked `@service(...)`, typed by its interface, such as `Carrier`.
- **Service provider:** the named part of the workflow that supplies the service of one key, either to every step that asks for that key or to the steps it is attached to, which it then serves instead of the provider of the same key that serves the others. Its **scope** says how long what it makes is used: the whole **journey**, one instance shared by every step that asks for it until the journey ends, or one **attempt**, a new instance for every attempt. In TypeScript, a method of the workflow's class marked `@provider(...)`.
- **Step status:** where a step stands in a journey: `NotExecuted` (not run yet, or never), `Executing` (an attempt is running), `MustRetry` (it failed and will be tried again), `Rescuing` (it failed and its failure was handed to its rescue step; it stays so until it runs again, or for good if the rescue fails), `Succeeded`, `Failed` (it will not be tried again), `Skipped`, or `Aborted` (the journey was aborted while it was running, waiting to retry, or being rescued).

## Data

Defined in [1.3, Data](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md#13-data).

- **Value:** data that can be written down and read back: null, true or false, numbers, text, lists and objects, and any type that turns into them. In TypeScript, plain data, such as `null`, booleans, numbers, strings, and arrays and plain objects of them. Functions, `undefined`, connections and handles are not values.
- **Value conversion:** code of your own, or code you customise, that Itinera calls to turn one of your objects into a value as it is stored, or to turn a value back into the type asked for, or check that it is of that type, as it is read. One that fails while a value is read makes the value of the wrong type, and one that fails while a value is stored makes it not a value.
- **Data bag:** the journey's data: a map from text keys to values. It starts with the instance's initial data, and changes only when a contribution is committed. When a journey succeeds or fails, its data bag is the journey's output.
- **Data from the workflow:** a value a step, hook, input adapter or service provider asks for by a key and a type, as **required** (the journey can't go on without it) or **optional** (it may be absent, an `Option` in TypeScript, asked for with `optional(...)`). Where it comes from is the workflow's decision, never the asker's. In TypeScript, the type is a type token, such as `integer`, which checks the value.
- **Input adapter:** a named part of the workflow, attached to one or more of its steps, that decides where a step's inputs come from. A step has at most one. Asked for each input when the step is built, it returns a value, or nothing, in which case the input is read from the data bag. Without one, every input comes from the data bag under its key. In TypeScript, a method of the workflow's class marked `@inputAdapter(...)`, returning `this.supply(value)` or `this.fromDataBag()`.
- **Adapter:** in this version, always an input adapter. There are no adapters for data from a step.
- **Data from the step:** a value a step hook asks for from the step it acts on, or a rescue step from the step it rescues, by key and type, required or optional. It is exactly what that step contributed.
- **Contributor:** what a step or a hook is given to add data, as key and value pairs. It is the only way to change the data bag, and it works only during the attempt or hook it was given for. In TypeScript, a `Contributor`, asked for with `@contributor()`.
- **Contribution:** one key and value given to a contributor. A step's contributions are **committed**, that is written to the data bag, only when its attempt succeeds; a hook's are committed when the hook returns.

## Policies, hooks and lifecycles

Defined in [1.4, Policies, hooks and lifecycles](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md#14-policies-hooks-and-lifecycles).

- **Policy:** a reusable object that defines one or more hooks. A **step policy** defines only step hooks and is attached to steps, such as `audit` on the charge. A **workflow policy** defines only workflow hooks and is attached to workflows, such as `notify`. In TypeScript, a class extending `StepPolicy` or `WorkflowPolicy`, marked `@stepPolicy(...)` or `@workflowPolicy(...)`, with a method marked for each hook it defines.
- **Hook:** a function of a policy that the executor calls. **Step hooks** (`on step success`, `on step failure`, `on step retry`, `on step abnormal termination`) are called in response to a step's outcome. **Workflow hooks** (`on workflow success`, `on workflow failure`) are called when the journey ends. A hook that no attached policy defines keeps its default behaviour.
- **Lifecycle:** what a hook may return to tell the executor to do something other than the default. In tier 1 there are two: `FailWorkflow`, which fails the journey at once, and `FinishWorkflow`, which ends it at once as succeeded. In TypeScript, each hook's return type admits only the lifecycles that hook may return.
- **Cause:** why the failure hook or the retry hook was called, or why a step was given up, retried or rescued: `failure`, `abnormal termination` or `retries exhausted`, and, for a step given up by a hook, `FailWorkflow`.
- **Workflow role:** an interface of operations that a workflow implements and offers to policies, such as telling the customer something. A hook asks for the roles it needs. Roles carry operations, never data. In TypeScript, an interface that the workflow's class implements, which a hook asks for with `@role<Messages>()`.

## Running

Defined in [1.5, Running](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md#15-running).

- **Workflow instance:** what your own code creates from a workflow for each journey, with that journey's initial data, its journey ID, its reporters and its initial global metadata. It holds the data bag and the rest of the journey's state. In TypeScript, made with `instance(...)` on the workflow object that `Orders.build()` makes.
- **Custom code:** any code of yours that the executor calls: building and running steps, input adapters, service providers, policies and their hooks, role operations, reporters, and dispatcher factories and dispatchers.
- **Executor:** runs a workflow instance, and carries out what the hooks decide. The **local executor** runs a journey inside your program, keeping nothing once it ends: `LocalExecutor` in TypeScript, whose `run` returns a promise.
- **Journey:** one execution of a workflow instance from start to end: one order going through the shop. In later tiers a journey may move between workflows.
- **Run:** one workflow executed within a journey. In this version a journey has exactly one run, so the two are the same.
- **Journey ID:** the identifier of a journey, made while the instance is created; a random UUID unless the workflow makes its own, for example from the order.
- **The scan:** how the executor chooses what to do next: again and again, it looks at the first step that is not finished.
- **Result:** what running a journey returns, in business terms: the journey ID, how it ended (succeeded, failed or aborted) with what explains it, and, unless it was aborted, the journey's output. Which steps ran, and how, is in the event stream instead. In TypeScript, a `JourneyResult`, whose `status` says how the journey ended, and whose output `result.output(...)` reads, each value checked by its type token.
- **Failed journey:** a journey that ended because a step failed or a hook returned `FailWorkflow`. A failure is a legitimate end, decided by the outcomes and the policies, such as a declined card.
- **Aborted journey:** a journey stopped because something illegal happened: a configuration error found while running, a hook or reporter that failed, or something stored or emitted that is not a value. No hook runs after an abort, and the journey has no output and is never resumed. Each abort has an **abort reason**, such as `required data missing`.
- **Unrecoverable failure:** a failure the language itself treats as unrecoverable, such as a panic in Rust. It is outside the model: the journey stops as if the program had crashed. JavaScript has no failure that code can tell apart from a recoverable one, so in TypeScript there is none: Itinera catches whatever your code throws while a journey runs.

## Events

Defined in [1.6, Events](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md#16-events).

- **Event:** a record of something that happened in a journey, with fixed fields. **Engine events** come from the executor. A step can also emit its own events, `step_info`, `step_warning` and `step_error`, and a hook can emit `journey_info`, `journey_warning` and `journey_error`. In TypeScript, an `Event`, told apart by its `kind`.
- **Fact** and **decision:** the two kinds of engine event. A fact records something that happened, such as a step succeeding. A decision records what the executor decided to do next, such as retrying a step or failing the journey, and who decided it.
- **Event stream:** every event of one journey, in order. It is never rolled back or rewritten. It is how you observe a journey.
- **Reporter:** receives events one at a time, and the journey's global metadata, and decides on its own what to do with them, such as writing them to a log. The workflow lists its reporters. A step receives a **step reporter** to emit its own events and set global metadata. In TypeScript, a reporter implements `Reporter`, and a step asks for its `StepReporter` with a parameter marked `@stepReporter()`.
- **Dispatcher:** holds a journey's reporters and delivers each event and each entry of global metadata to them. The executor creates one for each journey, from a **dispatcher factory**.
- **Global metadata:** entries of a key and a value, kept apart from the data bag, that every reporter receives as soon as each is set: the journey ID, the entries the instance was created with, and what steps, hooks and service providers set while the journey runs. Nothing in the journey can read it; it exists for reporters, for example to label every line of a log with the customer. In TypeScript, given with `reportersGlobalMetadata` when the instance is created, set with `setGlobalMetadata` on a step reporter, and received in a reporter's `receiveGlobalMetadata`.

## Configuration and admission

Defined in [1.7, Configuration and admission](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md#17-configuration-and-admission).

- **Configuration error:** anything wrong with how a workflow is put together or supplied, which its author is responsible for.
- **Violation:** one configuration error, by name, such as `duplicate step name`. In TypeScript, every violation found is listed in one `Violations`.
- **Refusal:** what running an instance reports instead of a result when a journey can't start: the workflow was refused at admission, or your code called before the journey starts failed. A refusal has no journey and no events. In TypeScript, the `Refusal` that the promise of `run` rejects with.
- **Admission:** checking how a workflow is put together before any journey: when the workflow is built, and, for what depends on the executor, when the instance is handed to it. A workflow refused at admission has no journey and no events.

## Tiers and capabilities

Defined in [1.8, Tiers and capabilities](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md#18-tiers-and-capabilities).

- **Tier:** a level of what Itinera means, each including everything before it. Tier 1 is a single workflow. What a tier contains is fixed for each version of the specification.
- **Capability:** a property of an executor or a language that varies independently of the tier. In tier 1, the capabilities are the execution modes an executor accepts, `sync` and `async`, test support and value conversions. Which of them `itinera` claims is in [What this version supports](appendix-3-what-this-version-supports.md).
- **Test support:** the capability `testing`, which lets you test your workflows: a recording reporter and the path it reads, fake providers, scripted runs and recorders. It does not count towards a tier. In TypeScript, it is `itinera/testing`, which only tests import.
- **Value conversions:** the capability `value-conversions`, claimed by a language whose executor calls value conversions while a journey runs.
- **Recording reporter:** a reporter of test support that keeps every event it receives and the latest value of each entry of global metadata. In TypeScript, a `RecordingReporter`.
- **Path:** a journey's flow as plain sentences, one line per attempt's outcome, step decision and journey end, read from a recording reporter.
- **Fake provider:** a test's own function that replaces one service provider of an instance, keeping its attachment, scope and requests.
- **Scripted run:** a journey whose steps are all built for real, in which a script says what some attempts do instead of running their steps; it gives the result, with a **record** of each attempt handed to a step's construction.
- **Recorder:** test support that lends a step, hook or input adapter a handle outside any journey, such as a contributor, and keeps what it was given. In TypeScript, `Contributions` and `StepEventReporter`.
- **Mistake in a test:** a misuse of test support, such as a script that leaves out an attempt, which stops the test before the journey. In TypeScript, a `TestMistake`.
- **Execution mode:** whether a step, hook or reporter is synchronous or asynchronous. In TypeScript, each has its own: a step's `run`, a hook or a reporter's method may return a value or a promise of one, and the executor waits for whichever it gets.
