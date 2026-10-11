//! A workflow of one step: the shop charges an order and reads the receipt from the result.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

// ANCHOR: step
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
// ANCHOR_END: step

// ANCHOR: workflow
/// The shop's workflow: for now, it only charges.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
struct Orders;
// ANCHOR_END: workflow

// ANCHOR: data
/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    price: i64,
}

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Receipt {
    charged: i64,
}
// ANCHOR_END: data

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = Orders::builder()
        .initial_data(NewOrder { price: 40 })
        .create()?;
    let result = LocalExecutor::new().run(instance)?;
    println!("the journey {}", result.status.kind());
    let receipt: Receipt = result.output()?;
    println!("charged {}", receipt.charged);
    Ok(())
}
// ANCHOR_END: run
