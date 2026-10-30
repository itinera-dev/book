# Stopping a journey when its log fails

## What you want to do

The shop's log now writes each line to a logging service, over the network, and a write can fail. You want to know what happens to a journey whose log can't take a line, so that no order goes on with no record of what it did.

## The code

The shop is [chapter 25](25-reading-the-result.md)'s, with its log changed. The log turns each event into a line, as before, and hands it to the logging service. The faked service can't take the line saying that the charge of order A-15 succeeded, and throws `LoggingDown`. The log doesn't catch it, so it escapes `report`:

```example
name: ch31_stopping_a_journey_when_its_log_fails
region: log
asynchronous: ch31_stopping_a_journey_when_its_log_fails_async
```

`detail` is [chapter 22](22-finishing-a-journey-early.md)'s, with the parts that page points to. The workflow still lists the log and the desk among its reporters, as [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) shows, the desk is [chapter 11](11-naming-a-journey.md)'s and `Notify` is [chapter 19](19-letting-policies-act-on-the-outside-world.md)'s.

In the asynchronous form, the service's `write` is a `suspend` function that waits for [chapter 26](26-running-async-journeys-while-others-wait.md)'s `network` before it answers, and the log awaits it in its `report`, as an `AsyncReporter`, as chapter 26 made it.

`whatWentWrong`, from [chapter 25](25-reading-the-result.md), now also names the error of a reporter that failed:

```example
name: ch31_stopping_a_journey_when_its_log_fails
region: explain
```

`main` runs two orders: Ann's, and the order of a new customer, Pat, who pays 50 for a parcel of 3 kg:

```example
name: ch31_stopping_a_journey_when_its_log_fails
region: orders
```

It runs them one after the other with one executor, as [chapter 25](25-reading-the-result.md)'s `main` does, and hands each result to that chapter's `followUp`. `NewOrder` is chapter 25's, and `Parcel` [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s. In the asynchronous form, `main` runs both at once, with [chapter 26](26-running-async-journeys-while-others-wait.md)'s `journey`, and hands each result to `followUp` once both have ended:

```example
name: ch31_stopping_a_journey_when_its_log_fails_async
region: run
```

It prints:

```output
name: ch31_stopping_a_journey_when_its_log_fails
asynchronous: ch31_stopping_a_journey_when_its_log_fails_async
```

## How it works

1. **A reporter that fails aborts the journey.** `LoggingDown` escaped the log's `report` on Pat's `step_succeeded`, so the executor aborted the journey, with the reason `reporter failed`. A reporter can't report an error and carry on, and a journey with no record of what it did is not one the shop can trust, so it doesn't go on.
2. **The abort is recorded at once.** Nothing after it runs: the shipping never starts, so no carrier books Pat's parcel, and no workflow hook runs, so Pat is not told, as [chapter 25](25-reading-the-result.md) showed of an aborted journey.
3. **What the journey did stays done.** The bank took Pat's 50 before the abort. No hook of `FailedShipping`, from [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md), is called, since no shipping was given up, so nothing gives the money back. An abort is a fault to fix, as [chapter 25](25-reading-the-result.md) said: here, the shop finds Pat's payment with the bank, and refunds it before running the order again, since a new journey would charge Pat again.
4. **The abort carries the reporter's error.** For this reason, the abort's `details` is a `ReporterFailed`, from `org.itineradev.itinera.event`, and `abort.errorMessage` is the message of what the log threw, which is how `whatWentWrong` prints the service's own words. `abort.error` holds the `LoggingDown` itself, for code that needs more than its message.
5. **Reporters should be written so that they never fail.** A reporter's failure costs the whole journey, at whatever point it happens. A log that can lose its service should keep its lines, and send them once the service is back, rather than let the exception out.

Pat's journey stopped before it did anything it couldn't record. But no record holds Pat's last two events, the charge's success and the abort: the log was the only reporter that printed events, and it was the one that failed.

## Outcomes

We have shown how a journey is aborted when one of its reporters fails, and what it leaves behind.

| The usual way | With Itinera |
|---|---|
| A log call that fails is often ignored, or crashes the request at whatever line made it. | A reporter that fails aborts the journey with `reporter failed`, at once, and the result carries its error. |
| What a request did before it crashed is often found only by reading the code that ran. | The events before the abort say what the journey did, up to the charge, and the shipping never started. |
