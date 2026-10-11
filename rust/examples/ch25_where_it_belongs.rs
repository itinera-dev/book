//! Each decision of the shop where it belongs: business rules in steps, flow in policies.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, Failure, InitialData, JourneyStatus};
use itinera::policy::{FailWorkflow, OnSuccess, step_policy, workflow_policy};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::{role, workflow};

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

// ANCHOR: charge
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
// ANCHOR_END: charge

// ANCHOR: ship
/// Ships an order through the carrier. A digital order has nothing to ship.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    digital: bool,
}

#[step]
impl Ship {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.digital {
            return Self::skip_because("digital");
        }
        contributor.contribute("tracking", format!("T-{}", self.order));
        Self::succeed()
    }
}
// ANCHOR_END: ship

// ANCHOR: policies
/// Audits every successful charge.
#[derive(Default)]
struct Audit;

#[step_policy(name = "audit")]
impl Audit {
    #[on_step_success]
    fn audited(
        &self,
        #[from_step] charged: i64,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<OnSuccess>, Error> {
        contributor.contribute("audit", format!("charged {charged}"));
        Self::carry_on()
    }
}

/// Gives up on the order at once when the payment network is down.
#[derive(Default)]
struct Outage;

#[step_policy(name = "outage")]
impl Outage {
    #[on_step_abnormal_termination]
    fn gave_up(&self) -> Result<Option<FailWorkflow>, Error> {
        Self::fail_workflow("payment-network-down")
    }
}

/// The shop's way of telling a customer something.
trait Messages {
    fn tell(&self, customer: &str, message: &str);
}

/// Tells the customer how the order went.
#[derive(Default)]
struct Notify;

#[workflow_policy(name = "notify")]
impl Notify {
    #[on_workflow_success]
    fn completed(
        &self,
        #[from_workflow] customer: String,
        #[from_workflow] order: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, &format!("order {order} is complete"));
        Ok(())
    }

    #[on_workflow_failure]
    fn not_completed(
        &self,
        #[from_workflow] customer: String,
        #[from_workflow] order: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, &format!("we could not complete order {order}"));
        Ok(())
    }
}
// ANCHOR_END: policies

// ANCHOR: workflow
/// The shop's workflow: the plan, and where each flow decision is attached.
#[workflow(name = "orders", policies = [Notify])]
#[with_step(Charge, name = "charge", policies = [Audit, Outage])]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders;

#[role]
impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        println!("to {customer}: {message}");
    }
}
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    price: i64,
    card: String,
    digital: bool,
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

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        ("A-1", "Ann", "card-ok", false),
        ("A-2", "Ben", "card-ok", true),
        ("A-3", "Cat", "card-expired", false),
        ("A-4", "Dan", "card-offline", false),
    ];
    let mut executor = LocalExecutor::new();
    for (order, customer, card, digital) in orders {
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                customer: customer.to_owned(),
                price: 40,
                card: card.to_owned(),
                digital,
            })
            .create()?;
        let result = executor.run(instance)?;
        println!("{order}: {}", described(&result.status));
    }
    Ok(())
}
// ANCHOR_END: run
