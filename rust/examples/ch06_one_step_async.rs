//! The workflow of one step, run on the asynchronous executor.

use itinera::error::Error;
use itinera::executor::AsyncLocalExecutor;
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
    async fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        contributor.contribute("charged", self.price);
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

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Charged {
    charged: i64,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = Orders::builder()
        .initial_data(NewOrder { price: 40 })
        .create()?;
    let result = futures::executor::block_on(AsyncLocalExecutor::new().run(instance))?;
    println!("the journey {}", result.status.kind());
    let charged: Charged = result.output()?;
    println!("charged {}", charged.charged);
    Ok(())
}
// ANCHOR_END: run
