# Quick start

## What you want to do

You want to see Itinera at work in Rust in one go, before reading about it. Here is the shop on a small scale: it charges an order, then ships it. The main carrier is down today, so shipping has to fall back to the backup carrier. And you want to watch everything that happens.

Don't worry about understanding every line yet. Each piece has a chapter of its own later. For now, notice where each decision lives.

## The code

Two steps hold the business rules, each a struct with an `impl` block. The charge fails when there is nothing to charge, and otherwise contributes what it charged. Shipping uses the carrier it is given; without one it tries the main carrier, which is down, so it fails in a way that is worth trying again.

```example
name: quick_start
region: steps
```

A policy holds the flow decision. When shipping is about to be tried again, its hook names the backup carrier for the next attempt.

```example
name: quick_start
region: policy
```

A reporter receives every event of the journey. It is any type that implements the `Reporter` trait, and this one prints each event on a line of its own.

```example
name: quick_start
region: reporter
```

The workflow puts it all together, in one place: charge, then ship; shipping may be tried once more, with the fallback policy deciding how; and every event goes to the printer.

```example
name: quick_start
region: workflow
```

Finally, the program says what the shop knows about order A-1, runs one journey for it, and reads back what the journey produced.

```example
name: quick_start
region: run
```

It prints:

```output
name: quick_start
```

## How it works

Read the output from the top, and you are watching the journey:

1. The journey starts, and the charge succeeds on its first attempt. What it contributed, `charged`, is committed to the data bag, the journey's store of data.
2. Shipping's first attempt has no carrier to use: `optional_input_absent` says so. It tries the main carrier, which is down, so it fails with the reason `carrier-down`, and says that trying again might work.
3. Shipping may be tried once more, so the executor calls the fallback policy's hook first. The hook contributes `carrier`, and the executor tries shipping again.
4. The second attempt gets the backup carrier, and succeeds with a tracking number.
5. The journey succeeds, and the program reads back what was charged and the tracking number.

Look at where each decision lives, and how little Rust it takes to say it. Everything here is declared with Itinera's macros, which check the declarations when the program compiles:

1. **The steps** know nothing about each other, or about retries. Deriving `Step` makes each field marked `#[input]` an input of the step, read from the journey's data under the field's name; a field of type `Option<String>`, such as `carrier`, is an optional input, `None` when the data is not there. The `#[step]` attribute declares `run`, which returns `Result<Outcome, Error>`: `Self::succeed()`, `Self::fail(...)` for a failure, or `Self::fail_retriably(...)` for a failure that trying again might mend. A parameter marked `#[contributor]` is how a step contributes data. Shipping only uses the carrier it is given, and says when it could not ship.
2. **The policy** decides what changes before shipping is tried again. The attribute `#[step_policy]` declares it, and `#[on_step_retry]` marks the method that is its retry hook. The hook contributes the carrier, then returns `Self::carry_on()`, which keeps the default: shipping is tried again.
3. **The workflow** says, in a few lines, what runs and how many times. The attribute `#[workflow]` declares it, with one `#[with_step(...)]` line per step, in the order they run; `retry_budget = 1` lets shipping be tried once more, `policies = [Fallback]` attaches the policy, and `reporters = [Printer]` sends every event to the printer.
4. **The journey's data** are plain structs. Deriving `InitialData` turns `NewOrder` into the journey's initial data, and deriving `Output` lets the program read the data bag back into `Shipped` once the journey is over.
5. **The events** tell the whole story without a single log line in the business code. The printer's `report` method receives each one, in order.

To run it yourself, run `cargo run --example quick_start` in the book's `rust` directory. To use Itinera in your own project, see [Installing Itinera](05-installing-itinera.md).

The rest of the book takes each of these pieces in turn. Part 1 starts with the question you may be asking already: why bother?

## Remember

1. Business rules go in steps; decisions about what happens next go in policies; the order and the number of attempts go in the workflow.
2. A step never knows what comes before it, after it, or whether it will be tried again.
3. Everything a journey does is an event, and a reporter sees every one.
