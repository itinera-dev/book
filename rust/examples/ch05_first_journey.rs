//! A first journey: the shop runs the charge for two orders, and the result says how each
//! journey ended.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::InitialData;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// The shop's bank, faked: it prints what it charges.
fn bank_charges(amount: i64) {
    println!("the bank charges {amount}");
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

// ANCHOR: workflow
/// The shop's workflow: for now, it only charges.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
struct Orders;
// ANCHOR_END: workflow

// ANCHOR: data
/// What the shop knows about an order when its journey starts.
#[derive(InitialData)]
struct NewOrder {
    price: i64,
}
// ANCHOR_END: data

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The customer, and the price of their order.
    for (customer, price) in [("Ann", 40), ("Hal", 0)] {
        println!("{customer}'s order:");
        let instance = Orders::builder()
            .initial_data(NewOrder { price })
            .create()?;
        let result = LocalExecutor::new().run(instance)?;
        println!("the journey {}", result.status.kind());
    }
    Ok(())
}
// ANCHOR_END: run
