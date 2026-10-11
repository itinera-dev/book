# Cheat sheet

Everything you declare with Itinera, in the order the book teaches it, with the chapter to read for more. The note that follows the table gives, for each line, how your language writes it.

## What you declare

| What you declare | What it does | Chapter |
|---|---|---|
| A step | holds one business rule, runs once per attempt and reports an outcome | [6](06-a-workflow-of-one-step.md) |
| A step's inputs, required or optional | the data a step needs, which the workflow supplies | [8](08-giving-a-step-what-it-needs.md) |
| A contributor | how a step adds data to the data bag | [9](09-passing-data-on.md) |
| An outcome | success, failure with a reason, a failure that might work next time, or skipped | [10](10-saying-how-it-went.md) |
| An abnormal termination | an error that escapes a running step | [11](11-when-something-goes-wrong-inside-a-step.md) |
| A service and its provider | a tool a step asks for by a key, and the part of the workflow that supplies it, per journey or per attempt | [12](12-giving-a-step-the-tools-it-needs.md) |
| A workflow and its steps | the plan: the steps in order, each under its name | [6](06-a-workflow-of-one-step.md), [7](07-adding-a-step.md) |
| A retry budget | how many times a step may be tried again | [13](13-trying-again.md) |
| A rescue step | a step that repairs the cause of a failure before the step runs once more | [14](14-repairing-then-trying-once-more.md) |
| A step policy and its hooks | reacts to a step's outcome: `on step success`, `on step failure`, `on step retry`, `on step abnormal termination` | [15](15-reacting-to-what-a-step-did.md) |
| What a hook requests | data from the step or the workflow, the reason, the cause, the error, the step's name, the attempt, the journey ID, a role, a contributor, a reporter | [15](15-reacting-to-what-a-step-did.md) |
| A lifecycle | what a hook returns to end the journey early: `FailWorkflow` or `FinishWorkflow` | [16](16-ending-a-journey-early.md) |
| A workflow policy and its hooks | reacts to how the journey ended: `on workflow success`, `on workflow failure` | [17](17-doing-something-when-the-journey-ends.md) |
| A role | operations the workflow provides to policies | [18](18-letting-policies-act-on-the-outside-world.md) |
| An input adapter | decides where a step's inputs come from | [19](19-deciding-where-a-steps-data-comes-from.md) |
| A journey ID generator | names each journey, for example after the order | [20](20-naming-your-journeys.md) |
| A workflow instance and its initial data | one journey's starting point | [21](21-running-many-journeys.md) |
| An executor, and running an instance | runs one journey and returns its result, or refuses it | [21](21-running-many-journeys.md) |
| Reading the result and its output | how the journey ended, and the data bag it left | [22](22-reading-the-result.md) |
| A reporter | receives every event of a journey | [23](23-seeing-what-happened.md) |
| A step's own events | what only the step knows, emitted as `step_info`, `step_warning` or `step_error`; a hook emits `journey_info`, `journey_warning` or `journey_error` | [23](23-seeing-what-happened.md) |
| A policy that may fail while it is built | a policy built by a function of your own, which may fail; a workflow policy that fails is a refusal | [27](27-mistakes-caught-before-a-journey.md) |
| A workflow declared by hand | the long form of everything above | [28](28-declaring-by-hand.md) |

## In your language

```note
name: appendix1_cheat_sheet
```
