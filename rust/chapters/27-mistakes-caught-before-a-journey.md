# Mistakes caught before a journey

## What you want to do

The shop's workflow is put together from many parts: steps with their names, policies attached to them, input adapters, roles, services and their providers. You want to know which mistakes in putting it together are caught before any order is charged, what you see when one is, and how to fix each.

## The code

### Caught when the program is compiled

A workflow declared with `#[workflow(...)]` and put together wrongly never gets as far as being built: the program doesn't compile. Here is the ship step listed twice:

```compile-error
name: ch27_duplicate_step_name
```

Two policies of the charge that both define `on step success`:

```compile-error
name: ch27_hook_defined_twice
```

Every violation in the table below is a compile error in a workflow declared with `#[workflow(...)]`, together with a few mistakes of the attributes themselves, such as a workflow without steps. Some of the messages say "evaluation panicked": the check runs in a constant the compiler evaluates, so it's still a compile error, and nothing panics when the program runs. A workflow declared by hand compiles with the same mistakes, and they are reported as data when it is built, as [chapter 28](28-declaring-by-hand.md#a-workflow-put-together-wrongly) shows.

### Rules Rust makes impossible to break

Some rules go further: through Rust's types, a mistake can't be written at all, in either form. A policy that tells the customer through the `Messages` role, attached to a workflow that doesn't provide that role:

```compile-error
name: ch27_role_not_provided
```

An asynchronous workflow handed to `LocalExecutor`, which runs only synchronous ones:

```compile-error
name: ch27_mode_not_accepted
```

A hook that tries to end the journey as succeeded after the charge failed, which `on step failure` may not do:

```compile-error
name: ch27_invalid_lifecycle
```

A step that contributes its connection to the bank, which is not a value and can never enter the data bag:

```compile-error
name: ch27_not_a_value
```

### Refused by run

A workflow can be put together correctly and still fail to start a journey, when something it supplies fails before the journey starts. [Chapter 21](21-running-many-journeys.md) showed one: the `notify` policy needs the texts the shop sends, and the file that holds them is missing.

```example
name: ch21_refused
region: policy
```

The executor builds the workflow's policies before the journey starts, so `run` refuses the instance instead of returning a result.

```example
name: ch21_refused
region: run
```

It prints:

```output
name: ch21_refused
```

The refusal is all there is to see. A refused journey has no events: had the workflow listed a reporter, it would have received none.

## How it works

1. **Admission happens before any journey.** Checking how a workflow is put together is called admission. It happens when the workflow is built, for everything that needs only its declaration, and again when an instance is handed to the executor, for what depends on the executor. Either way, no journey starts, no event is emitted, and you receive every violation found, not only the first, so you can fix them all at once.
2. **A violation is one mistake, by name.** These are the violations the specification names, and how to fix each:

   | Violation | What is wrong | How to fix it |
   |---|---|---|
   | `duplicate step name` | two steps of the workflow have the same name, or a step has the name of another step's rescue step, such as `charge_rescue` | give each step its own name; the same kind of step may be listed twice under two names |
   | `hook defined twice` | two policies attached to one step, or to the workflow, define the same hook | merge them into one policy, or attach one of them elsewhere |
   | `role not provided` | a policy requests a role the workflow doesn't provide | provide the role from the workflow, or attach the policy to a workflow that does |
   | `mode not accepted` | a step, hook, service provider or reporter is asynchronous and the executor runs only synchronous ones, or the reverse | run the instance on an executor that accepts that mode, or write the part in the executor's mode |
   | `step adapted twice` | two input adapters are attached to one step | merge them into one adapter for that step |
   | `input adapter for unknown step` | an input adapter is attached to a step the workflow doesn't have, or to a rescue step's name, such as `charge_rescue`, when the step has no rescue step | attach it to a step the workflow lists, or to the rescue step of one that has a rescue step, spelled as the workflow names it |
   | `rescued step retried` | a step has both a rescue step and a retry budget above 0 | keep one of them: a step with a rescue step is never retried |
   | `service not provided` | a step requests a service that no provider serves for it | add a provider of that key, or attach an existing one to the step too |
   | `service requested twice` | a step requests the same key twice among its services | request it once, and use it wherever the step needs it |
   | `service provided twice` | two providers of one key are both attached to the same step, or both serve every step | keep one of them, or attach each to different steps |
   | `duplicate provider name` | two service providers of the workflow have the same name | give each provider its own name |
   | `service provider for unknown step` | a service provider is attached to a step the workflow doesn't have, or to a rescue step's name when the step has no rescue step | attach it to a step the workflow lists, or to the rescue step of one that has a rescue step |
   | `wrong service type` | the provider serving a step supplies another type than the step requests for that key | make the provider return what the step asks for, or ask for the key the provider serves |
   | `attempt request in journey scope` | a service provider of journey scope requests the name of the step being built or the attempt number | make it a provider for every attempt, or stop requesting them |

