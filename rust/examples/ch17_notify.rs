//! Doing something when the journey ends: a workflow policy writes the message for the customer.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::policy::workflow_policy;
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
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        contributor.contribute("charged", self.price);
        Self::succeed()
    }
}

/// Ships the order.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
}

#[step]
impl Ship {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        contributor.contribute("tracking", format!("T-{}", self.order));
        Self::succeed()
    }
}

// ANCHOR: policy
/// Writes how the order went, for the customer, under `message`.
#[derive(Default)]
struct Notify;

#[workflow_policy(name = "notify")]
impl Notify {
    #[on_workflow_success]
    fn shipped(
        &self,
        #[from_workflow] order: String,
        #[from_workflow] tracking: Option<String>,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<(), Error> {
        let message = match tracking {
            Some(tracking) => format!("Order {order} is on its way, tracking {tracking}."),
            None => format!("Order {order} is complete."),
        };
        contributor.contribute("message", message);
        Ok(())
    }

    #[on_workflow_failure]
    fn failed(
        &self,
        #[from_workflow] order: String,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<(), Error> {
        let message = format!("We could not take order {order}.");
        contributor.contribute("message", message);
        Ok(())
    }
}
// ANCHOR_END: policy

// ANCHOR: workflow
/// The shop's workflow: once a journey ends, the customer gets a message.
#[workflow(name = "orders", policies = [Notify])]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
}

/// The message for the customer, read once the journey has ended.
#[derive(Output)]
struct Told {
    message: String,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        NewOrder {
            order: "A-1".to_owned(),
            price: 40,
        },
        NewOrder {
            order: "A-2".to_owned(),
            price: 0,
        },
    ];
    let mut executor = LocalExecutor::new();
    for order in orders {
        let instance = Orders::builder().initial_data(order).create()?;
        let result = executor.run(instance)?;
        let told: Told = result.output()?;
        println!("the journey {}", result.status.kind());
        println!("message: {}", told.message);
    }
    Ok(())
}
// ANCHOR_END: run
