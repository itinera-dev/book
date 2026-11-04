# Orders refused before they start

## What you want to do

The website now sends orders before they are finished: Quin's arrives without an order number, so nothing can name its journey, and order A-16 without the customer's name, so the logging service has nothing to file its lines under. You want such an order stopped before anything is done for it, and to know why.

## The code

The shop is [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s, with the order's number now an `Optional<String>` in its initial data, since the website may leave it out. An empty number adds no key, as an empty discount doesn't in [chapter 20](20-an-optional-input.md):

```example
name: ch36_orders_refused_before_they_start
region: data
```

`Parcel` is [chapter 21](21-deciding-where-a-steps-data-comes-from.md)'s. When the log receives the customer's name, it asks the logging service to file the journey's lines under it, and lets the service's exception out if it refuses:

```example
name: ch36_orders_refused_before_they_start
region: log
```

The faked logging service refuses an empty name with its exception, `NoCustomer`. The rest of the log is [chapter 33](33-giving-every-reporter-what-it-needs.md)'s, with [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s `detail`.

For each order, `main` first prints a heading, saying what the order lacks:

```example
name: ch36_orders_refused_before_they_start
region: heading
```

Then it creates the order's instance, runs it, and says why the order went no further at either point. The instance is a `WorkflowInstance<Sync>`, from `org.itineradev.itinera.instance` and `org.itineradev.itinera.workflow`, declared before the `try` so that the run after it can use it:

```example
name: ch36_orders_refused_before_they_start
region: run
```

`Orders` is the workflow [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md) declares, with [chapter 35](35-giving-the-rescue-step-a-service-of-its-own.md)'s provider for `cards`, `executor` is made before the loop, as in [chapter 24](24-running-many-journeys.md), and `followUp` is [chapter 34](34-saying-which-attempt-took-the-money.md)'s. It prints:

```output
name: ch36_orders_refused_before_they_start
```

## How it works

1. **An instance isn't created without its journey ID.** The ID generator of [chapter 11](11-naming-a-journey.md) names each journey after its order's number, and throws an `IllegalStateException` when there is none. Quin's order has no number, so `create()` throws one in turn, whose message says that the ID generator threw, followed by the generator's own message. With no instance, there is nothing to run.
2. **An executor can refuse an instance.** `run` throws a `Refusal` for a journey that can't start, as [chapter 24](24-running-many-journeys.md) showed. Before the first event, the executor gives every reporter the journey ID, then the instance's entries of global metadata, as [chapter 33](33-giving-every-reporter-what-it-needs.md) showed. The log threw on `customer`, so the executor refused the journey: the refusal's `getRefused()` is a `Refused.GlobalMetadataNotSet`, its message ends with the message of the log's exception, and `getRefused().getError()` holds the `NoCustomer` itself.
3. **A refused journey never started.** There is no `journey_started`, no step ran, and the bank was never called: unlike Pat's journey in [chapter 31](31-stopping-a-journey-when-its-log-fails.md), aborted after the money was taken, A-16's has nothing to undo. There is no `JourneyResult` either, since there was no journey to end.
4. **The order is still the shop's to deal with.** `main` learns why from what `create()` and `run` threw. Once the website sends what was missing, a new instance runs the order, as [chapter 24](24-running-many-journeys.md) said.

These orders were stopped by what they lacked. A mistake in how the shop's workflow is put together, such as two steps under one name, would stop every order instead, however complete.

## Outcomes

We have shown how an order that can't be named or logged is stopped before its journey starts, and how the program learns why.

| The usual way | With Itinera |
|---|---|
| An incomplete request often gets halfway through, past the payment, before something notices the missing field. | An order that can't be named, or whose journey a reporter can't take, is stopped before its first step. |
| Why a request never ran is often only a log line somewhere. | `create()` throws, and `run` throws a `Refusal`, saying why, to the program that called them. |
