# Running async journeys while others wait

## What you want to do

The shop's bank and carriers answer over the network, and while one order waits for them, the shop waits too, though it could take other orders forward in the meantime. You want the shop to run several orders at once, each going on while the others wait.

## The code

The shop is [chapter 25](25-reading-the-result.md)'s, with the bank and the carriers answering asynchronously. Itinera's TypeScript executor is asynchronous, and accepts steps and hooks in either **execution mode**, synchronous or asynchronous: as [chapter 9](09-saying-what-only-a-step-knows.md) showed, a step's `run` may return its outcome or a promise of it, and the executor waits for whichever it gets. So nothing needs to be turned on: Node's own event loop runs the promises.

The payment client and the carrier now return promises, and reject with [chapter 12](12-trying-again.md)'s errors; their tokens are [chapter 7](07-giving-a-step-the-tools-it-needs.md)'s. The order book of [chapter 22](22-finishing-a-journey-early.md) answers from the shop's own records without waiting, so it stays synchronous, as any service or role that doesn't wait may:

```example
name: ch26_running_async_journeys_while_others_wait
region: services
```

The faked bank and carriers await `network` before each answer. It waits for one turn of Node's event loop, with `setImmediate` from `node:timers/promises`, as a real call waits for the other side. No clock is involved, so the answers come in the order they were asked for:

```example
name: ch26_running_async_journeys_while_others_wait
region: network
```

The charge and its rescue step now await the bank, so their `run` becomes `async`, returning a promise of their outcome:

```example
name: ch26_running_async_journeys_while_others_wait
region: charge
```

The shipping's `run` was already `async`, for its step reporter. It now awaits the carrier too:

```example
name: ch26_running_async_journeys_while_others_wait
region: ship
```

Telling a customer and giving money back go over the network too, so the shop's roles return promises the same way:

```example
name: ch26_running_async_journeys_while_others_wait
region: roles
```

The workflow's implementations of them await `network` before they answer. The rest of the workflow class is [chapter 23](23-undoing-a-charge-when-the-shipping-fails.md)'s:

```example
name: ch26_running_async_journeys_while_others_wait
region: role_impls
```

The hooks that call a role become `async` as well, and await it: `Notify`'s two hooks, which now return a promise of nothing, and `FailedShipping`'s, which return a promise of their decision. `refund`, which calls the role, becomes `async` too:

```example
name: ch26_running_async_journeys_while_others_wait
region: notify
```

```example
name: ch26_running_async_journeys_while_others_wait
region: failed_shipping
```

The program runs three of the day's orders, Ann's, Gil's and Jo's, at the same time. `journey` runs an instance with an executor of its own; `WorkflowInstance`, from `itinera/instance`, is the type of what `instance(...)` makes, here for `Orders`. `Promise.all` waits for the three journeys together, and gives their results in the order it was given them. It then prints each result's `journeyId`, which a result carries as events do, and how it ended:

```example
name: ch26_running_async_journeys_while_others_wait
region: run
```

It prints:

```output
name: ch26_running_async_journeys_while_others_wait
```

## How it works

1. **Only what waits changes.** The charge, its rescue step, the shipping and the hooks that call a role await what they call, so they are `async`. The rest waits for nothing, and stays as it was: the lookup and `AlreadyCompleted` as [chapter 22](22-finishing-a-journey-early.md) shows them, the pricing and the audit as [chapter 20](20-an-optional-input.md) does, and `Outage` as [chapter 17](17-giving-up-a-step-early.md) does. The workflow's declaration, its input adapter, its ID generator and its providers don't change, nor do the log, as [chapter 22](22-finishing-a-journey-early.md) shows it, and the desk, as [chapter 11](11-naming-a-journey.md) does.
2. **Each step and each hook has its own execution mode.** There is no mode to choose for the workflow, or for the executor: the executor waits for whatever a step's `run` or a hook returns, a value or a promise of it, so the same journey runs synchronous and asynchronous steps alike. The journeys end as they did in chapter 25: Gil's parcel goes with the backup carrier, and Jo's card is renewed.
3. **A journey that waits lets the others go on.** The executor awaits everything it calls, reporters, steps and hooks alike, even one that answers at once, so from their first events the three journeys take turns: Ann's `journey_started`, then Gil's, then Jo's. Then each answer of the bank or a carrier comes in its turn, and its journey goes on from where it waited, until it waits again. So the log interleaves the three journeys.
4. **The output is the same on every run.** Nothing here waits for a clock, so the turns come in the same order each time. Real answers over the network come when they come, and their journeys go on in that order, but each journey's own events keep their order whatever the others do.
5. **An executor runs one journey at a time,** as [chapter 24](24-running-many-journeys.md) showed, so each journey that runs at the same time as others gets an executor of its own.

Ann's, Gil's and Jo's journeys now go on together, each while the others wait. But the only way to know that the shop still gives the same answers after a change is to run every order again, against the bank and the carriers, and read the log.

## Outcomes

We have shown how the shop runs journeys at the same time, each going on while the others wait.

| The usual way | With Itinera |
|---|---|
| Making code asynchronous often means changing every function between the request and the call that waits. | The services and roles that wait return promises, and the steps and hooks that await them become `async`; nothing between them and the executor changes, since it waits for whatever each returns. |
