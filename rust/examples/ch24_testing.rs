//! Testing the shop: the charge step alone, the notify policy alone, and a whole journey by its
//! result and its events.

use itinera::error::Error;
use itinera::event::{Event, EventBody, HookSource};
use itinera::executor::LocalExecutor;
use itinera::journey::{
    Contributions, Contributor, Failure, InitialData, JourneyResult, JourneyStatus, Output, Read,
};
use itinera::policy::workflow_policy;
use itinera::report::RecordingReporter;
use itinera::step::{Outcome, Reason, Step, step};
use itinera::workflow::workflow;

// ANCHOR: shop
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

/// Ships an order.
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

/// The shop's workflow: charge, then ship, then write the customer's message.
#[workflow(name = "orders", policies = [Notify])]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;

/// What the shop knows about an order when its journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
}
// ANCHOR_END: shop

/// Prints what was checked, or fails the program when it does not hold.
fn check(what: &str, holds: bool) -> Result<(), Error> {
    if !holds {
        return Err(Error::msg(format!("failed: {what}")));
    }
    println!("ok: {what}");
    Ok(())
}

/// Whether a journey failed because a step reported a failure with this code.
fn failed_with(result: &JourneyResult, code: &str) -> bool {
    match &result.status {
        JourneyStatus::Failed {
            failure: Failure::Failure(reason),
            ..
        } => reason.code() == code,
        _ => false,
    }
}

/// What a test reads back about the customer's message.
#[derive(Output)]
struct Told {
    message: String,
}

// ANCHOR: step_alone
/// Runs the charge step for a price, outside any journey, with a contributor lent by the test.
fn charge(price: i64, contributions: &mut Contributions) -> Result<Outcome, Error> {
    Charge { price }.run(contributions.contributor())
}

fn the_charge_step_alone() -> Result<(), Error> {
    let mut contributions = Contributions::default();
    let outcome = charge(40, &mut contributions)?;
    check(
        "the charge step succeeds for a price of 40",
        outcome.is_success(),
    )?;
    check(
        "the charge step contributes the 40 it charged",
        contributions.read::<i64>("charged") == Read::Present(40),
    )?;

    let mut contributions = Contributions::default();
    let outcome = charge(0, &mut contributions)?;
    check(
        "the charge step fails a price of 0 with nothing-to-charge",
        outcome.reason().map(Reason::code) == Some("nothing-to-charge"),
    )?;
    check(
        "the charge step contributes nothing for it",
        contributions.keys().count() == 0,
    )?;
    Ok(())
}
// ANCHOR_END: step_alone

// ANCHOR: policy_alone
/// The message a hook of the notify policy wrote, or nothing.
fn message(contributions: &Contributions) -> Read<String> {
    contributions.read("message")
}

fn the_notify_policy_alone() -> Result<(), Error> {
    let mut contributions = Contributions::default();
    Notify.shipped(
        "A-1".to_owned(),
        Some("T-A-1".to_owned()),
        contributions.contributor(),
    )?;
    check(
        "notify tells the customer a shipped order is on its way",
        message(&contributions)
            == Read::Present("Order A-1 is on its way, tracking T-A-1.".to_owned()),
    )?;

    let mut contributions = Contributions::default();
    Notify.failed("A-2".to_owned(), contributions.contributor())?;
    check(
        "notify tells the customer a failed order could not be taken",
        message(&contributions) == Read::Present("We could not take order A-2.".to_owned()),
    )?;
    Ok(())
}
// ANCHOR_END: policy_alone

// ANCHOR: journey
/// An event's kind, and the step or the policy and hook it is about, for the kinds this test
/// looks at.
fn line(event: &Event) -> String {
    let about = match &event.body {
        EventBody::AttemptStarted { step, .. }
        | EventBody::StepFailed { step, .. }
        | EventBody::StepGivenUp { step, .. } => step.step.to_string(),
        EventBody::HookCalled {
            hook: HookSource::Workflow { policy, hook, .. },
            ..
        } => format!("{policy} {hook}"),
        EventBody::JourneyFailed { step, .. } => step.to_string(),
        _ => return event.kind().to_owned(),
    };
    format!("{} {about}", event.kind())
}

fn a_free_orders_journey() -> Result<(), Error> {
    let recorder = RecordingReporter::default();
    let instance = Orders::builder()
        .initial_data(NewOrder {
            order: "A-2".to_owned(),
            price: 0,
        })
        .reporter(recorder.clone())
        .create()?;
    let result = LocalExecutor::new().run(instance)?;

    check(
        "a free order's journey fails with nothing-to-charge",
        failed_with(&result, "nothing-to-charge"),
    )?;
    let lines: Vec<String> = recorder.events().iter().map(line).collect();
    let expected = [
        "journey_started",
        "attempt_started charge",
        "step_failed charge",
        "step_given_up charge",
        "hook_called notify on workflow failure",
        "contribution_committed",
        "journey_failed charge",
    ];
    check(
        "a free order is never shipped, as its events show",
        lines == expected,
    )?;
    let told: Told = result.output()?;
    check(
        "a free order's customer is told",
        told.message == "We could not take order A-2.",
    )?;
    Ok(())
}
// ANCHOR_END: journey

// ANCHOR: main
fn main() -> Result<(), Error> {
    the_charge_step_alone()?;
    the_notify_policy_alone()?;
    a_free_orders_journey()?;
    Ok(())
}
// ANCHOR_END: main
