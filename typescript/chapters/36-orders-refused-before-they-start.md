# Orders refused before they start

## What you want to do

The website now sends orders before they are finished: Quin's arrives without an order number, so nothing can name its journey, and order A-16 without the customer's name, so the logging service has nothing to file its lines under. You want such an order stopped before anything is done for it, and to know why.

## The code

The shop is [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s. The website leaves `order` out of Quin's data, and sends order A-16 with an empty customer's name:

```example
name: ch36_orders_refused_before_they_start
region: data
```

When the log receives the customer's name, it asks the logging service to file the journey's lines under it, and fails if the service refuses. Filing waits for the service's answer over the network, so `receiveGlobalMetadata` is now `async`, as `report` became in [chapter 31](31-stopping-a-journey-when-its-log-fails.md):

```example
name: ch36_orders_refused_before_they_start
region: log
```

The rest of the log is [chapter 33](33-giving-every-reporter-what-it-needs.md)'s, with [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s `detail`. The faked logging service refuses an empty name with its error, `NoCustomer`.

For each order, the program prints a heading, then creates the order's instance, then runs it, and says why the order went no further at either point:

```example
name: ch36_orders_refused_before_they_start
region: run
```

`shop` and `executor` are made as in [chapter 24](24-running-many-journeys.md), `JourneyResult` is the type of [chapter 25](25-reading-the-result.md), `followUp` is [chapter 34](34-saying-which-attempt-took-the-money.md)'s, and `WorkflowInstance`, from `itinera/instance`, is the type of an instance, as in [chapter 26](26-running-async-journeys-while-others-wait.md). It prints:

```output
name: ch36_orders_refused_before_they_start
```

## How it works

1. **An instance isn't created without its journey ID.** The ID generator of [chapter 11](11-naming-a-journey.md) names each journey after its order's number, read from the initial data with `expect`. Quin's order has no number, so `expect` throws an `Error` with the message the generator gave it, `the order's number`, and `instance(...)` throws it in turn, as it was thrown. With no instance, there is nothing to run. The program checks that what it caught is an `Error` before it reads its `message`, as the outage policy does in [chapter 34](34-saying-which-attempt-took-the-money.md).
2. **An executor can refuse an instance.** `run` rejects with a `Refusal` for a journey that can't start, as [chapter 24](24-running-many-journeys.md) showed. Before the first event, the executor gives every reporter the journey ID, then the instance's entries of global metadata, as [chapter 33](33-giving-every-reporter-what-it-needs.md) showed. The log failed on `customer`, so the executor refused the journey: the refusal's `refused.kind` is `global metadata not set`, its message ends with the message of the log's error, and `refused.error` holds the error itself.
3. **A refused journey never started.** There is no `journey_started`, no step ran, and the bank was never called: unlike Pat's journey in [chapter 31](31-stopping-a-journey-when-its-log-fails.md), aborted after the money was taken, A-16's has nothing to undo. There is no `JourneyResult` either, since there was no journey to end.
4. **The order is still the shop's to deal with.** The program learns why from what `instance(...)` threw and what `run` rejected with. Once the website sends what was missing, a new instance runs the order, as [chapter 24](24-running-many-journeys.md) said.

These orders were stopped by what they lacked. A mistake in how the shop's workflow is put together, such as two steps under one name, would stop every order instead, however complete.

## Outcomes

We have shown how an order that can't be named or logged is stopped before its journey starts, and how the program learns why.

| The usual way | With Itinera |
|---|---|
| An incomplete request often gets halfway through, past the payment, before something notices the missing field. | An order that can't be named, or whose journey a reporter can't take, is stopped before its first step. |
| Why a request never ran is often only a log line somewhere. | `instance(...)` throws, and `run` rejects with a `Refusal`, saying why, to the program that called them. |
