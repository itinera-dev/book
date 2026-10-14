# Stopping a journey when its log fails

## What you want to do

The shop's log now writes each line to a logging service, over the network, and a write can fail. You want to know what happens to a journey whose log can't take a line, so that no order goes on with no record of what it did.

## The code

The shop is [chapter 25](25-reading-the-result.md)'s, with its log changed. The log turns each event into a line, as before, and hands it to the logging service. The faked service can't take the line saying that the charge of order A-15 succeeded, and its error becomes the log's: `?` returns it from `report`, so the report fails:

```example
name: ch31_stopping_a_journey_when_its_log_fails
region: log
asynchronous: ch31_stopping_a_journey_when_its_log_fails_async
```

In the asynchronous form, the service's `write` awaits the network, so the log implements `AsyncReporter`, whose `report` is an `async fn`, and the workflow names it under `async_reporters` in place of `reporters`, where the desk stays:

```example
name: ch31_stopping_a_journey_when_its_log_fails
region: reporters
asynchronous: ch31_stopping_a_journey_when_its_log_fails_async
```

`what_went_wrong`, from chapter 25, now also names the error of a reporter that failed:

```example
name: ch31_stopping_a_journey_when_its_log_fails
region: explain
```

`main` runs two orders: Ann's, and the order of a new customer, Pat, who pays 50 for a parcel of 3 kg:

```example
name: ch31_stopping_a_journey_when_its_log_fails
region: orders
asynchronous: ch31_stopping_a_journey_when_its_log_fails_async
```

It prints:

```output
name: ch31_stopping_a_journey_when_its_log_fails
asynchronous: ch31_stopping_a_journey_when_its_log_fails_async
```

## How it works

1. **A reporter that fails aborts the journey.** The log's `report` returned an error on Pat's `step_succeeded`, so the executor aborted the journey, with the reason `reporter failed`. A reporter can't report an error and carry on, and a journey with no record of what it did is not one the shop can trust, so it doesn't go on.
2. **The abort is recorded at once.** Nothing after it runs: the shipping never starts, so no carrier books Pat's parcel, and no workflow hook runs, so Pat is not told, as [chapter 25](25-reading-the-result.md) showed of an aborted journey.
3. **What the journey did stays done.** The bank took Pat's 50 before the abort. No hook of `failed_shipping` is called, since no shipping was given up, so nothing gives the money back. An abort is a fault to fix, as chapter 25 said: here, the shop finds Pat's payment with the bank, and refunds it before running the order again, since a new journey would charge Pat again.
4. **The abort carries the reporter's error.** `Abort::ReporterFailed` holds the error the log returned, which is how `what_went_wrong` prints the service's own words.
5. **Reporters should be written so that they never fail.** A reporter's failure costs the whole journey, at whatever point it happens. A log that can lose its service should keep its lines, and send them once the service is back, rather than return the error.

Pat's journey stopped before it did anything it couldn't record. But no record holds Pat's last two events, the charge's success and the abort: the log was the only reporter that printed events, and it was the one that failed.

## Outcomes

We have shown how a journey is aborted when one of its reporters fails, and what it leaves behind.

| The usual way | With Itinera |
|---|---|
| A log call that fails is often ignored, or crashes the request at whatever line made it. | A reporter that fails aborts the journey with `reporter failed`, at once, and the result carries its error. |
| What a request did before it crashed is found by reading the code that ran. | The events before the abort say what the journey did, up to the charge, and the shipping never started. |
