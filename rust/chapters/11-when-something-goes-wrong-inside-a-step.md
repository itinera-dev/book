# When something goes wrong inside a step

## What you want to do

The charge asks the customer's bank for the money. The bank may approve, it may decline the card, or it may not answer at all because the payment network is down. You want a declined card to be a failure the step reports, and a network that is down to be told apart from it.

## The code

The bank is faked inside the program. Its answer is either approved, with a receipt, or declined; a network that is down is an error instead of an answer, so `bank_charges` returns `Result<Answer, NetworkDown>`.

```example
name: ch11_abnormal_termination
region: bank
```

The charge turns the bank's answers into outcomes. It doesn't handle the error: the `?` after `bank_charges(...)` lets it escape.

```example
name: ch11_abnormal_termination
region: step
```

The shop runs three orders, one for each customer. The `report` function prints how each journey ended, and why when it failed; [chapter 22](22-reading-the-result.md) shows how to read a result.

```example
name: ch11_abnormal_termination
region: run
```

It prints:

```output
name: ch11_abnormal_termination
```

## How it works

A step's `run` returns `Result<Outcome, Error>`. An `Ok` holds the outcome the step chose, as in the previous chapter. An `Err` is something else: an error that escaped the step.

1. **A declined card is a failure on purpose.** The bank answered, and its answer is one the business expects. The step returns `Self::fail("declined")`, and Ben's journey fails with the cause `failure` and that reason.
2. **An error that escapes is an abnormal termination.** For Cat's order the bank gave no answer, only an error, and the step let it escape instead of choosing an outcome. That ends the attempt as a failure, called an abnormal termination. Cat's journey fails with the cause `abnormal termination`. The result holds the error itself, and `report` prints its message.
3. **A question mark is all it takes.** `itinera::error::Error` converts from any error that implements `std::error::Error + Send + Sync + 'static`, so the `?` on the bank's `Result` turns `NetworkDown` into an abnormal termination with no more code. When you have only a message and no error type, `Error::msg("...")` makes one.
4. **It is still a failure, never an abort.** An abnormal termination never aborts the journey. It's a failure of the step like any other, so the journey has an output, and the policies attached to the step can react to it, as [chapter 16](16-ending-a-journey-early.md) shows.
5. **It is not retried unless the workflow says so.** An error the step didn't expect may already have had effects: the bank may have taken the money before the network went down. So an abnormal termination isn't retried by default, whatever the step's retry budget. The workflow can declare that a step's abnormal terminations may be retried, as [chapter 13](13-trying-again.md) shows.
6. **It carries no contributions.** Whatever the step contributed before the error escaped is never committed, as for any failed attempt.
7. **Choosing between them.** When the step can say what happened in the business's own terms, it returns an outcome: a failure with a reason, retriable when trying again might help. When it can't, it returns the error, and the error tells the rest. If the shop knew a charge could safely be tried again, the step could `match` on the bank's `Result` instead of using `?`, and turn `NetworkDown` into `Self::fail_retriably("payment-network-down")`.
8. **A panic is outside the model.** A panic is Rust's unrecoverable failure, and Itinera never catches one. A panic in a step, such as an `unwrap` on `None` or an index out of bounds, stops the journey as if the program had crashed: no more events, and no result. It propagates out of the executor's `run` to whoever called it. Itinera never turns a panic into an outcome, so it is never the way to report something that can happen to an order. Keep `unwrap`, `expect` and `panic!` out of steps for anything an order can bring, and return an error or an outcome instead.

## Remember

1. A failure the business expects is an outcome: return it with a reason.
2. An `Err` a step returns ends its attempt as an abnormal termination: the step fails, and the journey is never aborted for it.
3. An abnormal termination is not retried unless the workflow declares that it may be.
4. Never panic for something that can happen in a journey: the journey stops with no result.
