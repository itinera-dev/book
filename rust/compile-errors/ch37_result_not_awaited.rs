//! An asynchronous workflow whose journey is run without awaiting it.

use itinera::error::Result;
use itinera::executor::local_executor_for;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// Charges an order, asynchronously.
#[derive(Step)]
struct Charge;

#[step]
impl Charge {
    async fn run(self) -> Result<Outcome> {
        Self::succeed()
    }
}

/// The shop's workflow, asynchronous because its steps are.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
struct Orders;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = local_executor_for!(Orders).run(Orders::builder().create()?)?;
    println!("the journey {}", result.status.kind());
    Ok(())
}
