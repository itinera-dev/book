# Repairing, then trying once more

## What you want to do

Cards expire. When the charge fails because the customer's card has expired, trying the same card again is pointless, but the customer's bank usually knows the card that replaced it. You want the shop to ask the bank for the new card, then charge once more, without the charge step knowing anything about renewing cards.

## The code

The charge step charges the card it is given. An expired card is a failure, with the reason `expired-card`, and nothing more: the step doesn't try to fix it. A charge the bank refuses, here because it is above the bank's limit of 1,000, is a failure too, with the reason `declined`.

```example
name: ch14_rescue
region: charge
```

The repair is a step of its own, `RenewCard`: the charge's rescue step, which runs when the charge fails, to fix what made it fail. It is a struct that derives `RescueStep` rather than `Step`, and its `run` is in a block marked `#[rescue_step]`. It reads why the charge failed, through its field marked `#[reason]`, and the card, as an input. When the card had expired, it asks the bank for the card that replaced it and contributes it under `card`, the key the charge reads. Otherwise there is nothing it can renew, and it fails with `nothing-to-renew`: when the charge failed for another reason, when it ended in an abnormal termination, which gives no reason at all, or when the bank has no new card.

```example
name: ch14_rescue
region: rescue
```

The workflow attaches the rescue step to the charge, with `rescue = RenewCard` on the charge's line. It gives the rescue step no name of its own: the executor names it after the step it rescues, `charge_rescue`. The workflow also lists `DebugReporter`, which prints every event, so that you can watch the rescue happen.

```example
name: ch14_rescue
region: workflow
```

The program runs two orders, both with a card that expired before this year: Ben's, A-2, and Dan's, A-4. The bank has a new card for each, but Dan's order costs 1,200, more than the bank's limit, so it refuses his new card too.

```example
name: ch14_rescue
region: run
```

It prints:

```output
name: ch14_rescue
```

Ben's journey reports, in order:

1. the charge's attempt 1 starts, and fails with `expired-card`;
2. `StepRescuing`: the charge will be rescued by `charge_rescue`, after a failure;
3. `charge_rescue` starts its attempt 1 and succeeds, and its `card` is committed, and `DataOverwritten` records that it replaced the expired one;
4. `StepRescued`: the repair worked, and the charge will be attempted again;
5. the charge's attempt 2 starts with the new card and succeeds, and `charged` is committed;
6. the ship step runs, and the journey succeeds, with the new card and the amount charged.

Dan's journey starts the same way, up to the charge's attempt 2. That attempt fails with `declined`, and since a step is rescued at most once, the charge is given up with the cause `failure`, and the journey fails at `charge`, with nothing charged. Had the charge failed for any reason but an expired card, `charge_rescue` would have failed with `nothing-to-renew`, and the journey would have failed at `charge_rescue` instead. [Chapter 25](25-step-rescue-step-or-policy.md) shows it, once its charge has a rescue step.

## How it works

1. **A rescue step repairs, then hands back.** When the charge fails, whether it reported a failure, retriable or not, or ended in an abnormal termination, the executor hands the failure to its rescue step instead of giving the charge up. The charge's status becomes `Rescuing`: its last attempt failed, and the repair hasn't succeeded yet. The rescue step runs next, then the charge once more. The charge's attempt 2 is built afresh from the data bag, so it sees the card the rescue step contributed. It isn't a retry: the charge has no retry budget to spend.
2. **The rescue step sees why the step failed.** Its fields say what it asks for. Besides its own inputs, marked `#[input]` and resolved as on any step, it can ask for what the failed attempt left: its reason, with `#[reason]`, if it reported a failure; its error, with `#[error]`, if it ended in an abnormal termination, which has no reason; and what it contributed before failing, which was never committed, with `#[from_rescued_step]`.
3. **Each of those is required or optional, by its type.** As for an input, a field of type `Option<T>` is optional, and a field of any other type is required: a required one that doesn't exist aborts the journey with `required data missing`. `RenewCard`'s reason is an `Option<Reason>`, which is `None` after an abnormal termination, so an abnormal termination doesn't abort the journey, and the rescue step repairs only what it knows how to repair. In the same way, an error asked for as `Option<Error>` is `None` after a failure.
4. **The rescue step is an ordinary step otherwise.** It is built once, runs once, contributes data and reports an outcome: its `run` returns `Result<RescueOutcome, Error>`, with the shorthands `Self::succeed()`, `Self::fail(...)` and `Self::fail_retriably(...)`. Its contributions are committed only when it succeeds, as for any step. It has no name you choose, no policies and no retry budget.
5. **It inherits nothing from the step it rescues.** Like any step, a rescue step may have services, and an input adapter, which [chapter 19](19-deciding-where-a-steps-data-comes-from.md) teaches, and they are attached to its own name, `charge_rescue`, as if it were a step the workflow lists. A provider marked `#[provider(steps = ["charge", "charge_rescue"])]` serves both the charge and its rescue step; one attached to `charge` alone serves the charge only, never its rescue step; and one attached to no step serves every step, rescue steps included. Attaching anything to `charge_rescue` when the charge has no rescue step is a mistake caught before any journey, as [chapter 27](27-mistakes-caught-before-a-journey.md) shows. The workflow's listing shows the rescue step in the charge's entry, with its services.
6. **A rescued step is never retried.** The rescue takes the place of retrying, so a step with a rescue step has no retry budget, and a step is rescued at most once in a journey. Neither the step's first attempt, nor the rescue step, nor the attempt after the rescue is tried again. Adding a `retry_budget` above 0 to a line that has `rescue` doesn't compile; a workflow declared by hand, as [chapter 28](28-declaring-by-hand.md) shows, is refused before any journey with the violation `rescued step retried` instead.
7. **If the rescue fails, the journey fails.** The rescue step is a step that failed: it is given up, and the journey fails, naming the rescue step with its cause and reason. An `Err` from its `run` is an abnormal termination of the rescue step, which fails the journey in the same way. No step hook is called, and the charge stays `Rescuing` and never runs again. If the rescue succeeds but the charge's attempt 2 fails, the charge is given up as any step would be, and the journey fails too.
8. **A rescue step can't skip.** Skipping would leave the failed step neither repaired nor given up, so the specification aborts a journey whose rescue step skips, with `rescue skipped`. In Rust you can't even write one: `RescueOutcome` has no skipped outcome, and the rescue step has no shorthand to skip.
9. **Policies see the rescue too.** What the hooks of a step's policies can do around a rescue, such as failing the journey before the rescue runs, is in [chapter 16](16-ending-a-journey-early.md).
10. **The events show the rescue.** After the charge's failed attempt comes the decision `StepRescuing`, carrying the cause of the failure (`failure`, `retriable failure` or `abnormal termination`) and the rescue step's name. The rescue step's own events follow, under the name `charge_rescue` and as its attempt 1, with its outcome and the contributions it committed. When it succeeded, the decision `StepRescued` records that the charge will run again, and the charge's attempt 2 begins.

## Remember

1. A rescue step repairs what made a step fail; the step itself never repairs its own failure.
2. A step with a rescue step has no retry budget, and is rescued at most once in a journey.
3. If the rescue step fails, the journey fails, and the rescued step never runs again.
4. A rescue step never skips: a `RescueOutcome` can't say it, and the specification aborts a journey whose rescue step does.
5. The rescue step's contributions reach the rescued step only when the rescue succeeds.
6. A rescue step inherits nothing from the step it rescues: what it needs is attached to its own name, such as `charge_rescue`.
