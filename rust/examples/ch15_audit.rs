//! Reacting to what a step did: a step policy audits every charge, whether it succeeded or failed.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
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
struct Ship;

#[step]
impl Ship {
    fn run(self) -> Result<Outcome, Error> {
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
    fn charged(
        &self,
        #[step_name] step: StepName,
        #[from_step] charged: i64,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<OnSuccess>, Error> {
        contributor.contribute("audit", format!("{step} took {charged}"));
        Self::carry_on()
    }

    #[on_step_failure]
    fn declined(
        &self,
        #[step_name] step: StepName,
        #[reason] reason: Reason,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<FailWorkflow>, Error> {
        contributor.contribute("audit", format!("{step} declined: {}", reason.code()));
        Self::carry_on()
    }
}
// ANCHOR_END: policy

// ANCHOR: workflow
/// The shop's workflow: every charge is audited.
#[workflow(name = "orders")]
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
        println!("order {}", order.order);
        let instance = Orders::builder().initial_data(order).create()?;
        let result = executor.run(instance)?;
        let audited: Audited = result.output()?;
        println!("the journey {}", result.status.kind());
        println!("audit: {}", audited.audit);
    }
    Ok(())
}
// ANCHOR_END: run