3. **Rust catches these as early as it can.** The specification asks implementations to report violations when the program is compiled wherever the language allows it, and Rust allows a lot. With the attributes, every violation in the table is a compile error, those about services included. The one exception is rare: when the providers of one key write the service type they return in different ways, or one of them writes its return type other than as `Result<Provided<T>, ...>`, a step or rescue step asking for another type is only found when the program first uses the workflow, which panics with `wrong service type`. Declared by hand, as chapter 28 shows, they are reported when the workflow is built, as the program runs and before any journey, except `role not provided`, `mode not accepted` and `attempt request in journey scope`, which Rust's types rule out in either form.
4. **Some rules can't be broken at all.** The specification lets a language make some mistakes impossible to write, and `itinera` lists the ones it does in its `conformance.json`, each proven by a test of its own:
   - a hook returning a lifecycle it may not return: each hook's return type lists only the lifecycles that hook may return. `on step failure` returns a `Result<Option<FailWorkflow>, Error>`, and `Self::finish_workflow()` gives a `Result<Option<OnSuccess>, Error>`, the return type of `on step success`, the only hook that may return `FinishWorkflow`. The types don't match, which is what the message says;
   - a policy attached to a workflow that doesn't provide its role: the workflow must implement `Provides<dyn Messages>`, which `#[role]` does on its implementation of the role's trait;
   - a part in a mode the executor doesn't accept: `LocalExecutor` takes only synchronous instances, and `AsyncLocalExecutor` only asynchronous ones;
   - something that is not a value in the data bag, in initial data, in an input adapter's answer, in a reason's details, in an event or in global metadata: all of them must implement `Value`, which needs the type to be `Clone` and to serialise and deserialise with serde;
   - a step's contributor or step reporter used after its attempt, or an input adapter's access to the data bag used after its call: their lifetimes end there;
   - an input adapter writing through its access to the data bag, which is read-only;
   - a rescue step that skips: a `RescueOutcome` has no skip;
   - a provider of journey scope requesting the name of the step being built or the attempt number, the violation `attempt request in journey scope`: only a provider marked `#[provider(per_attempt)]` may take a `#[step_name]` or `#[attempt]` parameter;
   - a provider that returns no service: a provider returns a `Provided` of the service, never nothing.
5. **A refusal is not a result.** When a journey can't start, `run` returns a `Refusal` as its `Err` instead of a journey result. The specification refuses a journey when the workflow is refused at admission, and when the workflow's own code called before the journey starts fails. In Rust, admission has almost all happened by the time you call `run`: what's left of it is an instance whose global metadata sets `journey_id`, the key only the executor sets. The rest is that code: a workflow policy while it is built, or the dispatcher factory, the dispatcher and the reporters while they are given the journey ID and the instance's global metadata. A refusal has no journey and no events. It's never a business outcome: nothing ran, so fix the cause and run a new instance.
6. **One instance, one journey.** The specification refuses an instance that has already run. In Rust you can't even try: `run` takes the instance by value, as [chapter 21](21-running-many-journeys.md) shows.
7. **The rest shows up during a journey.** Some mistakes can't be seen before a journey runs: a step that can't be built, required data missing from the data bag, or a value of the wrong type. Each aborts the journey with its own abort reason, as [chapter 22](22-reading-the-result.md) shows. A step policy, built afresh for every attempt, is one of them: when its `#[factory]` fails, the journey is aborted, not refused.
8. **Where the rules come from.** `service not provided` is the violation of a step asking for a service that no provider of the workflow serves; [chapter 7](07-giving-a-step-the-tools-it-needs.md) teaches services and providers. The rule behind `rescued step retried`, that a step with a rescue step is never retried, is in [chapter 14](14-repairing-then-trying-once-more.md), with what may be attached to a rescue step's name.

## Remember

1. A workflow refused at admission has no journey and no events: fix the declaration, never work around the refusal at run time.
2. Admission reports every violation at once, so read them all before building again.
3. Step names are unique within a workflow, and each hook is defined at most once for a step and once for a workflow.
4. A policy is attached only to a workflow that provides every role the policy requests.
5. A refusal is never a business outcome: nothing ran, so fix its cause and run a new instance.
6. Mistakes that can only show up during a journey abort it; they are never refusals.
