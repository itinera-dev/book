# Repairing, then trying once more

## What you want to do

Cards expire. When the charge fails because the customer's card has expired, trying the same card again is pointless, but the customer's bank usually knows the card that replaced it. You want the shop to ask the bank for the new card, then charge once more, without the charge step knowing anything about renewing cards.

## The code

The charge step charges the card it is given. An expired card is a failure, with the reason `expired-card`, and nothing more: the step does not try to fix it.

```example
name: ch14_rescue
region: charge
```

The repair is a step of its own, `RenewCard`, the rescue step of the charge. It reads why the charge failed. When the card had expired, it asks the bank for the new one and contributes it under `card`, the key the charge reads. For any other failure it cannot help, and fails; so too when the charge ended in an abnormal termination, which gives no reason at all.

```example
name: ch14_rescue
region: rescue
```

The workflow attaches the rescue step to the charge. It gives the rescue step no name of its own: the executor names it after the step it rescues, `charge_rescue`.

```example
name: ch14_rescue
region: workflow
```

Two orders go through, both with an expired card. Ada's bank has issued her a new card; Ben's has not.

```example
name: ch14_rescue
region: run
```

For Ada, the journey succeeds: the charge fails, the rescue step contributes her new card, the charge runs once more and succeeds, and the order is shipped. For Ben, the rescue step fails with `no-renewed-card`, the journey fails, and the charge never runs again.

```note
name: ch14_rescue
```

## How it works

1. **A rescue step repairs, then hands back.** When the charge fails, whether it reported a failure or ended in an abnormal termination, the executor runs its rescue step next, then the charge once more. The charge's second run is its attempt 2, built afresh from the data bag, so it sees the card the rescue step contributed.
2. **The rescue step sees why the step failed.** Besides its own inputs from the data bag, it can ask for what the failed attempt reported: its reason, if it reported a failure; its error, if it ended in an abnormal termination, which has no reason; and what it contributed before failing. `RenewCard` reads the reason, when there is one, so it repairs only what it knows how to repair.
3. **The rescue step is an ordinary step otherwise.** It is built once, runs once, contributes data and reports an outcome. Its contributions are committed only when it succeeds, as for any step.
4. **A rescued step is never retried.** The rescue takes the place of retrying, so a step with a rescue step has no retry budget: declaring one is refused before any journey starts. Neither the step's first run, nor the rescue step, nor the second run is tried again, and a step is rescued at most once in a journey.
5. **If the rescue fails, the journey fails.** The rescue step is a step that failed, so the journey ends there, naming the rescue step, and the charge does not run again. If the rescue succeeds but the second run of the charge fails, the charge is given up as any step would be, and the journey fails too.
6. **A rescue step cannot skip.** Skipping would leave the failed step neither repaired nor given up, so a rescue step that skips aborts the journey with `rescue skipped`.
7. **The step's policies still have their say.** After the first failure, the `on step retry` and `on step failure` hooks are not called, since the step is neither retried nor given up. An abnormal termination still calls `on step abnormal termination`, which may fail the journey instead of letting the rescue run; chapter 16 shows it.
8. **The events show the rescue.** After the charge's failed attempt, a decision records that the charge will be rescued. The rescue step's own events follow, under the name `charge_rescue`, with its outcome and the contributions it committed. When it succeeded, a decision records that the charge will run again, and its attempt 2 begins.

## Remember

1. A rescue step repairs what made a step fail; the step itself never repairs its own failure.
2. A step with a rescue step has no retry budget, and is rescued at most once in a journey.
3. If the rescue step fails, the journey fails, and the rescued step never runs again.
4. A rescue step never skips: skipping aborts the journey.
5. The rescue step's contributions reach the rescued step only when the rescue succeeds.
