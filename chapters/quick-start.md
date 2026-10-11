# Quick start

## What you want to do

You want to see Itinera in one go before reading about it. Here is the shop on a small scale: it charges an order, then ships it. The main carrier is down today, so shipping has to fall back to the backup carrier. And you want to watch everything that happens.

Don't worry about understanding every line yet. Each piece has a chapter of its own later. For now, notice where each decision lives.

## The code

Two steps hold the business rules. The charge fails when there is nothing to charge, and otherwise contributes what it charged. Shipping uses the carrier it is given; without one it tries the main carrier, which is down, so it fails in a way that is worth trying again.

```example
name: quick_start
region: steps
```

A policy holds the flow decision. When shipping is about to be tried again, its hook names the backup carrier for the next attempt.

```example
name: quick_start
region: policy
```

A reporter receives every event of the journey. This one prints each event on a line of its own.

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

```note
name: quick_start
```

## How it works

Read the output from the top, and you are watching the journey:

1. The journey starts, and the charge succeeds on its first attempt. What it contributed, `charged`, is committed to the data bag, the journey's store of data.
2. Shipping's first attempt has no carrier to use, so it tries the main carrier, which is down. It fails with the reason `carrier-down`, and says that trying again might work.
3. Shipping may be tried once more, so the executor calls the fallback policy's hook first. The hook contributes `carrier`, and the executor tries shipping again.
4. The second attempt gets the backup carrier, and succeeds with a tracking number.
5. The journey succeeds, and the program reads back what was charged and the tracking number.

Look at where each decision lives. The steps know nothing about each other, or about retries: shipping only uses the carrier it is given, and says when it could not ship. The workflow says, in a few lines, what runs and how many times. The policy decides what changes before shipping is tried again. And the events tell the whole story without a single log line in the business code.

The rest of the book takes each of these pieces in turn. Part 1 starts with the question you may be asking already: why bother?

## Remember

1. Business rules go in steps; decisions about what happens next go in policies; the order and the number of attempts go in the workflow.
2. A step never knows what comes before it, after it, or whether it will be tried again.
3. Everything a journey does is an event, and a reporter sees every one.
