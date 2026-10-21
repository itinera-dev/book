# Stopping a journey when its log fails

## What you want to do

The shop's log now writes each line to a logging service, over the network, and a write can fail. You want to know what happens to a journey whose log can't take a line, so that no order goes on with no record of what it did.

## The code

The shop is [chapter 26](26-running-async-journeys-while-others-wait.md)'s, with its log changed. The log turns each event into a line, as before, and hands it to the logging service, which answers over the network, so `report` is now `async` and awaits it. The faked service can't take the line saying that the charge of order A-15 succeeded, and throws `LoggingDown`. The log doesn't catch it, so the promise `report` returns rejects with it:

```example
name: ch31_stopping_a_journey_when_its_log_fails
region: log
```

`network` is [chapter 26](26-running-async-journeys-while-others-wait.md)'s, and `detail` is the function [chapter 12](12-trying-again.md) shows, with the cases later chapters added to it, the last in [chapter 22](22-finishing-a-journey-early.md). The workflow still lists the log and the desk among its reporters, as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) shows: a reporter's `report` may return nothing or a promise of nothing, and the executor waits for whichever it gets, as it does for a step's `run`. The desk is [chapter 11](11-naming-a-journey.md)'s.

`whatWentWrong`, from [chapter 25](25-reading-the-result.md), now also names the error of a reporter that failed:

```example
name: ch31_stopping_a_journey_when_its_log_fails
region: explain
```

The program runs two orders one after the other, with `shop` and `executor` made as [chapter 24](24-running-many-journeys.md) makes them, and hands each result to [chapter 25](25-reading-the-result.md)'s `followUp`: Ann's, and the order of a new customer, Pat, who pays 50 for a parcel of 3 kg:

```example
name: ch31_stopping_a_journey_when_its_log_fails
region: orders
```

It prints:

```output
name: ch31_stopping_a_journey_when_its_log_fails
```

## How it works

1. **A reporter that fails aborts the journey.** The log's `report` rejected on Pat's `step_succeeded`, so the executor aborted the journey, with the reason `reporter failed`. A throw from a reporter, or a promise it returns that rejects, is the same failure. A reporter can't report an error and carry on, and a journey with no record of what it did is not one the shop can trust, so it doesn't go on.
2. **The abort is recorded at once.** Nothing after it runs: the shipping never starts, so no carrier books Pat's parcel, and no workflow hook runs, so Pat is not told, as [chapter 25](25-reading-the-result.md) showed of an aborted journey.
3. **What the journey did stays done.** The bank took Pat's 50 before the abort. No hook of `failedShipping`, as [chapter 26](26-running-async-journeys-while-others-wait.md) shows it, is called, since no shipping was given up, so nothing gives the money back. An abort is a fault to fix, as [chapter 25](25-reading-the-result.md) said: here, the shop finds Pat's payment with the bank, and refunds it before running the order again, since a new journey would charge Pat again.
4. **The abort carries the reporter's error.** Once `whatWentWrong` has compared `abort.reason` with `reporter failed`, TypeScript lets it read `abort.errorMessage`, the message of what the log threw, which is how it prints the service's own words. `abort.error` holds what was thrown itself, the `LoggingDown`, for code that needs more than its message.
5. **Reporters should be written so that they never fail.** A reporter's failure costs the whole journey, at whatever point it happens. A log that can lose its service should keep its lines, and send them once the service is back, rather than let the error out.

Pat's journey stopped before it did anything it couldn't record. But no record holds Pat's last two events, the charge's success and the abort: the log was the only reporter that printed events, and it was the one that failed.

## Outcomes

We have shown how a journey is aborted when one of its reporters fails, and what it leaves behind.

| The usual way | With Itinera |
|---|---|
| A log call that fails is often ignored, or crashes the request at whatever line made it. | A reporter that fails aborts the journey with `reporter failed`, at once, and the result carries its error. |
| What a request did before it crashed is found by reading the code that ran. | The events before the abort say what the journey did, up to the charge, and the shipping never started. |
