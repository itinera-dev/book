# Mistakes caught before a journey

## What you want to do

The shop's workflow is put together from many parts: steps with their names, policies attached to them, input adapters, roles. You want to know which mistakes in putting it together are caught before any order is charged, what you see when one is, and how to fix each.

## The code

### A workflow put together wrongly

This declaration of the shop has three mistakes: the ship step is listed twice under the same name, two policies attached to the charge both react to its success, and the pricing adapter is attached to `charging`, a step the workflow does not have.

```example
name: ch27_violations
region: wrong
```

Building it does not give you a workflow. It gives you every violation found, which you can read one by one.

```example
name: ch27_violations
region: report
```

Fixed, the declaration lists each step once, keeps one policy for the charge's success, and attaches the adapter to `charge`.

```example
name: ch27_violations
region: fixed
```

It prints:

```output
name: ch27_violations
```

```note
name: ch27_violations
```

### Caught when the program is compiled

Depending on the language, and on how you declare the workflow, some mistakes never get as far as building it: the program does not compile. Here is the ship step listed twice:

```compile-error
name: ch27_duplicate_step_name
```

Two policies of the charge that both define `on step success`:

```compile-error
name: ch27_hook_defined_twice
```

```note
name: ch27_compiled
```

A language may also make some rules impossible to break, through its types. A policy that tells the customer through the `Messages` role, attached to a workflow that does not provide that role:

```compile-error
name: ch27_role_not_provided
```

An asynchronous workflow handed to an executor that runs only synchronous ones:

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

A workflow can be put together correctly and still fail to start a journey, when something it supplies fails before the journey starts. Here the `notify` policy is built from the shop's message templates, and the template for a completed order is missing.

```example
name: ch27_refusal
region: policy
```

The executor builds the workflow's policies before the journey starts, so `run` refuses the instance instead of returning a result.

```example
name: ch27_refusal
region: run
```

It prints:

```output
name: ch27_refusal
```

```note
name: ch27_refusal
```

The workflow lists a reporter that prints every event, and it printed nothing: a refused journey has no events.

## How it works

1. **Admission happens before any journey.** Checking how a workflow is put together is called admission. It happens when the workflow is built, for everything that needs only its declaration, and again when an instance is handed to the executor, for what depends on the executor. Either way, no journey starts, no event is emitted, and you receive every violation found, not only the first, so you can fix them all at once.
2. **A violation is one mistake, by name.** These are the violations the specification names, and how to fix each:

   | Violation | What is wrong | How to fix it |
   |---|---|---|
   | `duplicate step name` | two steps of the workflow have the same name | give each step its own name; the same kind of step may be listed twice under two names |
   | `hook defined twice` | two policies attached to one step, or to the workflow, define the same hook | merge them into one policy, or attach one of them elsewhere |
   | `role not provided` | a policy requests a role the workflow does not provide | provide the role from the workflow, or attach the policy to a workflow that does |
   | `mode not accepted` | a step, hook or reporter is asynchronous and the executor runs only synchronous ones, or the reverse | run the instance on an executor that accepts that mode, or write the part in the executor's mode |
   | `step adapted twice` | two input adapters are attached to one step | merge them into one adapter for that step |
   | `input adapter for unknown step` | an input adapter is attached to a step the workflow does not have | attach it to a step the workflow lists, spelled as the workflow names it |

3. **Initial data has to be values.** Initial data holding something that is not a value is refused too, when the instance is handed to the executor, naming each key concerned.
4. **Some languages catch these even earlier.** The specification asks implementations to report violations at compile time where the language allows it. Some rules can be made impossible to write at all: a hook returning a lifecycle it may not return, a policy attached to a workflow that does not provide its role, a part in a mode the executor does not accept, something that is not a value in the data bag or an event, a step's contributor kept after its attempt, and an input adapter writing through its read access to the data bag. In such a language those mistakes never reach a journey.
5. **A refusal is not a result.** When a journey cannot start, `run` reports a refusal instead of a journey result. That happens when the workflow is refused at admission, and when the workflow's own code called before the journey starts fails: a workflow policy while it is built, or the dispatcher factory and dispatcher that deliver the events. A refusal has no journey and no events. It means the workflow was put together or supplied wrongly, never that the order failed.
6. **One instance, one journey.** Handing the executor an instance that has already run is refused as well, as [chapter 21](21-running-many-journeys.md) shows.
7. **The rest shows up during a journey.** Some mistakes cannot be seen before a journey runs: a step that cannot be built, required data missing from the data bag, a value of the wrong type, a hook returning a lifecycle it may not return in a language that allows writing it. Each aborts the journey with its own abort reason, as [chapter 22](22-reading-the-result.md) shows. A step policy, built afresh for every attempt, is one of them: when it fails while being built, the journey is aborted, not refused.
8. **More violations arrive with services and rescue steps.** A step that requests a service no provider supplies, or a step with both a rescue step and a retry budget, will be refused at admission in the same way, as `service not provided` and `rescued step retried`. [Chapter 12](12-giving-a-step-the-tools-it-needs.md) and [chapter 14](14-repairing-then-trying-once-more.md) show them.

## Remember

1. A workflow refused at admission has no journey and no events: fix the declaration, never work around the refusal at run time.
2. Admission reports every violation at once, so read them all before building again.
3. Step names are unique within a workflow, and each hook is defined at most once for a step and once for a workflow.
4. A policy is attached only to a workflow that provides every role the policy requests.
5. A refusal means the workflow was put together or supplied wrongly; it is never a business outcome.
6. Mistakes that can only show up during a journey abort it; they are never refusals.
