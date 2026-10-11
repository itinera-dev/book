//! Repairing, then trying once more: when the charge fails because the card has expired, its
//! rescue step asks the customer's bank for the new card, and the charge runs once more.

use itinera::error::Error;
use itinera::event::{Event, EventBody};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::report::Reporter;
use itinera::step::{Outcome, Reason, RescueOutcome, RescueStep, Step, rescue_step, step};
use itinera::workflow::workflow;

/// The customers' bank, faked: it knows which cards have expired, which card replaced each, and
/// which of those it refuses.
struct Bank;

impl Bank {
    /// Whether a card can be charged.
    fn accepts(card: &str) -> Result<(), &'static str> {
        match card {
            "card-2019" => Err("expired-card"),
            "card-2030" => Err("declined"),
            _ => Ok(()),
        }
    }

    /// The card the bank issued to a customer after their old one expired, if any.
    fn renewed_card(customer: &str) -> Option<String> {
        match customer {
            "Ben" => Some("card-2031".to_owned()),
            "Dan" => Some("card-2030".to_owned()),
            _ => None,
        }
    }
}

// ANCHOR: charge
/// Charges an order's price to the customer's card.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
    #[input]
    card: String,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if let Err(code) = Bank::accepts(&self.card) {
            return Self::fail(code);
        }
        contributor.contribute("charged", self.price);
        Self::succeed()
    }
}
// ANCHOR_END: charge

// ANCHOR: rescue
/// Asks the customer's bank for the card that replaced an expired one. A charge that failed for
/// any other reason, or ended in an abnormal termination, which has no reason, leaves nothing to
/// renew.
#[derive(RescueStep)]
struct RenewCard {
    #[input]
    customer: String,
    #[reason]
    reason: Option<Reason>,
}

#[rescue_step]
impl RenewCard {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<RescueOutcome, Error> {
        let renewed = match &self.reason {
            Some(reason) if reason.code() == "expired-card" => Bank::renewed_card(&self.customer),
            _ => None,
        };
        let Some(card) = renewed else {
            return Self::fail("nothing-to-renew");
        };
        contributor.contribute("card", card);
        Self::succeed()
    }
}
// ANCHOR_END: rescue

/// Ships the order.
#[derive(Step)]
struct Ship;

#[step]
impl Ship {
    fn run(self) -> Result<Outcome, Error> {
        Self::succeed()
    }
}

/// Prints the attempts, the rescue and how the journey ended, one line each.
#[derive(Default)]
struct Printer;

impl Reporter for Printer {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        let kind = event.kind();
        match &event.body {
            EventBody::AttemptStarted { step, .. } | EventBody::StepSucceeded { step, .. } => {
                println!("  {kind} {} {}", step.step, step.attempt);
            }
            EventBody::StepFailed { step, reason, .. } => {
                println!("  {kind} {} {}: {}", step.step, step.attempt, reason.code());
            }
            EventBody::StepRescuing {
                step,
                cause,
                rescue,
                ..
            } => {
                println!(
                    "  {kind} {} {}: {cause}, by {rescue}",
                    step.step, step.attempt
                );
            }
            EventBody::StepRescued { step, rescue, .. } => {
                println!("  {kind} {} {}, by {rescue}", step.step, step.attempt);
            }
            EventBody::StepGivenUp { step, cause, .. } => {
                println!("  {kind} {} {}: {cause}", step.step, step.attempt);
            }
            EventBody::ContributionCommitted { key, .. } => println!("  {kind} {key}"),
            EventBody::JourneySucceeded { .. } => println!("  {kind}"),
            EventBody::JourneyFailed { step, failure, .. } => {
                println!("  {kind} at {step}: {}", failure.cause());
            }
            _ => {}
        }
        Ok(())
    }
}

// ANCHOR: workflow
/// The shop's workflow: a charge that fails is rescued once, then the order is shipped.
#[workflow(name = "orders", reporters = [Printer])]
#[with_step(Charge, name = "charge", rescue = RenewCard)]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    price: i64,
    card: String,
}

/// What the shop reads back once the journey has ended.
#[derive(Output)]
struct Charged {
    charged: Option<i64>,
    card: String,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        NewOrder {
            order: "A-2".to_owned(),
            customer: "Ben".to_owned(),
            price: 40,
            card: "card-2019".to_owned(),
        },
        NewOrder {
            order: "A-4".to_owned(),
            customer: "Dan".to_owned(),
            price: 25,
            card: "card-2019".to_owned(),
        },
    ];
    let mut executor = LocalExecutor::new();
    for order in orders {
        println!("order {}", order.order);
        let instance = Orders::builder().initial_data(order).create()?;
        let result = executor.run(instance)?;
        let charged: Charged = result.output()?;
        println!("the journey {}", result.status.kind());
        println!("card {}", charged.card);
        match charged.charged {
            Some(amount) => println!("charged {amount}"),
            None => println!("nothing charged"),
        }
    }
    Ok(())
}
// ANCHOR_END: run
