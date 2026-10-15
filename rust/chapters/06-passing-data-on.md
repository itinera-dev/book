# Passing data on

## What you want to do

When the bank charges an order, it gives a receipt. You want the charge to give that receipt back, so that the shop has it once the journey ends, and so that the parcel can carry it when the shop ships the order.

## The code

The charge now asks for the order's number as well as its price. The bank, still faked, gives a receipt, and the charge hands it on through a **contributor**, a parameter of `run` of type `&mut Contributor`.

```example
name: ch06_passing_data_on
region: charge
```

Shipping is a second step. It books the carrier, which gives a tracking number at once, and hands that on the same way. The carrier, faked too, weighs the parcel only when it comes to collect it, and takes it only up to 20 kg. The receipt is one of the shipping's inputs, asked for like the order's number.

```example
name: ch06_passing_data_on
region: ship
```

The workflow lists the two steps, one `#[with_step(...)]` line each:

```example
name: ch06_passing_data_on
region: workflow
```

Each order now carries its number and its parcel's weight as well as its price:

```example
name: ch06_passing_data_on
region: data
```

Once a journey has ended, the shop reads back the receipt and the tracking number into a struct of its own. A tracking number may be missing, so its field is an `Option<String>`.

```example
name: ch06_passing_data_on
region: output
```

Two orders come in: Ann's, whose parcel weighs 2 kg, and Gil's, whose parcel weighs 60 kg.

```example
name: ch06_passing_data_on
region: run
```

It prints:

```output
name: ch06_passing_data_on
```

## How it works

1. **A step gives back what it did by contributing it.** The contributor's `contribute` method takes a name and a value: here the receipt, under `receipt`. What a step gives this way is a **contribution**, and its names need no declaring in advance. A step's contributor belongs to one **attempt**, one run of the step: `run` borrows it for that attempt only, and it serves no other.
2. **The journey's data is its data bag.** A journey holds its data in its **data bag**: values under names. It starts as the initial data, and a step's contributions are committed to it when its attempt succeeds. Only values go in: data that could be serialized, such as numbers, strings, and structs that derive `Clone` and Serde's `Serialize` and `Deserialize`.
3. **A later step reads a contribution as an ordinary input.** The shipping asks for `receipt` with an `#[input]` field, just as it asks for the order's number, which came from the initial data. It doesn't know the charge made the receipt, and the charge doesn't know anyone reads it. **A step never calls another step: it reads what an earlier one contributed, by name.**
4. **The steps run in the order the workflow lists them.** The charge's line comes first, so the charge runs first, and once it has succeeded, the shipping runs. Nothing reorders them: to read what the shop does with an order, you read the workflow's list, top to bottom.
5. **A failed attempt's contributions are never committed.** For Gil's order, the carrier booked the parcel and gave its tracking number, which the shipping contributed. Then the carrier refused to collect a parcel of 60 kg, and the shipping failed with `too-heavy`. **A step's contributions are committed to the data bag only when its attempt succeeds**, so Gil's tracking number never reached it, and nothing in the shop can believe his parcel left.
6. **The data bag is the journey's output.** When a journey ends, its data bag, as it stands then, is its output, whether the journey succeeded or failed. `#[derive(Output)]` lets `result.output()` read it into `Order`, each field from the name it is called by. An `Option` field is `None` when its name is absent, as `tracking` is after Gil's journey. It returns a `Result`, since the data bag might lack what a field needs, and `?` passes such an error on. Ann's output holds her receipt and her tracking number. Gil's holds his receipt, since his charge succeeded, and no tracking number.

So the charge gives back its receipt, and the shipping uses it, without either knowing about the other. But each still reaches the outside world on its own: the charge calls the shop's one bank, and the shipping its one carrier, by name, inside its `run`. A step written that way can only ever use that bank and that carrier. To charge through another bank, or a pretend one, you would have to change the step.

## Outcomes

We have shown how a step gives back what it did, and how a later step and the shop read it.

| The usual way | With Itinera |
|---|---|
| A function usually returns what it did to its caller, which hands it on to the next function itself. | A step contributes what it did, and a later step or the shop reads it by name, with no step calling another. |
| When a function fails halfway, what it already wrote often stays written, and undoing it is up to the code. | A failed attempt's contributions are never committed, so the data bag holds only what steps that succeeded gave. |
