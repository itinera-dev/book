//! The shop declared in the short form: a step, a policy of each kind, a role and the workflow.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::policy::{OnSuccess, step_policy, workflow_policy};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::{ListedStep, Workflow, role, workflow};

// ANCHOR: step
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

/// Ships an order through the carrier.
#[derive(Step)]
struct Ship;

#[step]
impl Ship {
    fn run(self) -> Result<Outcome, Error> {
        Self::succeed()
    }
}
// ANCHOR_END: step

// ANCHOR: policies
/// Audits every successful charge.
#[derive(Default)]
struct Audit;

#[step_policy(name = "audit")]
impl Audit {
    #[on_step_success]
    fn audited(
        &self,
        #[from_step] charged: i64,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<OnSuccess>, Error> {
        contributor.contribute("audit", format!("charged {charged}"));
        Self::carry_on()
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
        #[from_workflow] order: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, &format!("order {order} is complete"));
        Ok(())
    }
}
// ANCHOR_END: policies

// ANCHOR: workflow
/// The shop's workflow.
#[workflow(name = "orders", policies = [Notify])]
#[with_step(Charge, name = "charge", policies = [Audit])]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders;

#[role]
impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        println!("to {customer}: {message}");
    }
}
// ANCHOR_END: workflow

// ANCHOR: data
/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    price: i64,
}

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Receipt {
    charged: i64,
    audit: String,
}
// ANCHOR_END: data

/// One line of the workflow's listing: the step's position, its name and its policies.
fn line(listed: &ListedStep) -> String {
    let policies: Vec<String> = listed.policies.iter().map(ToString::to_string).collect();
    if policies.is_empty() {
        return format!("{}. {}", listed.position, listed.step);
    }
    format!(
        "{}. {}, with {}",
        listed.position,
        listed.step,
        policies.join(", ")
    )
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for listed in Orders::descriptor().listing() {
        println!("{}", line(&listed));
    }
    let instance = Orders::builder()
        .initial_data(NewOrder {
            order: "A-1".to_owned(),
            customer: "Ann".to_owned(),
            price: 40,
        })
        .create()?;
    let result = LocalExecutor::new().run(instance)?;
    let receipt: Receipt = result.output()?;
    println!("the journey {}", result.status.kind());
    println!("charged {}; audit: {}", receipt.charged, receipt.audit);
    Ok(())
}
// ANCHOR_END: run
