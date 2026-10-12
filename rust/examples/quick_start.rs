//! Quick start: the shop charges an order and ships it, falling back to the backup carrier when the
//! main one is down, and prints every event of the journey.

use itinera::error::Error;
use itinera::event::{Event, EventBody, HookSource};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::policy::{FailWorkflow, step_policy};
use itinera::report::Reporter;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

// ANCHOR: steps
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

/// Ships an order through the carrier it is given, or the main carrier, which is down today.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    carrier: Option<String>,
}

#[step]
impl Ship {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.carrier.as_deref() != Some("backup") {
            return Self::fail_retriably("carrier-down");
        }
        contributor.contribute("tracking", format!("B-{}", self.order));
        Self::succeed()
    }
}
// ANCHOR_END: steps

// ANCHOR: policy
/// Names the backup carrier for the next attempt of the step it is attached to.
#[derive(Default)]
struct Fallback;

#[step_policy(name = "fallback")]
impl Fallback {
    #[on_step_retry]
    fn backup_carrier(
        &self,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<FailWorkflow>, Error> {
        contributor.contribute("carrier", "backup".to_owned());
        Self::carry_on()
    }
}
// ANCHOR_END: policy

// ANCHOR: reporter
/// Prints each event of a journey on a line of its own.
#[derive(Default)]
struct Printer;

impl Reporter for Printer {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        let kind = event.kind();
        match &event.body {
            EventBody::AttemptStarted { step, .. } | EventBody::StepSucceeded { step, .. } => {
                println!("  {kind} {} {}", step.step, step.attempt);
            }
            EventBody::StepFailed {
                step,
                retriable,
                reason,
                ..
            } => {
                let retriable = if *retriable { ", retriable" } else { "" };
                let code = reason.code();
                println!("  {kind} {} {}: {code}{retriable}", step.step, step.attempt);
            }
            EventBody::HookCalled { hook, .. } => println!("  {kind} {}", hook_name(hook)),
            EventBody::ContributionCommitted { key, .. }
            | EventBody::OptionalInputAbsent { key, .. } => println!("  {kind} {key}"),
            _ => println!("  {kind}"),
        }
        Ok(())
    }
}

/// The policy and hook an event names.
fn hook_name(hook: &HookSource) -> String {
    match hook {
        HookSource::Step { policy, hook, .. } => format!("{policy} {hook}"),
        HookSource::Workflow { policy, hook, .. } => format!("{policy} {hook}"),
        _ => String::new(),
    }
}
// ANCHOR_END: reporter

// ANCHOR: workflow
/// The shop's workflow: charge, then ship, trying shipping twice with the fallback policy.
#[workflow(name = "orders", reporters = [Printer])]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship", retry_budget = 1, policies = [Fallback])]
struct Orders;
// ANCHOR_END: workflow

// ANCHOR: run
/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
}

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Shipped {
    charged: i64,
    tracking: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = Orders::builder()
        .initial_data(NewOrder {
            order: "A-1".to_owned(),
            price: 40,
        })
        .create()?;
    let result = LocalExecutor::new().run(instance)?;
    println!("the journey {}", result.status.kind());
    let shipped: Shipped = result.output()?;
    println!("charged {}, tracking {}", shipped.charged, shipped.tracking);
    Ok(())
}
// ANCHOR_END: run
