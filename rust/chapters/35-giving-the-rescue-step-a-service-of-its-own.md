# Giving the rescue step a service of its own

## What you want to do

The bank now answers which card replaced an expired one from its card registry, a system of its own in each region, and its library hands back the registry's client already boxed. You want the rescue step to ask the registry instead of the payment system, and no other step to have it.

## The code

The shop is [chapter 34](34-saying-which-attempt-took-the-money.md)'s. The registry is a service with a trait of its own, `Cards`, and the bank's library makes its client for a region:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: cards
asynchronous: ch35_giving_the_rescue_step_a_service_of_its_own_async
```

The rescue step asks for the registry under the key `cards`, though it calls it `registry`, and reads the order's card, under the key `card`, as `expired`:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: rescue
asynchronous: ch35_giving_the_rescue_step_a_service_of_its_own_async
```

The workflow's provider for `cards` is attached to the rescue step alone, by its name, and hands over the library's boxed client with the region it is for:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: provider
asynchronous: ch35_giving_the_rescue_step_a_service_of_its_own_async
```

The log's `detail` now hands the data a provider supplied to `supplied`, in its arm for `service_provided`:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: service_provided
```

`supplied` writes the data when it is text, and nothing otherwise:

```example
name: ch35_giving_the_rescue_step_a_service_of_its_own
region: supplied
```

In the asynchronous form, `Cards` and its implementation take `#[async_trait]`, and the rescue step awaits `replacement`. `order` is [chapter 33](33-giving-every-reporter-what-it-needs.md)'s, `journey` [chapter 26](26-running-async-journeys-while-others-wait.md)'s and `Audited` [chapter 34](34-saying-which-attempt-took-the-money.md)'s. `main` runs Jo's order, whose expired card the rescue step renews, and prints:

```output
name: ch35_giving_the_rescue_step_a_service_of_its_own
asynchronous: ch35_giving_the_rescue_step_a_service_of_its_own_async
```

## How it works

1. **A request can name its key.** Until now, a step's input and service were found under the name of the field or the parameter. `#[service("cards")]` asks for the service under the key `cards`, and `#[input("card")]` reads the data bag's `card` into the field `expired`. The step names what it holds for its own code, and the key says what the journey calls it.
2. **A provider can serve some steps only.** `steps = ["charge_rescue"]` attaches the registry's provider to the rescue step, named by its own name in the workflow, as [chapter 14](14-repairing-then-trying-once-more.md) showed. The bank's provider, [chapter 12](12-trying-again.md)'s, attached to no step, still serves every step that asks for `payments`, which is now the charge alone.
3. **A provider can hand over a service already boxed.** `Provided::from_box` takes the `Box<dyn Cards>` the bank's library made, as it is. The box hides the client's own type, so `service_provided`, event 16, names the implementation as the trait, `dyn …::Cards`, where `provide!` named the type inside, as [chapter 8](08-seeing-what-happened.md) showed.
4. **A provider can supply data with a service.** `with_data` gives the region along with the client, and `service_provided` carries it. The log now writes the data a provider supplied after the implementation: `with eu`. It says which registry served the rescue step, which the boxed type can't. Like everything an event carries, it is for the reporters: the rescue step never sees it.

Jo's card is now renewed by the registry, and only the rescue step can reach it. Jo's order came complete, with its number and its customer's name. But the website has started sending orders before they are finished: one arrives without an order number, another without the customer's name.

## Outcomes

We have shown how a step names the keys of what it asks for, and how a provider serves some steps only, with a service already boxed and data that says which one it is.

| The usual way | With Itinera |
|---|---|
| A function's parameters often have to be named as its callers name the values, or a layer between them renames everything. | A step names its inputs and services for its own code, and says under which key the journey has each. |
| A client that a vendor's library builds often hides its type, so nothing records which one a request used. | The provider hands the boxed client over as it is, with data saying which one it is, on `service_provided`. |
