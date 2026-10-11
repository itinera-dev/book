//! Passing data on: the charge contributes a receipt that the ship step needs, and a failed
//! attempt's contributions never reach the data bag.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// The shop's carrier, faked: it books a parcel and gives a tracking number at once.
fn carrier_books(order: &str) -> String {
    println!("the carrier books order {order}");
    format!("T-{order}")
}

/// The carrier, faked: it delivers only to customers whose address it knows.
fn carrier_delivers_to(customer: &str) -> bool {
    customer == "Ada"
}

// ANCHOR: charge
/// Charges an order's price, and gives a receipt for it.
#[derive(Step)]
struct Charge {
    #[input]
    order: String,
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
        contributor.contribute("receipt", format!("R-{}", self.order));
        Self::succeed()
    }
}
// ANCHOR_END: charge

// ANCHOR: ship
/// Ships an order, with its receipt in the parcel.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    customer: String,
    #[input]
    receipt: String,
}

#[step]
impl Ship {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        let tracking = carrier_books(&self.order);
        contributor.contribute("tracking", tracking);
        if !carrier_delivers_to(&self.customer) {
            return Self::fail("no-address");
        }
        println!("the parcel leaves with receipt {}", self.receipt);
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
    customer: String,
}

// ANCHOR: output
/// What the shop reads back once the journey has ended, whether it succeeded or failed.
#[derive(Output)]
struct Order {
    charged: i64,
    receipt: String,
    tracking: Option<String>,
}
// ANCHOR_END: output

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut executor = LocalExecutor::new();
    for (order, customer) in [("A-1", "Ada"), ("A-2", "Bo")] {
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price: 40,
                customer: customer.to_owned(),
            })
            .create()?;
        let result = executor.run(instance)?;
        let data: Order = result.output()?;
        let tracking = data.tracking.as_deref().unwrap_or("none");
        println!(
            "{order}: {}, charged {}, receipt {}, tracking {tracking}",
            result.status.kind(),
            data.charged,
            data.receipt,
        );
    }
    Ok(())
}
// ANCHOR_END: run
