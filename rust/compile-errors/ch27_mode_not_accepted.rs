//! An asynchronous workflow handed to the synchronous executor.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// Charges an order, asynchronously.
#[derive(Step)]
struct Charge;

#[step]
impl Charge {
    async fn run(self) -> Result<Outcome, Error> {
        Self::succeed()
    }
}

/// The shop's workflow, asynchronous because its steps are.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
struct Orders;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = LocalExecutor::new().run(Orders::builder().create()?)?;
    println!("the journey {}", result.status.kind());
    Ok(())
}
