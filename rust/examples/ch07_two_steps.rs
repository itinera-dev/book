//! Adding a step: the shop charges an order, then ships it, in the order the workflow lists them.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// The shop's bank, faked: it charges whatever it is asked to.
fn bank_charges(amount: i64) {
    println!("the bank charges {amount}");
}

/// The shop's carrier, faked: it collects the parcel and gives a tracking number.
fn carrier_collects(order: &str) -> String {
    println!("the carrier collects order {order}");
    format!("T-{order}")
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
        bank_charges(self.price);
        contributor.contribute("charged", self.price);
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
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        let tracking = carrier_collects(&self.order);
        contributor.contribute("tracking", tracking);
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

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Shipped {
    charged: i64,
    tracking: String,
}
// ANCHOR_END: data

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = Orders::builder()
        .initial_data(NewOrder {
            order: "A-1".to_owned(),
            price: 40,
        })
        .create()?;
    let result = LocalExecutor::new().run(instance)?;
    println!("the journey {}", result.status.kind());
    let shipped: Shipped = result.output()?;
    println!("charged {}, tracking {}", shipped.charged, shipped.tracking);
    Ok(())
}
// ANCHOR_END: run
