# What happens when it runs

## What you want to do

You have a workflow; now you want to run it for real orders, and to know what each part of a run is called: the workflow instance, the journey, the executor, the data bag, the result and the events. You also want to know where Itinera fits in your system, and where it does not.

## The code

This shop is smaller than the one in [chapter 3](03-read-the-workflow-know-what-happens.md): three steps, the `audit` and `notify` policies, and no card renewal or backup carrier, so that its events stay short. The workflow also lists a **reporter**, `Log`, which receives everything that happens.

```example
name: ch04_running
region: workflow
asynchronous: ch04_running_async
```

A journey starts with what the shop knows about the order:

```example
name: ch04_running
region: data
```

The reporter prints every event it receives, one line each:

```example
name: ch04_running
region: reporter
asynchronous: ch04_running_async
```

The program runs two orders. Ann's card is accepted; the bank refuses Dan's.

```example
name: ch04_running
region: run
asynchronous: ch04_running_async
```

It prints:

```output
name: ch04_running
```

```note
name: ch04_running
```

## How it works

1. **Workflow instance.** For every order, your code creates a new **workflow instance** from the workflow, with that order's initial data. The instance holds everything about that order's run: its data, its journey ID and its reporters. An instance runs once; the next order gets a new one.
2. **Journey.** A **journey** is one run of an instance, from its first step to its result. The output shows two journeys, one per order.
3. **Executor.** The **executor** runs the instance. It builds each step afresh for every attempt, runs it, calls the hooks of the attached policies after each outcome, and carries out what they decide. The executor here runs the journey in your process and keeps nothing once it returns: no data, no state, nothing saved to disk. It runs one journey at a time; to run journeys side by side, you create several executors.
4. **Data bag.** The **data bag** holds the journey's data, as values under string keys. It starts with the initial data: five keys for Ann's order, four for Dan's, which has no discount. It changes only when a contribution is committed: a step's contributions when its attempt succeeds, a hook's when the hook returns. That is why Ann's data bag ends with `charged`, `audit` and `tracking`, and Dan's holds only what it started with. When the journey ends, the data bag is its output.
5. **Result.** What `run` returns is the **result**: the journey ID, how the journey ended, with what explains it, and, unless it was aborted, the data bag. A journey **succeeded** when every step succeeded or skipped itself, or a hook finished it early. It **failed** when a step failed or a hook failed it, and the result says why: Dan's failed because the charge reported a failure, `declined`. It was **aborted** when something illegal happened, such as a hook that failed or required data that was missing; an aborted journey has no output and is never resumed. When a journey cannot even start, for example because the workflow is put together wrongly, `run` returns a **refusal** instead of a result: no journey, and no events.
6. **Events.** Everything that happens in a journey is an **event**, numbered in order, and the workflow's reporters receive every one. Some events are facts: `step_failed` says the charge failed, with its reason. Others are decisions: `step_given_up` says the executor will not try the charge again, and why, and `journey_failed` says the journey failed. Read Dan's journey line by line and you see the order of things: the step's outcome, the decision about the step, the workflow hook that told Dan, then the decision about the journey. Every journey reports the same kinds of event in the same order, whoever wrote the steps, so one reporter can log every workflow you have.
7. **Execution modes.** A step, a hook or a reporter is either **synchronous** or **asynchronous**, and an executor accepts one or both modes. In the asynchronous form of this program, every step, hook and reporter is asynchronous and runs on the asynchronous executor; it prints exactly the same. Something in a mode the executor does not accept is refused before the journey starts.

### Where Itinera fits

Itinera fits a piece of business work made of steps that run in a fixed order, with decisions around them: try again, repair and try once more, give up, finish early, tell someone. A checkout, a sign-up, an invoice run or a refund are good examples. What Itinera is not:

1. **Not a job queue.** It does not schedule work or hand it to other machines. `run` runs one journey, now, in your process, and returns its result. It never waits between attempts: a step that needs to wait does so itself.
2. **Not BPMN.** There is no diagram language and no engine server. A workflow is a declaration in your own code, built and checked with it.
3. **No branching inside a workflow.** The steps always run in the order the workflow lists them. Nothing reorders them or jumps between them. A step can skip itself, and a policy can end the journey early, and that is all.
4. **Not durable, yet.** A journey lives only as long as the call that runs it. Nothing is saved, so a journey cut short by a crash is not resumed.

## Remember

1. Create a new workflow instance for every journey; an instance runs only once.
2. Only committed contributions change the data bag: a step's when its attempt succeeds, a hook's when it returns.
3. Read how a journey ended from its result, and what happened during it from its events.
4. The steps run in the order the workflow lists them; nothing branches, reorders or jumps.
5. An aborted journey has no output, and it is never resumed.
