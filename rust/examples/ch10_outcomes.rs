//! Saying how it went: a step succeeds, fails, fails in a way that might work next time, or skips.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{
    Contributor, Failure, InitialData, JourneyResult, JourneyStatus, LastFailure, Output,
};
use itinera::step::{Outcome, Reason, Step, step};
use itinera::workflow::workflow;

/// The heaviest parcel the shop's carrier takes, in kg.
const CARRIER_MAX_WEIGHT: i64 = 20;

/// The shop's carrier, faked: it collects a parcel and gives a tracking number, unless the parcel
/// is too heavy for it.
fn carrier_collects(order: &str, weight: i64) -> Option<String> {
    if weight > CARRIER_MAX_WEIGHT {
        println!("the carrier cannot take a parcel of {weight} kg");
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
    #[input]
    weight: i64,
}

#[step]
impl Ship {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.digital {
            return Self::skip_because("digital");
        }
        let Some(tracking) = carrier_collects(&self.order, self.weight) else {
            return Self::fail_retriably("parcel-refused");
        };
        contributor.contribute("tracking", tracking);
        Self::succeed()
    }
}
// ANCHOR_END: ship

/// The shop's workflow: it charges, then ships.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    digital: bool,
    weight: i64,
}

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Shipped {
    tracking: Option<String>,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The order, its price, whether it is digital, and its parcel's weight in kg.
    let orders = [
        ("A-1", 40, false, 2),
        ("A-2", 25, true, 0),
        ("A-3", 30, false, 30),
        ("A-4", 0, false, 1),
    ];
    let mut executor = LocalExecutor::new();
    for (order, price, digital, weight) in orders {
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price,
                digital,
                weight,
            })
            .create()?;
        report(order, &executor.run(instance)?);
    }
    Ok(())
}
// ANCHOR_END: run

/// Prints how one order's journey ended.
fn report(order: &str, result: &JourneyResult) {
    match &result.status {
        JourneyStatus::Failed { failure, .. } => println!("{order}: failed, {}", why(failure)),
        status => match result.output::<Shipped>().map(|shipped| shipped.tracking) {
            Ok(Some(tracking)) => println!("{order}: {}, tracking {tracking}", status.kind()),
            _ => println!("{order}: {}, nothing shipped", status.kind()),
        },
    }
}

/// Why a journey failed: its cause, then the reason the step gave, with its message if it has one.
fn why(failure: &Failure) -> String {
    match failure {
        Failure::Failure(reason) | Failure::RetriesExhausted(LastFailure::Reason(reason)) => {
            match reason.message() {
                Some(message) => format!("{}, {}: {message}", failure.cause(), reason.code()),
                None => format!("{}, {}", failure.cause(), reason.code()),
            }
        }
        _ => failure.cause().to_string(),
    }
}
