//! A policy that needs a role the workflow does not provide.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::policy::workflow_policy;
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

/// The shop's way of telling a customer something.
trait Messages {
    fn tell(&self, customer: &str, message: &str);
}

/// Tells the customer once the order is complete.
#[derive(Default)]
struct Notify;

#[workflow_policy(name = "notify")]
impl Notify {
    #[on_workflow_success]
    fn completed(
        &self,
        #[from_workflow] customer: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, "your order is complete");
        Ok(())
    }
}

/// The shop's workflow. It can tell a customer something, but does not provide it as a role.
#[workflow(name = "orders", policies = [Notify])]
#[with_step(Charge, name = "charge")]
struct Orders;

impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        println!("to {customer}: {message}");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = LocalExecutor::new().run(Orders::builder().create()?)?;
    println!("the journey {}", result.status.kind());
    Ok(())
}
