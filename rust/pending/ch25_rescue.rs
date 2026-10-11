//! The repair of an expired card where it belongs: in a rescue step of the charge.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, Failure, InitialData, JourneyStatus};
use itinera::policy::{FailWorkflow, step_policy};
use itinera::step::{Outcome, Reason, Step, step};
use itinera::workflow::workflow;

/// What the customer's bank answers to a charge.
enum BankAnswer {
    Paid,
    Expired,
    Declined,
}

/// The customer's bank, faked: it answers from the card alone, and one card's bank is offline.
fn bank(card: &str) -> Result<BankAnswer, Error> {
    match card {
        "card-expired" => Ok(BankAnswer::Expired),
        "card-stolen" => Ok(BankAnswer::Declined),
        "card-offline" => Err(Error::msg("the payment network is down")),
        _ => Ok(BankAnswer::Paid),
    }
}

/// The customer's bank, faked: the card that replaces an expired one.
fn renewed_card(card: &str) -> String {
    card.replace("expired", "renewed")
}

/// Charges an order's price to the customer's card, and says how it went.
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
        match bank(&self.card)? {
            BankAnswer::Paid => {
                contributor.contribute("charged", self.price);
                Self::succeed()
            }
            BankAnswer::Expired => Self::fail("expired-card"),
            BankAnswer::Declined => Self::fail("declined"),
        }
    }
}

// ANCHOR: rescue
/// Asks the customer's bank for the renewed card, after the charge failed because the card had
/// expired. Any other failure, or an abnormal termination, which has no reason, is not its to
/// repair.
#[derive(Step)]
struct RenewCard {
    #[input]
    card: String,
    #[reason]
    reason: Option<Reason>,
}

#[step]
impl RenewCard {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.reason.as_ref().map(Reason::code) != Some("expired-card") {
            return Self::fail("not-repairable");
        }
        contributor.contribute("card", renewed_card(&self.card));
        Self::succeed()
    }
}
// ANCHOR_END: rescue

/// Gives up on the order at once when the payment network is down, instead of rescuing it.
#[derive(Default)]
struct Outage;

#[step_policy(name = "outage")]
impl Outage {
    #[on_step_abnormal_termination]
    fn gave_up(&self) -> Result<Option<FailWorkflow>, Error> {
        Self::fail_workflow("payments-down")
    }
}

// ANCHOR: workflow
/// The shop's workflow: the charge has a rescue step, and so no retry budget.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge", rescue = RenewCard, policies = [Outage])]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    price: i64,
    card: String,
}

/// How a journey ended, with the reason a step or hook gave when it failed.
fn described(status: &JourneyStatus) -> String {
    match status {
        JourneyStatus::Failed {
            failure: failure @ (Failure::Failure(reason) | Failure::FailWorkflow(reason)),
            ..
        } => format!("failed, {}: {}", failure.cause(), reason.code()),
        _ => status.kind().to_string(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        ("A-3", "card-expired"),
        ("A-4", "card-offline"),
        ("A-5", "card-stolen"),
    ];
    let mut executor = LocalExecutor::new();
    for (order, card) in orders {
        let instance = Orders::builder()
            .initial_data(NewOrder {
                price: 40,
                card: card.to_owned(),
            })
            .create()?;
        let result = executor.run(instance)?;
        println!("{order}: {}", described(&result.status));
    }
    Ok(())
}
