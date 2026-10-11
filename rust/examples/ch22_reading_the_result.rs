//! Reading the result: one order succeeds, one fails and one is aborted, and the shop acts on each.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{
    Abort, Contributor, DataBag, Failure, InitialData, JourneyResult, JourneyStatus, LastFailure,
    MissingData, Output,
};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// Reserves the order's stock in the warehouse.
#[derive(Step)]
struct Reserve;

#[step]
impl Reserve {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        contributor.contribute("reserved", true);
        Self::succeed()
    }
}

/// Charges an order's price.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        contributor.contribute("charged", self.price);
        Self::succeed()
    }
}

/// The shop's workflow: reserve the stock, then charge. Each journey is named after its order.
#[workflow(name = "orders")]
#[with_step(Reserve, name = "reserve")]
#[with_step(Charge, name = "charge")]
struct Orders;

#[workflow]
impl Orders {
    #[id_generator]
    fn order_id(&self, data: &DataBag) -> Result<String, Error> {
        Ok(data.required("order")?)
    }
}

/// What the shop knows about an order when its journey starts. An order whose price has not
/// been set yet has no `price` key.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: Option<i64>,
}

// ANCHOR: output
/// What the shop reads from a journey that succeeded.
#[derive(Output)]
struct Receipt {
    charged: i64,
}

/// What the shop reads from a journey that failed: whether it reserved stock before it did.
#[derive(Output)]
struct Leftovers {
    reserved: Option<bool>,
}
// ANCHOR_END: output

// ANCHOR: read
/// Does what the shop does once an order's journey has ended.
fn follow_up(result: &JourneyResult) -> Result<(), Box<dyn std::error::Error>> {
    let order = &result.journey_id;
    match &result.status {
        JourneyStatus::Succeeded { .. } => {
            let receipt: Receipt = result.output()?;
            println!(
                "{order} succeeded: charged {}, confirm the order",
                receipt.charged
            );
        }
        JourneyStatus::Failed { failure, .. } => {
            println!("{order} failed ({}): tell the customer", why(failure));
            let leftovers: Leftovers = result.output()?;
            if leftovers.reserved == Some(true) {
                println!("{order} still holds stock: release it");
            }
        }
        JourneyStatus::Aborted(abort) => {
            println!(
                "{order} aborted ({}): call the developers",
                what_went_wrong(abort)
            );
        }
        _ => println!("{order} ended as {}", result.status.kind()),
    }
    Ok(())
}
// ANCHOR_END: read

// ANCHOR: explain
/// The cause of a failure, with the reason or error that explains it.
fn why(failure: &Failure) -> String {
    let cause = failure.cause();
    match failure {
        Failure::Failure(reason) | Failure::FailWorkflow(reason) => {
            format!("{cause}: {}", reason.code())
        }
        Failure::RetriesExhausted(LastFailure::Reason(reason)) => {
            format!("{cause}: {}", reason.code())
        }
        Failure::RetriesExhausted(LastFailure::Error(error))
        | Failure::AbnormalTermination(error) => format!("{cause}: {error}"),
        _ => cause.to_string(),
    }
}

/// The abort reason, with the key that was missing when that is what happened.
fn what_went_wrong(abort: &Abort) -> String {
    let reason = abort.reason();
    match abort {
        Abort::RequiredDataMissing(MissingData::Key { key, .. }) => format!("{reason}: {key}"),
        _ => reason.to_string(),
    }
}
// ANCHOR_END: explain

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut executor = LocalExecutor::new();
    let orders = [
        NewOrder {
            order: "A-1".to_owned(),
            price: Some(40),
        },
        NewOrder {
            order: "A-2".to_owned(),
            price: Some(0),
        },
        NewOrder {
            order: "A-3".to_owned(),
            price: None,
        },
    ];
    for order in orders {
        let instance = Orders::builder().initial_data(order).create()?;
        let result = executor.run(instance)?;
        follow_up(&result)?;
    }
    Ok(())
}
// ANCHOR_END: run
