# Repairing, then trying once more

## What you want to do

Cards expire. When the charge fails because the customer's card has expired, trying the same card again is pointless, but the customer's bank usually knows the card that replaced it. You want the shop to ask the bank for the new card, then charge once more, without the charge step knowing anything about renewing cards.

## The code

Rescue steps aren't in `itinera` yet. The specification defines them, and they'll come in a later release; until then, this chapter's program waits in `pending/` and isn't compiled. It follows the planned syntax, which may still change.

The charge step charges the card it is given. An expired card is a failure, with the reason `expired-card`, and nothing more: the step doesn't try to fix it.

```example
name: ch14_rescue
region: charge
```

The repair is a step of its own, `RenewCard`: the charge's rescue step, which runs when the charge fails, to fix what made it fail. It is a struct that derives `RescueStep` rather than `Step`, and its `run` is in a block marked `#[rescue_step]`. It reads why the charge failed, through its field marked `#[reason]`. When the card had expired, it asks the bank for the new one and contributes it under `card`, the key the charge reads. Otherwise there is nothing it can renew, and it fails with `nothing-to-renew`: when the charge failed for another reason, when it ended in an abnormal termination, which gives no reason at all, or when the bank has no new card.

```example
name: ch14_rescue
region: rescue
```

The workflow attaches the rescue step to the charge, with `rescue = RenewCard` on the charge's line. It gives the rescue step no name of its own: the executor names it after the step it rescues, `charge_rescue`. A small reporter prints the attempts, the rescue and how each journey ended. It sees the two new decisions as `EventBody::StepRescuing`, with the failure's `cause` and the rescue step's name under `rescue`, and `EventBody::StepRescued`.

```example
name: ch14_rescue
region: workflow
```

Two orders go through, both with an expired card. Ann's bank has issued her a new card; Ben's hasn't.

```example
name: ch14_rescue
region: run
```

When it runs, Ann's journey reports, in order:

1. the charge's attempt 1 starts, and fails with `expired-card`;
2. `step_rescuing`: the charge will be rescued by `charge_rescue`, after a failure;
3. `charge_rescue` starts its attempt 1 and succeeds, and its `card` is committed, replacing the expired one;
4. `step_rescued`: the repair worked, and the charge will run again;
5. the charge's attempt 2 starts with the new card and succeeds, and `charged` is committed;
6. the ship step runs, and the journey succeeds, with the new card and the amount charged.

Ben's journey starts the same way, up to `step_rescuing`. Then `charge_rescue` fails with `nothing-to-renew` and is given up with the cause `failure`, and the journey fails at `charge_rescue`, with nothing charged. The charge never runs again.

## How it works

1. **A rescue step repairs, then hands back.** When the charge fails, whether it reported a failure, retriable or not, or ended in an abnormal termination, the executor hands the failure to its rescue step instead of giving the charge up. The charge's status becomes `Rescuing`: its last attempt failed, and the repair hasn't succeeded yet. The rescue step runs next, then the charge once more. The charge's second run is its attempt 2, built afresh from the data bag, so it sees the card the rescue step contributed. It isn't a retry: the charge has no retry budget to spend.
2. **The rescue step sees why the step failed.** Its fields say what it asks for. Besides its own inputs, marked `#[input]` and read from the data bag as on any step, it can ask for what the failed attempt left: its reason, with `#[reason]`, if it reported a failure; its error, with `#[error]`, if it ended in an abnormal termination, which has no reason; and what it contributed before failing, which was never committed, with `#[from_rescued_step]`.
3. **Each of those is required or optional, by its type.** As for an input, a field of type `Option<T>` is optional, and a field of any other type is required: a required one that doesn't exist aborts the journey with `required data missing`. `RenewCard`'s reason is an `Option<Reason>`, which is `None` after an abnormal termination, so an abnormal termination doesn't abort the journey, and the rescue step repairs only what it knows how to repair. In the same way, an error asked for as `Option<Error>` is `None` after a failure.
4. **The rescue step is an ordinary step otherwise.** It is built once, runs once, contributes data and reports an outcome: its `run` returns `Result<RescueOutcome, Error>`, with the shorthands `Self::succeed()`, `Self::fail(...)` and `Self::fail_retriably(...)`. Its contributions are committed only when it succeeds, as for any step. It has nothing else in the workflow: no name you choose, no policies, no input adapter and no retry budget.
5. **A rescued step is never retried.** The rescue takes the place of retrying, so a step with a rescue step has no retry budget, and a step is rescued at most once in a journey. Neither the step's first run, nor the rescue step, nor the second run is tried again. Adding `retry_budget` to a line that has `rescue` doesn't compile; a workflow declared by hand, as [chapter 28](28-declaring-by-hand.md) shows, is refused before any journey with the violation `rescued step retried` instead.
6. **If the rescue fails, the journey fails.** The rescue step is a step that failed: it is given up, and the journey fails, naming the rescue step with its cause and reason. An `Err` from its `run` is an abnormal termination of the rescue step, which fails the journey in the same way. No step hook is called, and the charge stays `Rescuing` and never runs again. If the rescue succeeds but the second run of the charge fails, the charge is given up as any step would be, its `on step failure` hook is called, and the journey fails too.
7. **A rescue step can't skip.** Skipping would leave the failed step neither repaired nor given up, so the specification aborts a journey whose rescue step skips, with `rescue skipped`. In Rust you can't even write one: `RescueOutcome` has no skipped outcome, and the rescue step has no shorthand to skip.
8. **The step's policies still have their say.** After the first failure, the `on step retry` and `on step failure` hooks aren't called, since the step is neither retried nor given up: its failure goes to the rescue. An abnormal termination still calls `on step abnormal termination` first, which may fail the journey instead of letting the rescue run; [chapter 16](16-ending-a-journey-early.md) shows it.
9. **The events show the rescue.** After the charge's failed attempt comes the decision `step_rescuing`, carrying the cause of the failure (`failure`, `retriable failure` or `abnormal termination`) and the rescue step's name. The rescue step's own events follow, under the name `charge_rescue` and as its attempt 1, with its outcome and the contributions it committed. When it succeeded, the decision `step_rescued` records that the charge will run again, and the charge's attempt 2 begins.

## Remember

1. A rescue step repairs what made a step fail; the step itself never repairs its own failure.
2. A step with a rescue step has no retry budget, and is rescued at most once in a journey.
3. If the rescue step fails, the journey fails, and the rescued step never runs again.
4. A rescue step never skips: a `RescueOutcome` can't say it, and the specification aborts a journey whose rescue step does.
5. The rescue step's contributions reach the rescued step only when the rescue succeeds.
