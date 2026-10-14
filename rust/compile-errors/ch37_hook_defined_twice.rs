//! Two policies of one step that both define `on step success`.

use itinera::error::Error;
use itinera::executor::local_executor_for;
use itinera::journey::Contributor;
use itinera::policy::{OnSuccess, step_policy};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// Charges an order.
#[derive(Step)]
struct Charge;

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        contributor.contribute("charged", 40);
        Self::succeed()
    }
}

/// Audits every successful charge.
#[derive(Default)]
struct Audit;

#[step_policy(name = "audit")]
impl Audit {
    #[on_step_success]
    fn audited(&self, #[from_step] charged: i64) -> Result<Option<OnSuccess>, Error> {
        println!("audit: charged {charged}");
        Self::carry_on()
    }
}

/// Writes a receipt for every successful charge.
#[derive(Default)]
struct Receipts;

#[step_policy(name = "receipts")]
impl Receipts {
    #[on_step_success]
    fn written(&self, #[from_step] charged: i64) -> Result<Option<OnSuccess>, Error> {
        println!("receipt: R-{charged}");
        Self::carry_on()
    }
}

/// The shop's workflow, with both policies attached to the charge.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge", policies = [Audit, Receipts])]
struct Orders;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = local_executor_for!(Orders).run(Orders::builder().create()?)?;
    println!("the journey {}", result.status.kind());
    Ok(())
}
