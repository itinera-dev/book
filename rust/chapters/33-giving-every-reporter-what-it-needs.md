# Giving every reporter what it needs

## What you want to do

The logging service files lines by customer, and the shop needs the bank's receipt to refund an order. You want every line of the log to name the customer, and the receipt once the bank has given it, even when the journey stops right after.

## The code

The shop is [chapter 31](31-stopping-a-journey-when-its-log-fails.md)'s, without chapter 32's `DebugReporter`. `main` gives each instance the customer's name with `reporters_global_metadata`, which makes it an entry of the journey's global metadata: data for its reporters, which no step reads:

```example
name: ch33_giving_every_reporter_what_it_needs
region: instance
asynchronous: ch33_giving_every_reporter_what_it_needs_async
```

The charge asks for its step reporter, and sets the receipt as an entry as soon as the bank gives it, before contributing it:

```example
name: ch33_giving_every_reporter_what_it_needs
region: charge
asynchronous: ch33_giving_every_reporter_what_it_needs_async
```

In the asynchronous form, the charge's step reporter is an `AsyncStepReporter`, and `set_global_metadata` is awaited.

The log keeps each entry it receives in a `GlobalMetadata`, from `itinera::report`, through the method `set_global_metadata` of `Reporter`. It starts each line with the customer's name and, once there is one, the receipt:

```example
name: ch33_giving_every_reporter_what_it_needs
region: log
asynchronous: ch33_giving_every_reporter_what_it_needs_async
```

`set_global_metadata` stays synchronous on an `AsyncReporter` too: it only keeps the entry for the events that follow. `main` runs Ann's and Pat's orders, and prints:

```output
name: ch33_giving_every_reporter_what_it_needs
asynchronous: ch33_giving_every_reporter_what_it_needs_async
```

## How it works

1. **Global metadata is for the reporters.** Each entry is a key and a value, which every reporter of the journey receives. Steps and hooks never read it, so it changes no decision.
2. **The entries come before the first event.** The journey ID comes first, under the key `journey_id`, then the instance's entries, in the order they were set. So Pat's first line, `journey_started`, already names Pat. The key `journey_id` is the executor's own.
3. **A reporter does what it wants with an entry.** By default, `set_global_metadata` does nothing, which is why the desk is unchanged. The log keeps the entries, and reads them back with `get`, as an `AnyValue` that `downcast_ref` reads as `String`.
4. **A step can set an entry while it runs.** Every reporter receives it, then the journey emits `global_metadata_set`, which names the key and who set it, but not the value, all before `set_global_metadata` returns. The log wrote that line with the receipt in it, event 13, before it failed on the charge's success.
5. **An entry stands whatever happens after.** Pat's receipt never reached the data bag, since the journey was aborted first, but the log had it. So the shop finds Pat's payment, `R-A-15`, in the log's last line, and can refund it.

Every line of the log now says whose order it is and, from the charge on, how it was paid. But the audit's line, which a hook writes, says neither which journey nor which attempt took the money.

## Outcomes

We have shown how a journey's reporters receive data of their own, from the instance and from the steps, as global metadata.

| The usual way | With Itinera |
|---|---|
| A value every log line needs, such as a customer's name, is often passed down through every call, or kept in a global variable. | The instance gives it once to every reporter, before the first event. |
| A value a log line needs, found halfway through a request, is often lost when the request crashes right after. | A step sets it for every reporter as soon as it has it, and it stands whatever happens next. |
