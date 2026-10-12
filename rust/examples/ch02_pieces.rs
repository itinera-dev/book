//! Steps, policies and workflows: a charge step, a review policy and the workflow that lists
//! them with the ship step, run for three orders.

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
        contributor.contribute("tracking", format!("T-{}", self.order));
        Self::succeed()
    }
}

// ANCHOR: policy
/// Stops an order whose charge is above the order's review limit, so a person looks at it before
/// anything is shipped.
#[derive(Default)]
struct Review;

#[step_policy(name = "review")]
impl Review {
    #[on_step_success]
    fn checked(
        &self,
        #[from_step] charged: i64,
        #[from_workflow] review_limit: i64,
    ) -> Result<Option<OnSuccess>, Error> {
        if charged > review_limit {
            return Self::fail_workflow("needs-review");
        }
        Self::carry_on()
    }
}
// ANCHOR_END: policy

// ANCHOR: workflow
/// The shop's workflow: charge, then ship.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge", policies = [Review])]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    discount: Option<i64>,
    review_limit: i64,
}

/// What the shop reads back once the journey is over.
#[derive(Output)]
struct Summary {
    charged: Option<i64>,
    tracking: Option<String>,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        ("A-1", 40, Some(5)),
        ("A-2", 10, Some(10)),
        ("A-3", 900, None),
    ];
    let mut executor = LocalExecutor::new();
    for (order, price, discount) in orders {
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price,
                discount,
                review_limit: 500,
            })
            .create()?;
        let result = executor.run(instance)?;
        match &result.status {
            JourneyStatus::Failed {
                failure: Failure::Failure(reason),
                ..
            } => println!(
                "{order}: the journey failed, the step said {}",
                reason.code()
            ),
            JourneyStatus::Failed {
                failure: Failure::FailWorkflow(reason),
                ..
            } => println!("{order}: the journey failed, a hook said {}", reason.code()),
            status => println!("{order}: the journey {}", status.kind()),
        }
        let summary: Summary = result.output()?;
        println!("  charged: {}", shown(summary.charged));
        println!("  tracking: {}", shown(summary.tracking));
    }
    Ok(())
}
// ANCHOR_END: run

/// A value read from the journey's output, or a dash when the journey never contributed it.
fn shown(value: Option<impl std::fmt::Display>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}
