//! Adding a step: the shop charges an order, then ships it, in the order the workflow lists them,
//! and an order that fails its charge is never shipped.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::InitialData;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// The shop's bank, faked: it charges whatever it is asked to.
fn bank_charges(amount: i64) {
    println!("the bank charges {amount}");
}

/// The shop's carrier, faked: it prints the parcel it collects.
fn carrier_collects(order: &str) {
    println!("the carrier collects order {order}");
}

/// Charges an order's price.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
}

#[step]
impl Charge {
    fn run(self) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        bank_charges(self.price);
        Self::succeed()
    }
}

// ANCHOR: step
/// Ships an order through the carrier.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
}

#[step]
impl Ship {
    fn run(self) -> Result<Outcome, Error> {
        carrier_collects(&self.order);
        Self::succeed()
    }
}
// ANCHOR_END: step

// ANCHOR: workflow
/// The shop's workflow: it charges, then ships.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: workflow

// ANCHOR: data
/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
}
// ANCHOR_END: data

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for (order, price) in [("A-1", 40), ("A-2", 0)] {
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price,
            })
            .create()?;
        let result = LocalExecutor::new().run(instance)?;
        println!("{order}: the journey {}", result.status.kind());
    }
    Ok(())
}
// ANCHOR_END: run
