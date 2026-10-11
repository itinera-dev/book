# When something goes wrong inside a step

## What you want to do

The charge asks the customer's bank for the money. The bank may approve, it may decline the card, or it may not answer at all because the payment network is down. You want a declined card to be a failure the step reports, and a network that is down to be told apart from it.

## The code

The bank is faked inside the program. Its answer is either approved, with a receipt, or declined; a network that is down is an error instead of an answer.

```example
name: ch11_abnormal_termination
region: bank
```

The charge turns the bank's answers into outcomes. It does not handle the error: it lets it escape.

```example
name: ch11_abnormal_termination
region: step
```

The shop runs three orders, one for each customer, and prints how each journey ended, and why when it failed.

```example
name: ch11_abnormal_termination
region: report
```

It prints:

```output
name: ch11_abnormal_termination
```

```note
name: ch11_abnormal_termination
```

## How it works

1. **A declined card is a failure on purpose.** The bank answered, and its answer is one the business expects. The step reports a failure with the reason `declined`, and Ben's journey fails with the cause `failure` and that reason.
2. **An error that escapes is an abnormal termination.** For Cat's order the bank gave no answer, only an error, and the step let it escape instead of choosing an outcome. That ends the attempt as a failure, called an abnormal termination. Cat's journey fails with the cause `abnormal termination`, and the result carries the error itself, with its message.
3. **It is still a failure, never an abort.** An abnormal termination never aborts the journey. It is a failure of the step like any other, so the journey has an output, and the policies attached to the step can react to it, as chapter 16 shows.
4. **It is not retried unless the workflow says so.** An error the step did not expect may already have had effects: the bank may have taken the money before the network went down. So an abnormal termination is not retried by default, whatever the step's retry budget. The workflow can declare that a step's abnormal terminations may be retried, as chapter 13 shows.
5. **It carries no contributions.** Whatever the step contributed before the error escaped is never committed, as for any failed attempt.
6. **Choosing between them.** When the step can say what happened in the business's own terms, it reports an outcome: a failure with a reason, retriable when trying again might help. When it cannot, it lets the error escape, and the error tells the rest. If the shop knew a charge could safely be tried again, the step could catch the error and report a retriable failure, `payment-network-down`, instead.
7. **An unrecoverable failure is outside the model.** Some failures the language itself treats as unrecoverable. If a step fails that way, the journey stops as if the program had crashed: no more events, and no result. Itinera never turns such a failure into an outcome, so it is never the way to report something that can happen to an order.

## Remember

1. A failure the business expects is an outcome: report it with a reason.
2. An error a step lets escape ends its attempt as an abnormal termination: the step fails, and the journey is never aborted for it.
3. An abnormal termination is not retried unless the workflow declares that it may be.
4. Never use an unrecoverable failure for something that can happen in a journey: the journey stops with no result.
