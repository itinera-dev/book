//! A step that contributes something that is not a value: its connection to the bank.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::Contributor;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// A connection to the customer's bank, faked.
struct BankConnection {
    bank: &'static str,
}

/// Charges an order, and hands its bank connection on.
#[derive(Step)]
struct Charge;

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        let connection = BankConnection {
            bank: "the customer's bank",
        };
        contributor.contribute("bank", connection);
        Self::succeed()
    }
}

/// The shop's workflow.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
struct Orders;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = LocalExecutor::new().run(Orders::builder().create()?)?;
    println!("the journey {}", result.status.kind());
    Ok(())
}
