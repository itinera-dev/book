//! Steps, policies and workflows: a charge step, an audit policy and the workflow that lists
//! them with the ship step, run for two orders.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, Failure, InitialData, JourneyStatus, Output};
use itinera::policy::{OnSuccess, step_policy};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

// ANCHOR: step
/// Charges an order's price, less any discount.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
    #[input]
    discount: Option<i64>,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        let amount = self.price - self.discount.unwrap_or(0);
        if amount <= 0 {
            return Self::fail("nothing-to-charge");
        }
        contributor.contribute("charged", amount);
        Self::succeed()
    }
}
// ANCHOR_END: step

/// Ships an order through the carrier, which hands back a tracking number.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
}

#[step]
impl Ship {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        contributor.contribute("tracking", format!("RAPID-{}", self.order));
        Self::succeed()
    }
}

// ANCHOR: policy
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
// ANCHOR_END: policy

// ANCHOR: workflow
/// The shop's workflow: charge, then ship.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge", policies = [Audit])]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    discount: Option<i64>,
}

/// What the shop reads back once the journey is over.
#[derive(Output)]
struct Summary {
    charged: Option<i64>,
    audit: Option<String>,
    tracking: Option<String>,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [("A-1", 40, Some(5)), ("A-2", 10, Some(10))];
    let mut executor = LocalExecutor::new();
    for (order, price, discount) in orders {
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price,
                discount,
            })
            .create()?;
        let result = executor.run(instance)?;
        match &result.status {
            JourneyStatus::Failed {
                failure: Failure::Failure(reason),
                ..
            } => println!("{order}: the journey failed, {}", reason.code()),
            status => println!("{order}: the journey {}", status.kind()),
        }
        let summary: Summary = result.output()?;
        println!("  charged: {}", shown(summary.charged));
        println!("  audit: {}", shown(summary.audit));
        println!("  tracking: {}", shown(summary.tracking));
    }
    Ok(())
}
// ANCHOR_END: run

/// A value read from the journey's output, or a dash when the journey never contributed it.
fn shown(value: Option<impl std::fmt::Display>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}
