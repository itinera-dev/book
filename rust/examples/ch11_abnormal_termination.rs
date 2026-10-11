//! When something goes wrong inside a step: a declined card is a failure the step reports on
//! purpose, and a payment network that is down is an error the step lets escape.

use std::fmt;

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, Failure, InitialData, JourneyResult, JourneyStatus, Output};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

// ANCHOR: bank
/// What the customer's bank answers when it is asked to charge.
enum Answer {
    Approved { receipt: String },
    Declined,
}

/// The error of a bank that cannot be reached.
#[derive(Debug)]
struct NetworkDown;

impl fmt::Display for NetworkDown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the payment network is down")
    }
}

impl std::error::Error for NetworkDown {}

/// The customers' banks, faked: Ann's approves, Ben's declines, and Cat's cannot be reached.
fn bank_charges(customer: &str, order: &str) -> Result<Answer, NetworkDown> {
    match customer {
        "Ann" => Ok(Answer::Approved {
            receipt: format!("R-{order}"),
        }),
        "Ben" => Ok(Answer::Declined),
        _ => Err(NetworkDown),
    }
}
// ANCHOR_END: bank

// ANCHOR: step
/// Charges an order through the customer's bank.
#[derive(Step)]
struct Charge {
    #[input]
    order: String,
    #[input]
    customer: String,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        match bank_charges(&self.customer, &self.order)? {
            Answer::Approved { receipt } => {
                contributor.contribute("receipt", receipt);
                Self::succeed()
            }
            Answer::Declined => Self::fail("declined"),
        }
    }
}
// ANCHOR_END: step

/// The shop's workflow: for now, it only charges.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
struct Orders;

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
}

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Receipt {
    receipt: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut executor = LocalExecutor::new();
    for (order, customer) in [("A-1", "Ann"), ("A-2", "Ben"), ("A-3", "Cat")] {
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                customer: customer.to_owned(),
            })
            .create()?;
        report(order, &executor.run(instance)?);
    }
    Ok(())
}

// ANCHOR: report
/// Prints how one order's journey ended, and why when it failed.
fn report(order: &str, result: &JourneyResult) {
    match &result.status {
        JourneyStatus::Failed { failure, .. } => match failure {
            Failure::Failure(reason) => {
                println!("{order}: failed, {}: {}", failure.cause(), reason.code());
            }
            Failure::AbnormalTermination(error) => {
                println!("{order}: failed, {}: {error}", failure.cause());
            }
            _ => println!("{order}: failed, {}", failure.cause()),
        },
        status => match result.output::<Receipt>() {
            Ok(receipt) => println!("{order}: {}, receipt {}", status.kind(), receipt.receipt),
            Err(_) => println!("{order}: {}", status.kind()),
        },
    }
}
// ANCHOR_END: report
