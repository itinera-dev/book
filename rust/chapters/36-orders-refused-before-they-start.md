# Orders refused before they start

## What you want to do

The website now sends orders before they are finished: Quin's arrives without an order number, so nothing can name its journey, and order A-16 without the customer's name, so the logging service has nothing to file its lines under. You want such an order stopped before anything is done for it, and to know why.

## The code

The shop is [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s, with the order's number now optional in its initial data, since the website may leave it out. When the log receives the customer's name, it asks the logging service to file the journey's lines under it, and fails if the service refuses:

```example
name: ch36_orders_refused_before_they_start
region: log
asynchronous: ch36_orders_refused_before_they_start_async
```

The rest of the log is [chapter 33](33-giving-every-reporter-what-it-needs.md)'s.

`main` creates each order's instance, then runs it, and says why an order went no further at either point:

```example
name: ch36_orders_refused_before_they_start
region: run
asynchronous: ch36_orders_refused_before_they_start_async
```

`Orders` is the workflow [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) declares, with [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s provider for `cards`, and `Parcel` is [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s. In the asynchronous form, `order` creates the instance, as in [chapter 33](33-giving-every-reporter-what-it-needs.md), and `journey` runs it, awaited, as in [chapter 26](26-running-async-journeys-while-others-wait.md). It prints:

```output
name: ch36_orders_refused_before_they_start
asynchronous: ch36_orders_refused_before_they_start_async
```

## How it works

1. **An instance isn't created without its journey ID.** The ID generator of [chapter 11](11-naming-a-journey.md) names each journey after its order's number, read from the initial data with `required`. Quin's order has no number, so the generator returns an error, and `create` returns it in turn: `the journey ID could not be produced`. With no instance, there is nothing to run.
2. **An executor can refuse an instance.** `run` returns a refusal for a journey that can't start, as [chapter 5](05-a-first-journey.md) said. Before the first event, the executor gives every reporter the journey ID, then the instance's entries of global metadata, as [chapter 33](33-giving-every-reporter-what-it-needs.md) showed. The log failed on `customer`, so the executor refused the journey: the refusal says the global metadata could not be set, and carries the log's error.
3. **A refused journey never started.** There is no `journey_started`, no step ran, and the bank was never called: unlike Pat's journey in [chapter 31](31-stopping-a-journey-when-its-log-fails.md), aborted after the money was taken, A-16's has nothing to undo. There is no `JourneyResult` either, since there was no journey to end.
4. **The order is still the shop's to deal with.** `main` learns why from the error and the refusal, as values. Once the website sends what was missing, a new instance runs the order, as [chapter 24](24-running-many-journeys.md) said.

These orders were stopped by what they lacked. A mistake in how the shop's workflow is put together, such as two steps under one name, would stop every order instead, however complete.

## Outcomes

We have shown how an order that can't be named or logged is stopped before its journey starts, and how the program learns why.

| The usual way | With Itinera |
|---|---|
| An incomplete request often gets halfway through, past the payment, before something notices the missing field. | An order that can't be named, or whose journey a reporter can't take, is stopped before its first step. |
| Why a request never ran is often only a log line somewhere. | `create` and `run` return why, as values the program reads. |
