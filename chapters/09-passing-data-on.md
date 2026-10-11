# Passing data on

## What you want to do

The charge gives a receipt, and the parcel must carry it. You want the shipping to receive the receipt the charge made, and you want the order's data to stay true when the shipping fails: no tracking number for a parcel that never left.

## The code

The charge contributes two things: the amount it charged, and the receipt.

```example
name: ch09_contributions
region: charge
```

The shipping asks for the receipt as one of its inputs, as it asks for the order's ID. It books the carrier, contributes the tracking number, and only then learns whether the carrier can deliver.

```example
name: ch09_contributions
region: ship
```

The shop reads the same data back whether the journey succeeded or failed. The tracking number may be missing.

```example
name: ch09_contributions
region: output
```

The shop runs two orders: Ada's, whose address the carrier knows, and Bo's, whose address it does not.

```example
name: ch09_contributions
region: run
```

It prints:

```output
name: ch09_contributions
```

```note
name: ch09_contributions
```

## How it works

1. **A step contributes through its contributor.** The contributor is the step's only way to add data: it takes a key and a value, and keys need no declaring in advance. What a step gives it is a contribution.
2. **A successful attempt's contributions are committed.** Each run of a step is an attempt. When the charge's attempt succeeds, `charged` and `receipt` are committed to the data bag, the journey's data, and every later step and the result can see them.
3. **A later step asks for them like any other input.** The shipping asks for `receipt` by its key. It does not know the charge made it, and the charge does not know anyone reads it. If the workflow listed the shipping before the charge, the receipt would not be there yet, and the journey would be aborted with `required data missing`.
4. **A failed attempt's contributions are never committed.** For Bo's order, the shipping contributed `tracking` and then failed. Its contributions are discarded: the tracking number never reaches the data bag, so no later step and no result can believe the parcel left.
5. **A failed journey still has its data.** The journey failed, and its output is the data bag as it stood at the end: the order, the amount charged and the receipt, all committed by the charge's successful attempt, and nothing from the shipping's failed one. Only an aborted journey has no output.
6. **The last value wins.** A step that contributes the same key twice in one attempt keeps the last value. A committed contribution may also replace a key already in the data bag, even one from the initial data, and the events record that it did.
7. **Only values go in.** Everything in the data bag is a value: data that could be serialized, such as numbers, text, lists and records. A connection, a client or a function never goes in. Chapter 12 shows how a step gets those.

Policies, which later chapters introduce, can look at what a failed attempt contributed when they decide what happens next. That is all such contributions are for: they never reach a later step or the result.

## Remember

1. A step changes the data bag only by contributing, and its contributions are committed only when its attempt succeeds.
2. A step reads what an earlier step contributed as an ordinary input, by key, never by asking for that step.
3. A step that needs a key some step contributes runs after it in the workflow's list.
4. Only values go in the data bag: never connections, clients or functions.
