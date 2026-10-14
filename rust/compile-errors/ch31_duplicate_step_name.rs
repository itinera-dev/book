//! Two steps of one workflow under the same name.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// Charges an order.
#[derive(Step)]
struct Charge;

#[step]
impl Charge {
    fn run(self) -> Result<Outcome, Error> {
        Self::succeed()
    }
}

/// Ships an order through the carrier.
#[derive(Step)]
struct Ship;

#[step]
impl Ship {
    fn run(self) -> Result<Outcome, Error> {
        Self::succeed()
    }
}

/// The shop's workflow, with `ship` listed twice.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = LocalExecutor::new().run(Orders::builder().create()?)?;
    println!("the journey {}", result.status.kind());
    Ok(())
}
