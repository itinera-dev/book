//! The role "tell the customer", used by an asynchronous policy on the asynchronous executor.

use itinera::error::Error;
use itinera::executor::AsyncLocalExecutor;
use itinera::journey::{Contributor, InitialData};
use itinera::policy::workflow_policy;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::{role, workflow};

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

/// Ships the order.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
}

#[step]
impl Ship {
    async fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        contributor.contribute("tracking", format!("T-{}", self.order));
        Self::succeed()
    }
}

// ANCHOR: role
/// What the shop can do for its policies: tell a customer something.
trait Messages: Sync {
    fn tell(&self, customer: &str, message: &str);
}
// ANCHOR_END: role

// ANCHOR: policy
/// Tells the customer how the order went.
#[derive(Default)]
struct Notify;

#[workflow_policy(name = "notify")]
impl Notify {
    #[on_workflow_success]
    async fn shipped(
        &self,
        #[from_workflow] customer: String,
        #[from_workflow] order: String,
        #[from_workflow] tracking: Option<String>,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        let message = match tracking {
            Some(tracking) => format!("Order {order} is on its way, tracking {tracking}."),
            None => format!("Order {order} is complete."),
        };
        messages.tell(&customer, &message);
        Ok(())
    }

    #[on_workflow_failure]
    async fn failed(
        &self,
        #[from_workflow] customer: String,
        #[from_workflow] order: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, &format!("We could not take order {order}."));
        Ok(())
    }
}
// ANCHOR_END: policy

// ANCHOR: workflow
/// The shop's workflow, asynchronous because its steps are, which tells customers through the shop's channel.
#[workflow(name = "orders", policies = [Notify])]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders {
    channel: &'static str,
}

#[role]
impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        println!("{} to {customer}: {message}", self.channel);
    }
}
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    price: i64,
}

// ANCHOR: run
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        NewOrder {
            order: "A-1".to_owned(),
            customer: "Ann".to_owned(),
            price: 40,
        },
        NewOrder {
            order: "A-2".to_owned(),
            customer: "Ben".to_owned(),
            price: 0,
        },
    ];
    let mut executor = AsyncLocalExecutor::new();
    for order in orders {
        let instance = Orders::builder()
            .channel("email")
            .initial_data(order)
            .create()?;
        let result = executor.run(instance).await?;
        println!("the journey {}", result.status.kind());
    }
    Ok(())
}
// ANCHOR_END: run
