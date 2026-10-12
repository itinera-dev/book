//! The audit policy, run on the asynchronous executor.

use itinera::error::Error;
use itinera::executor::AsyncLocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::policy::{FailWorkflow, OnSuccess, step_policy};
use itinera::step::{Outcome, Reason, Step, StepName, step};
use itinera::workflow::workflow;

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
struct Ship;

#[step]
impl Ship {
    async fn run(self) -> Result<Outcome, Error> {
        Self::succeed()
    }
}

// ANCHOR: policy
/// Writes down what every charge did, under `audit`.
#[derive(Default)]
struct Audit;

#[step_policy(name = "audit")]
impl Audit {
    #[on_step_success]
    async fn charged(
        &self,
        #[step_name] step: StepName,
        #[from_step] charged: i64,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<OnSuccess>, Error> {
        contributor.contribute("audit", format!("{step} took {charged}"));
        Self::carry_on()
    }

    #[on_step_failure]
    async fn failed(
        &self,
        #[step_name] step: StepName,
        #[reason] reason: Option<Reason>,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<FailWorkflow>, Error> {
        let why = reason.as_ref().map_or("no reason", Reason::code);
        contributor.contribute("audit", format!("{step} failed: {why}"));
        Self::carry_on()
    }
}
// ANCHOR_END: policy

// ANCHOR: workflow
/// The shop's workflow, declared asynchronous: every charge is audited.
#[workflow(name = "orders", asynchronous)]
#[with_step(Charge, name = "charge", policies = [Audit])]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
}

/// What the auditors read once the journey has ended.
#[derive(Output)]
struct Audited {
    audit: String,
}

// ANCHOR: run
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let mut executor = AsyncLocalExecutor::new();
    for order in orders {
        println!("order {}", order.order);
        let instance = Orders::builder().initial_data(order).create()?;
        let result = executor.run(instance).await?;
        let audited: Audited = result.output()?;
        println!("the journey {}", result.status.kind());
        println!("audit: {}", audited.audit);
    }
    Ok(())
}
// ANCHOR_END: run
