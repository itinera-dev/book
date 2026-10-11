//! Ending a journey early: a digital order finishes once it is paid, and an order whose payment
//! network is down fails at once.

use std::fmt;

use itinera::error::Error;
use itinera::event::{Event, EventBody, GiveUpCause, HookSource};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, Failure, InitialData, JourneyStatus, Output};
use itinera::policy::{FailWorkflow, OnSuccess, step_policy};
use itinera::report::Reporter;
use itinera::step::{Outcome, Reason, Step, step};
use itinera::workflow::workflow;

/// The payment network could not reach the customer's bank.
#[derive(Debug)]
struct NetworkDown;

impl fmt::Display for NetworkDown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the payment network is down")
    }
}

impl std::error::Error for NetworkDown {}

/// The payment network. Today it cannot reach Cat's bank.
fn take_payment(customer: &str, amount: i64) -> Result<i64, NetworkDown> {
    if customer == "Cat" {
        return Err(NetworkDown);
    }
    Ok(amount)
}

// ANCHOR: charge
/// Charges an order's price. Paying for a digital order unlocks its download.
#[derive(Step)]
struct Charge {
    #[input]
    order: String,
    #[input]
    customer: String,
    #[input]
    price: i64,
    #[input]
    digital: bool,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        let charged = take_payment(&self.customer, self.price)?;
        contributor.contribute("charged", charged);
        if self.digital {
            contributor.contribute("download", format!("D-{}", self.order));
        }
        Self::succeed()
    }
}
// ANCHOR_END: charge

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

// ANCHOR: downloads
/// Finishes the journey once a digital order is paid: there is nothing to ship.
#[derive(Default)]
struct Downloads;

#[step_policy(name = "downloads")]
impl Downloads {
    #[on_step_success]
    fn paid(&self, #[from_step] download: Option<String>) -> Result<Option<OnSuccess>, Error> {
        match download {
            Some(_) => Self::finish_workflow(),
            None => Self::carry_on(),
        }
    }
}
// ANCHOR_END: downloads

// ANCHOR: outage
/// Gives up at once when the charge ends in an abnormal termination, which in this shop only the
/// payment network causes.
#[derive(Default)]
struct Outage;

#[step_policy(name = "outage")]
impl Outage {
    #[on_step_abnormal_termination]
    fn gave_up(&self, #[error] error: &Error) -> Result<Option<FailWorkflow>, Error> {
        Self::fail_workflow(Reason::new("payment-network-down").with_message(error.to_string()))
    }
}
// ANCHOR_END: outage

/// Prints the attempts, the hooks and how the journey ended, one line each.
#[derive(Default)]
struct Printer;

impl Reporter for Printer {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        let kind = event.kind();
        match &event.body {
            EventBody::AttemptStarted { step, .. } | EventBody::StepSucceeded { step, .. } => {
                println!("  {kind} {} {}", step.step, step.attempt);
            }
            EventBody::StepAbnormalTermination { step, message, .. } => {
                println!("  {kind} {} {}: {message}", step.step, step.attempt);
            }
            EventBody::HookCalled {
                hook, lifecycle, ..
            } => {
                let returned = lifecycle
                    .as_ref()
                    .map_or("nothing".to_owned(), ToString::to_string);
                println!("  {kind} {}: returned {returned}", hook_name(hook));
            }
            EventBody::StepGivenUp { step, cause, .. } => {
                println!(
                    "  {kind} {} {}: {}",
                    step.step,
                    step.attempt,
                    given_up(cause)
                );
            }
            EventBody::JourneySucceeded { decided_by, .. } => match decided_by {
                Some(policy) => println!("  {kind}, decided by {policy}"),
                None => println!("  {kind}, decided by default"),
            },
            EventBody::JourneyFailed { step, failure, .. } => {
                println!("  {kind} at {step}: {}", failure.cause());
            }
            _ => {}
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

/// Why a step was given up, and who decided it.
fn given_up(cause: &GiveUpCause) -> String {
    match cause {
        GiveUpCause::FailWorkflow { decided_by, .. } => {
            format!(
                "{cause}, decided by {} {}",
                decided_by.policy, decided_by.hook
            )
        }
        _ => format!("{cause}, decided by default"),
    }
}

// ANCHOR: workflow
/// The shop's workflow: the charge's policies may end the journey before shipping.
#[workflow(name = "orders", reporters = [Printer])]
#[with_step(Charge, name = "charge", policies = [Downloads, Outage])]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    price: i64,
    digital: bool,
}

/// What the shop reads back once the journey has ended.
#[derive(Output)]
struct Delivery {
    download: Option<String>,
    tracking: Option<String>,
}

/// How a journey ended, with the reason when a policy failed it.
fn ending(status: &JourneyStatus) -> String {
    match status {
        JourneyStatus::Failed {
            failure: Failure::FailWorkflow(reason),
            ..
        } => format!("failed: {}", reason.code()),
        other => other.kind().to_string(),
    }
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        NewOrder {
            order: "A-1".to_owned(),
            customer: "Ann".to_owned(),
            price: 40,
            digital: false,
        },
        NewOrder {
            order: "A-2".to_owned(),
            customer: "Ben".to_owned(),
            price: 15,
            digital: true,
        },
        NewOrder {
            order: "A-3".to_owned(),
            customer: "Cat".to_owned(),
            price: 25,
            digital: false,
        },
    ];
    let mut executor = LocalExecutor::new();
    for order in orders {
        println!("order {}", order.order);
        let instance = Orders::builder().initial_data(order).create()?;
        let result = executor.run(instance)?;
        let delivery: Delivery = result.output()?;
        println!("the journey {}", ending(&result.status));
        if let Some(download) = delivery.download {
            println!("download {download}");
        }
        if let Some(tracking) = delivery.tracking {
            println!("tracking {tracking}");
        }
    }
    Ok(())
}
// ANCHOR_END: run
