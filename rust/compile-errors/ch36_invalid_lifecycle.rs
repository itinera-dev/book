//! A hook that returns a lifecycle it may not return: finishing the journey after a failure.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::policy::{FailWorkflow, step_policy};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// Charges an order, and fails when there is nothing to charge.
#[derive(Step)]
struct Charge;

#[step]
impl Charge {
    fn run(self) -> Result<Outcome, Error> {
        Self::fail("nothing-to-charge")
    }
}

/// Tries to end the journey as succeeded when the charge fails.
#[derive(Default)]
struct Forgive;

#[step_policy(name = "forgive")]
impl Forgive {
    #[on_step_failure]
    fn forgiven(&self) -> Result<Option<FailWorkflow>, Error> {
        Self::finish_workflow()
    }
}

/// The shop's workflow.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge", policies = [Forgive])]
struct Orders;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = LocalExecutor::new().run(Orders::builder().create()?)?;
    println!("the journey {}", result.status.kind());
    Ok(())
}
