# Giving the rescue step a service of its own

## What you want to do

The bank now answers which card replaced an expired one from its card registry, a system of its own in each region, and its library hands back the registry's client as an object it makes itself. You want the rescue step to ask the registry instead of the payment client, and no other step to have it.

## The code

The shop is [chapter 34](34-saying-which-attempt-took-the-money.md)'s. Its payment client no longer answers which card replaced one, so `Payments` keeps only `charge`:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: payments
asynchronous: ch35_giving_the_rescue_step_a_service_of_its_own_async
```

The registry is a service with an interface of its own, `Cards`, and the bank's library makes its client for a region:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: cards
asynchronous: ch35_giving_the_rescue_step_a_service_of_its_own_async
```

`Registry` is the library's own class, faked here like the bank. In the asynchronous form, `charge`, `replacement` and the rescue step's `run` are `suspend` functions, as in [chapter 26](26-running-async-journeys-while-others-wait.md).

The rescue step asks for the registry under the key `cards`, though it calls it `registry`, and reads the order's card, under the key `card`, as `expired`:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: rescue
asynchronous: ch35_giving_the_rescue_step_a_service_of_its_own_async
```

The workflow class has one more provider, for `cards`, attached to the rescue step alone, by its name. It hands over the library's client with the region it is for:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: provider
asynchronous: ch35_giving_the_rescue_step_a_service_of_its_own_async
```

The rest of the workflow class is [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md)'s, and its asynchronous form [chapter 26](26-running-async-journeys-while-others-wait.md)'s, where the provider is a `suspend` function, as every provider of an asynchronous workflow is.

The log's `detail`, [chapter 33](33-giving-every-reporter-what-it-needs.md)'s with the parts that page points to, now hands the data a provider supplied to `supplied`, in its branch for `ServiceProvided`:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: service_provided
```

`supplied` writes the data when it is text, and nothing otherwise:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: supplied
```

`main` runs Jo's order, whose expired card the rescue step renews, as [chapter 34](34-saying-which-attempt-took-the-money.md) runs it, and prints:

```output
name: ch35_giving_the_rescue_step_a_service_of_its_own
asynchronous: ch35_giving_the_rescue_step_a_service_of_its_own_async
```

## How it works

1. **A request can name its key.** Until now, a step's input and service were found under the name of the parameter, as [chapter 4](04-a-first-step.md) and [chapter 7](07-giving-a-step-the-tools-it-needs.md) showed. `@Service("cards")` asks for the service under the key `cards`, and `@Input("card")` reads the data bag's `card` into the parameter `expired`. The step names what it holds for its own code, and the key says what the journey calls it.
2. **A provider can serve some steps only.** `steps = ["charge_rescue"]` attaches the registry's provider to the rescue step, named by its own name in the workflow, as [chapter 14](14-repairing-then-trying-once-more.md) showed. The bank's provider, [chapter 12](12-trying-again.md)'s, attached to no step, still serves every step that asks for `payments`, which is now the charge alone.
3. **A provider can hand over a client it didn't make.** `cardRegistry` makes the client, and the provider gives it on as it is. `service_provided`, event 16, names the implementation by the object's class, as [chapter 8](08-seeing-what-happened.md) showed: here the library's own class, `Registry`, which no code of the shop names.
4. **A provider can supply data with a service.** `provide(...)` takes the data as a second argument, here the region, and `service_provided` carries it as its `data`. The log now writes the data a provider supplied after the implementation: `with eu`. It says which registry served the rescue step, which the client's class can't. Like everything an event carries, it is for the reporters: the rescue step never sees it.

Jo's card is now renewed by the registry, and only the rescue step can reach it. Jo's order came complete, with its number and its customer's name. But the website has started sending orders before they are finished: one arrives without an order number, another without the customer's name.

## Outcomes

We have shown how a step names the keys of what it asks for, and how a provider serves some steps only, with a client it didn't make and data that says which one it is.

| The usual way | With Itinera |
|---|---|
| A function's parameters often have to be named as its callers name the values, or a layer between them renames everything. | A step names its inputs and services for its own code, and says under which key the journey has each. |
| A client that a vendor's library builds often hides where it came from, so nothing records which one a request used. | The provider hands the library's client over as it is, with data saying which one it is, on `service_provided`. |
