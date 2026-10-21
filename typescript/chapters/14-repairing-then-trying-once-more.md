# Repairing, then trying once more

## What you want to do

Jo's card has expired, so charging it again is pointless, but Jo's bank knows the card that replaced it. When the charge fails, you want the shop to ask the bank for the new card, then charge once more, without the charge knowing anything about renewing cards.

## The code

The payment client can now also say which card replaced one. The faked bank says that an expired card was replaced by its holder's card that expires in three years, and prints it. Its error, `NetworkDown`, is [chapter 12](12-trying-again.md)'s, and the services' tokens, `PAYMENTS` and `CARRIER`, [chapter 7](07-giving-a-step-the-tools-it-needs.md)'s.

```example
name: ch14_repairing_then_trying_once_more
region: payments
```

The repair is a **rescue step**: a class extending `RescueStep`, marked `@rescueStep()`, whose `run` returns a `RescueOutcome`, all three from `itinera/step`. Its constructor asks for the order's card as an input, and for the reason the charge failed with a parameter marked `@failureReason()` and typed `Reason`, both from `itinera/step` too. Its `run` asks the bank for the card's replacement, and contributes it as the order's new `card`.

```example
name: ch14_repairing_then_trying_once_more
region: rescue
```

The workflow names it in the charge's options, with `rescue: RenewCard`, in place of the charge's `retryBudget` and `noRetryOnAbnormalTermination`: a step with a rescue step has no retry budget, and is rescued after a failed attempt whatever that setting says. The bank's provider is [chapter 12](12-trying-again.md)'s, and the carriers' provider, one carrier for each attempt, [chapter 13](13-a-carrier-for-each-attempt.md)'s.

```example
name: ch14_repairing_then_trying_once_more
region: workflow
```

The charge and the shipping are [chapter 12](12-trying-again.md)'s, and the desk is [chapter 11](11-naming-a-journey.md)'s.

The log is [chapter 11](11-naming-a-journey.md)'s, with the `detail` function [chapter 12](12-trying-again.md) shows, whose `switch` now also says what the two new decisions are about, and names the data a contribution replaced:

```example
name: ch14_repairing_then_trying_once_more
region: rescue_events
```

The shop runs Ann's, Gil's and Jo's orders again, and prints:

```output
name: ch14_repairing_then_trying_once_more
```

## How it works

1. **A rescue step repairs what made a step fail.** When the charge's attempt fails, the executor doesn't give the charge up: it decides to rescue it, recorded as `step_rescuing`, with the cause of the failure, here `failure`, and the rescue step's name. The rescue step is named after the step it rescues, `charge_rescue`, and runs next. **A step never repairs its own failure: its rescue step does, and the workflow decides which rescue step a step has.**
2. **It knows why the step failed.** Besides inputs it asks for as any step does, such as the card, a rescue step can ask for what the failed attempt reported: its reason, with `@failureReason()`. `RenewCard` reads its `code` to check that the card expired, the one failure it knows how to repair, before it asks the bank.
3. **It is a step otherwise.** It runs once, with its own attempt, asks for services and contributes data, and says how it went with `this.succeed()` or `this.fail(...)`, as any step does. The bank's provider serves every step that asks for `payments`, the rescue step included, with the bank already made for Jo's journey, which is why the rescue step's attempt has no `service_provided` of its own.
4. **Its contributions are committed only when it succeeds.** The rescue step succeeded, so its `card` was committed, replacing the card Jo's order started with. The executor records that as `data_overwritten`, after the commit, naming the key and never the value: the data bag now holds `jo-2029`, and only the bank's printed line still names the old card.
5. **Then the rescued step runs once more.** Once the rescue step has succeeded, the executor records `step_rescued`, and starts the charge's attempt 2, built afresh from the data bag, so it reads the new card, and the bank charges it. **A step with a rescue step has no retry budget, and can't be given one:** running it once more is part of the rescue, not a retry.
6. **Only a step that fails is rescued.** Ann's and Gil's charges succeed, so their journeys never run `charge_rescue`, and their logs are the same as in chapter 13.

Jo's order now succeeds, and so do Ann's and Gil's. But the shop's accountants want a line in their audit for every charge that succeeded, with what it took, and nothing in the shop writes one. The charge could, but then the charge would be about auditing as well as charging.

## Outcomes

We have shown how a rescue step repairs a step that failed, so that the step can run once more.

| The usual way | With Itinera |
|---|---|
| Repairing a failure, such as renewing an expired card, is usually written inside the code that failed, in a branch of its error handling. | The repair is a step of its own, named on the failing step's line in the workflow, and the step it repairs never knows about it. |
| After a repair, it is often hard to tell from the code which call ran again and with which data. | The log shows the rescue, what it replaced in the data bag, and the rescued step's next attempt, in order. |
