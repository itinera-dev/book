//! Saying how it went: a step succeeds, fails, fails in a way that might work next time, or skips.

use std::sync::atomic::{AtomicBool, Ordering};

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, Failure, InitialData, JourneyResult, JourneyStatus, Output};
use itinera::step::{Outcome, Reason, Step, step};
use itinera::workflow::workflow;

/// Whether the carrier is busy: it is the first time it is called, and free afterwards.
static CARRIER_BUSY: AtomicBool = AtomicBool::new(true);

/// The shop's carrier, faked: it collects a parcel and gives a tracking number, unless it is busy.
fn carrier_collects(order: &str) -> Option<String> {
    if CARRIER_BUSY.swap(false, Ordering::Relaxed) {
        println!("the carrier is busy");
        return None;
    }
    println!("the carrier collects order {order}");
    Some(format!("T-{order}"))
}

// ANCHOR: charge
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
            return Self::fail(
                Reason::new("nothing-to-charge").with_message("the order costs nothing"),
            );
        }
        contributor.contribute("charged", self.price);
        Self::succeed()
    }
}
// ANCHOR_END: charge

// ANCHOR: ship
/// Ships an order through the carrier, unless it is digital.
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
        let Some(tracking) = carrier_collects(&self.order) else {
            return Self::fail_retriably("carrier-busy");
        };
        contributor.contribute("tracking", tracking);
        Self::succeed()
    }
}
// ANCHOR_END: ship

// ANCHOR: workflow
/// The shop's workflow: it charges, then ships, trying the shipping once more if it fails
/// retriably.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    digital: bool,
}

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Shipped {
    tracking: Option<String>,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [("A-1", 40, false), ("A-2", 25, true), ("A-3", 0, false)];
    let mut executor = LocalExecutor::new();
    for (order, price, digital) in orders {
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price,
                digital,
            })
            .create()?;
        report(order, &executor.run(instance)?);
    }
    Ok(())
}
// ANCHOR_END: run

// ANCHOR: report
/// Prints how one order's journey ended.
fn report(order: &str, result: &JourneyResult) {
    match &result.status {
        JourneyStatus::Failed {
            failure: Failure::Failure(reason),
            ..
        } => println!(
            "{order}: failed, {}: {}",
            reason.code(),
            reason.message().unwrap_or_default()
        ),
        status => match result.output::<Shipped>().map(|shipped| shipped.tracking) {
            Ok(Some(tracking)) => println!("{order}: {}, tracking {tracking}", status.kind()),
            _ => println!("{order}: {}, nothing shipped", status.kind()),
        },
    }
}
// ANCHOR_END: report
