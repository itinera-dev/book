//! Passing data on: the charge gives back a receipt that the shipping puts in the parcel, and a
//! failed attempt's contributions never reach the data bag.

use itinera::error::Result;
use itinera::executor::local_executor_for;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// The shop's bank, faked: it prints what it charges, and gives a receipt for it.
fn bank_charges(order: &str, amount: i64) -> String {
    println!("the bank charges {amount} for order {order}");
    format!("R-{order}")
}

/// The shop's carrier, faked: it books any parcel, and gives its tracking number at once.
fn carrier_books(order: &str) -> String {
    println!("the carrier books order {order}");
    format!("T-{order}")
}

/// The carrier, faked: it weighs a parcel when it comes to collect it, and takes it only up to
/// 20 kg.
fn carrier_collects(weight: i64) -> bool {
    weight <= 20
}

// ANCHOR: charge
/// Charges an order's price, and gives back the bank's receipt.
#[derive(Step)]
struct Charge {
    #[input]
    order: String,
    #[input]
    price: i64,
}

#[step]
impl Charge {
    fn run(self, contributor: &mut Contributor) -> Result<Outcome> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        let receipt = bank_charges(&self.order, self.price);
        contributor.contribute("receipt", receipt);
        Self::succeed()
    }
}
// ANCHOR_END: charge

// ANCHOR: ship
/// Ships an order, with its receipt in the parcel, and gives back its tracking number.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    weight: i64,
    #[input]
    receipt: String,
}

#[step]
impl Ship {
    fn run(self, contributor: &mut Contributor) -> Result<Outcome> {
        let tracking = carrier_books(&self.order);
        contributor.contribute("tracking", tracking);
        if !carrier_collects(self.weight) {
            return Self::fail("too-heavy");
        }
        println!("the parcel leaves with receipt {}", self.receipt);
        Self::succeed()
    }
}
// ANCHOR_END: ship

// ANCHOR: workflow
/// The shop's workflow: it charges, then ships.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: workflow

// ANCHOR: data
/// What the shop knows about an order when its journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    weight: i64,
}
// ANCHOR_END: data

// ANCHOR: output
/// What the shop reads back once the journey has ended.
#[derive(Output)]
struct Order {
    receipt: String,
    tracking: Option<String>,
}
// ANCHOR_END: output

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The order, the customer, the price and the parcel's weight in kg.
    for (order, customer, price, weight) in [("A-1", "Ann", 40, 2), ("A-7", "Gil", 45, 60)] {
        println!("{customer}'s order:");
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price,
                weight,
            })
            .create()?;
        let result = local_executor_for!(Orders).run(instance)?;
        let data: Order = result.output()?;
        let tracking = data.tracking.as_deref().unwrap_or("none");
        println!(
            "the journey {}, with receipt {} and tracking {tracking}",
            result.status.kind(),
            data.receipt,
        );
    }
    Ok(())
}
// ANCHOR_END: run
