//! The workflow of one step, run on the asynchronous executor.

use itinera::error::Error;
use itinera::executor::AsyncLocalExecutor;
use itinera::journey::InitialData;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// The shop's bank, faked: it prints what it charges.
fn bank_charges(amount: i64) {
    println!("the bank charges {amount}");
}

// ANCHOR: step
/// Charges an order's price.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
}

#[step]
impl Charge {
    async fn run(self) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        bank_charges(self.price);
        Self::succeed()
    }
}
// ANCHOR_END: step

// ANCHOR: workflow
/// The shop's workflow, declared asynchronous.
#[workflow(name = "orders", asynchronous)]
#[with_step(Charge, name = "charge")]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    price: i64,
}

// ANCHOR: run
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for price in [40, 0] {
        let instance = Orders::builder()
            .initial_data(NewOrder { price })
            .create()?;
        let result = AsyncLocalExecutor::new().run(instance).await?;
        println!("price {price}: the journey {}", result.status.kind());
    }
    Ok(())
}
// ANCHOR_END: run
