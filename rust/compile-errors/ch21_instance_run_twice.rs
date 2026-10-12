//! Running one instance twice: an instance runs one journey, so this does not compile.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::InitialData;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

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
        Self::succeed()
    }
}

/// The shop's workflow: for now, it only charges.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
struct Orders;

/// What the shop knows about an order when its journey starts.
#[derive(InitialData)]
struct NewOrder {
    price: i64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut executor = LocalExecutor::new();
    let instance = Orders::builder()
        .initial_data(NewOrder { price: 40 })
        .create()?;
    let first = executor.run(instance)?;
    let second = executor.run(instance)?;
    println!("{} {}", first.status.kind(), second.status.kind());
    Ok(())
}
