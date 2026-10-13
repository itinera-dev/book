# A carrier for each attempt

## What you want to do

When the main carrier won't collect a parcel, the shop has a backup carrier that will collect heavier ones. You want each attempt of the shipping to get its own carrier, the main one first and the backup one when the shipping is tried again, without the shipping knowing that there are two.

## The code

The two carriers are faked, like the main one before: each books any parcel, prints what it does, and collects a parcel only up to its weight limit, 20 kg for the main carrier and 80 kg for the backup.

```example
name: ch13_a_carrier_for_each_attempt
region: carriers
```

The workflow's provider for `carrier` is now marked `per_attempt`, and asks for the attempt's number with a parameter marked `#[attempt]`. It makes the main carrier for attempt 1, and the backup carrier for any attempt after it.

```example
name: ch13_a_carrier_for_each_attempt
region: provider
```

Everything else is [chapter 12](12-trying-again.md)'s shop, unchanged, including the shipping, which still asks for a `&dyn Carrier` and fails retriably when it won't be collected, and its retry budget of 1. It runs Ann's, Gil's and Jo's orders again, and prints:

```output
name: ch13_a_carrier_for_each_attempt
```

## How it works

1. **A provider decides how long what it makes is kept.** A provider marked `#[provider]` alone, as in [chapter 7](07-giving-a-step-the-tools-it-needs.md), is called once for a journey, the first time a step asks, and every attempt that asks for its key gets the same instance. One marked `per_attempt` is called for every attempt that asks, and what it makes serves that attempt only, then is dropped. **How long a service lives is the provider's decision, never the step's.**
2. **A provider for every attempt may ask which attempt it serves.** A parameter marked `#[attempt]`, a `NonZeroU32`, is the number of the attempt the provider is called for, so the provider can decide from it which carrier to make. `NonZeroU32::MIN` is 1, the first attempt.
3. **The log shows which carrier served which attempt.** Gil's journey now records `service_provided` before each attempt of the shipping: the `MainCarrier` for attempt 1, which wouldn't collect 60 kg, and the `BackupCarrier` for attempt 2, which did. The journey succeeds, and the tracking number committed is the backup carrier's, `B-A-7`, since only the attempt that succeeded commits what it contributed. Ann's parcel of 2 kg goes with the main carrier on attempt 1, and never meets the backup.
4. **The shipping is unchanged.** It still uses whatever carrier it is lent, and says only whether the parcel was collected. Choosing the backup carrier for a retry is the workflow's decision, made in its provider, as choosing the main carrier was.

Jo's order still fails at the charge. Her card expired in 2024, and no attempt with that card would ever be charged. But a bank usually knows which card replaced one that expired, and the charge has no way to ask for it: it charges the card it was given, and nothing more.

## Outcomes

We have shown how a provider gives each attempt of a step a service of its own, chosen by the attempt's number.

| The usual way | With Itinera |
|---|---|
| Falling back to another client on a second try often means the code that retries also knows every client it may fall back to. | The workflow's provider chooses the carrier for each attempt, and the step that uses it never knows there are two. |
