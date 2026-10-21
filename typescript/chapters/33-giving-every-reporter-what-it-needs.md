# Giving every reporter what it needs

## What you want to do

The logging service files lines by customer, and the shop needs the bank's receipt to refund an order. You want every line of the log to name the customer, and the receipt once the bank has given it, even when the journey stops right after.

## The code

The shop is [chapter 31](31-stopping-a-journey-when-its-log-fails.md)'s, without chapter 32's `DebugReporter`. The program gives each instance the customer's name with `reportersGlobalMetadata`, among the options of `instance(...)` beside the `reporters` of [chapter 30](30-testing-a-whole-journey.md). That makes it an entry of the journey's **global metadata**: data for its reporters, which no step reads:

```example
name: ch33_giving_every_reporter_what_it_needs
region: instance
```

`shop` is made as [chapter 24](24-running-many-journeys.md) makes it, and `order` is each of [chapter 31](31-stopping-a-journey-when-its-log-fails.md)'s orders, Ann's and Pat's, which the loop hands over one at a time.

The charge asks for its step reporter, as the shipping does since [chapter 9](09-saying-what-only-a-step-knows.md), and sets the receipt as an entry as soon as the bank gives it, before contributing it:

```example
name: ch33_giving_every_reporter_what_it_needs
region: charge
```

The payment client it asks for is [chapter 26](26-running-async-journeys-while-others-wait.md)'s `Payments`, with [chapter 7](07-giving-a-step-the-tools-it-needs.md)'s token `PAYMENTS`. `setGlobalMetadata` returns a promise, like the step reporter's `info`, so the charge awaits it.

The log keeps each entry it receives in a `Map`, through `receiveGlobalMetadata`, a method a reporter may add beside `report`. A value comes as a `PlainValue`, from `itinera/value`: plain data, such as a string, a number or an object of them. The log starts each line with the customer's name and, once there is one, the receipt:

```example
name: ch33_giving_every_reporter_what_it_needs
region: log
```

The logging service is [chapter 31](31-stopping-a-journey-when-its-log-fails.md)'s. `detail` has one more case, which names the key of `global_metadata_set`:

```example
name: ch33_giving_every_reporter_what_it_needs
region: metadata_set
```

The program runs Ann's and Pat's orders, and prints:

```output
name: ch33_giving_every_reporter_what_it_needs
```

## How it works

1. **Global metadata is for the reporters.** Each entry is a key and a value, which every reporter of the journey receives. Steps and hooks never read it, so it changes no decision.
2. **The entries come before the first event.** The journey ID comes first, under the key `journey_id`, then the instance's entries, in the order they were given. So Pat's first line, `journey_started`, already names Pat. The key `journey_id` is the executor's own.
3. **A reporter does what it wants with an entry.** A reporter without `receiveGlobalMetadata` accepts every entry and does nothing with it, which is why [chapter 11](11-naming-a-journey.md)'s desk is unchanged. The log keeps the entries, and reads them back from its `Map`, checking with `typeof` that a value is a string before it writes it.
4. **A step can set an entry while it runs.** Every reporter receives it, then the journey emits `global_metadata_set`, which names the key and who set it, but not the value, all before the promise `setGlobalMetadata` returns settles. The log wrote that line with the receipt in it, event 13, before it failed on the charge's success.
5. **An entry stands whatever happens after.** Pat's receipt never reached the data bag, since the journey was aborted first, but the log had it. So the shop finds Pat's payment, `R-A-15`, in the log's last line, and can refund it.

Every line of the log now says whose order it is and, from the charge on, how it was paid. But the audit's line, which a hook writes, says neither which journey nor which attempt took the money.

## Outcomes

We have shown how a journey's reporters receive data of their own, from the instance and from the steps, as global metadata.

| The usual way | With Itinera |
|---|---|
| A value every log line needs, such as a customer's name, is often passed down through every call, or kept in a global variable. | The instance gives it once to every reporter, before the first event. |
| A value a log line needs, found halfway through a request, is often lost when the request crashes right after. | A step sets it for every reporter as soon as it has it, and it stands whatever happens next. |
