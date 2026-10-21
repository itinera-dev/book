# Saying which attempt took the money

## What you want to do

The accountants match every audit line with the bank's records, but the line says neither which journey nor which attempt took the money, and a charge that ends in the bank's error, which may have taken the money before its network failed, has no line at all. You want every line to say which journey and which attempt it is about, and a line for a charge that ended in an error, with the error.

## The code

The shop is [chapter 33](33-giving-every-reporter-what-it-needs.md)'s. The audit's hook asks for two more things, the journey ID and the attempt number, and writes them on its line:

```example
name: ch34_saying_which_attempt_took_the_money
region: audit
```

The outage policy, which gives up a charge that ended in an error, asks for the same two, the step's name, the amount, and the error itself, and writes an audit line of its own before giving the charge up:

```example
name: ch34_saying_which_attempt_took_the_money
region: outage
```

`followUp`, from [chapter 25](25-reading-the-result.md), now prints the audit line of a failed journey too:

```example
name: ch34_saying_which_attempt_took_the_money
region: follow_up
```

`orderOutput` is [chapter 24](24-running-many-journeys.md)'s, `why` [chapter 25](25-reading-the-result.md)'s and `whatWentWrong` [chapter 31](31-stopping-a-journey-when-its-log-fails.md)'s.

The program runs Ann's order, Jo's, whose expired card is renewed by the rescue step, and Cat's, of 120, on which the bank's network fails, each with its customer's name as global metadata, as [chapter 33](33-giving-every-reporter-what-it-needs.md) gives it. It prints each journey's audit line, even for a journey that failed:

```output
name: ch34_saying_which_attempt_took_the_money
```

## How it works

1. **A hook can ask for the journey ID and the attempt number.** A parameter marked `@journeyId()`, from `itinera/journey`, and typed `JourneyId`, from the same module, gives the hook the journey's ID. One marked `@attempt()` and typed `Attempt`, as the provider of [chapter 13](13-a-carrier-for-each-attempt.md) asks for it, gives it the attempt whose outcome it follows, counted from 1, and `String(attempt)` writes its number. Every step hook can ask for both, and every workflow hook for the journey ID.
2. **The attempt is the one that took the money.** Jo's charge failed on its first attempt, the rescue step renewed the card, and the second attempt succeeded, so Jo's audit line says `attempt 2`. Ann's says `attempt 1`.
3. **A hook called after an error can ask for the error.** A parameter marked `@error()`, from `itinera/step`, gives `on step abnormal termination` what the step's `run` threw: for Cat's charge, the bank's `NetworkDown`, from [chapter 12](12-trying-again.md), whose message, `the payment network is down`, `step_abnormal_termination`, event 13, carries too. TypeScript lets code throw anything, not only an `Error`, so the parameter's type is `unknown`, and the hook checks that it is an `Error` before it reads its `message`. `on step retry` and `on step failure` are called after a failure as well as after an error, and a failure has no error, so they ask for it as `@error({ optional: true })`, which gives an `Option`, as `@failureReason({ optional: true })` does in [chapter 17](17-giving-up-a-step-early.md).
4. **The line is kept though the charge was given up.** The outage's line is a contribution, committed as event 15 though the charge was given up, as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) showed, so Cat's line is in the journey's output, telling the accountants to check whether the bank took 120.

Every charge now has its line, with the journey and the attempt. But look at Jo's journey: the rescue step asks the bank's payment client for the new card, through the same `payments` service as the charge. The bank now answers that question from a card registry of its own, which the rescue step will have to ask instead.

## Outcomes

We have shown how a hook asks for the journey ID, the attempt number and the error, as the executor knows them.

| The usual way | With Itinera |
|---|---|
| An audit trail often records that a payment happened, but not which try of a retried call took the money. | A hook asks for the journey ID and the attempt number, and the executor gives it the ones it recorded. |
| When a call to a payment service ends in an error, code often logs the error and moves on, and nobody checks whether the money was taken. | The hook called after the error asks for it, and writes it where the accountants read. |
