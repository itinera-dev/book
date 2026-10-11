//! Testing the shop: the charge step alone, the notify policy alone, and a whole journey by its
//! result and its events, against a fake outbox instead of the real email service.

use std::sync::mpsc::{Receiver, Sender, channel};

use itinera::error::Error;
use itinera::event::{Event, EventBody, HookSource};
use itinera::executor::LocalExecutor;
use itinera::journey::{
    Contributor, DataBag, Failure, InitialData, JourneyId, JourneyResult, JourneyStatus, Output,
};
use itinera::policy::workflow_policy;
use itinera::report::Reporter;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::{role, workflow};

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

/// Ships an order through the carrier.
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

/// The role the shop provides to policies: telling a customer something.
trait Messages {
    fn tell(&self, customer: &str, message: &str);
}

/// Tells the customer how the order went.
#[derive(Default)]
struct Notify;

#[workflow_policy(name = "notify")]
impl Notify {
    #[on_workflow_success]
    fn paid(
        &self,
        #[journey_id] order: &JourneyId,
        #[from_workflow] customer: String,
        #[from_workflow] charged: i64,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, &format!("{order}: paid {charged}"));
        Ok(())
    }

    #[on_workflow_failure]
    fn sorry(
        &self,
        #[journey_id] order: &JourneyId,
        #[from_workflow] customer: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(
            &customer,
            &format!("{order}: we could not complete your order"),
        );
        Ok(())
    }
}

/// Where the shop's messages go: an email service in production.
trait Outbox: Send + Sync {
    fn send(&self, to: &str, text: &str);
}

/// The shop's workflow: charge, then ship, then tell the customer, through its outbox.
#[workflow(name = "orders", policies = [Notify])]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders {
    outbox: Box<dyn Outbox>,
}

#[role]
impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        self.outbox.send(customer, message);
    }
}

#[workflow]
impl Orders {
    #[id_generator]
    fn order_id(&self, data: &DataBag) -> Result<String, Error> {
        Ok(data.required("order")?)
    }
}

/// What the shop knows about an order when its journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    price: i64,
}
// ANCHOR_END: shop

// ANCHOR: fakes
/// An outbox that sends nothing, and keeps every text for the test to read. It fails the test,
/// by panicking, if the test has stopped reading.
struct FakeOutbox {
    sent: Sender<String>,
}

impl Outbox for FakeOutbox {
    fn send(&self, to: &str, text: &str) {
        let kept = self.sent.send(format!("to {to}: {text}"));
        assert!(kept.is_ok(), "the test stopped reading the outbox");
    }
}

/// A reporter that keeps a line for each event: its kind, and what it is about.
struct Recorder {
    lines: Sender<String>,
}

impl Reporter for Recorder {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        let line = match about(&event.body) {
            Some(about) => format!("{} {about}", event.kind()),
            None => event.kind().to_owned(),
        };
        Ok(self.lines.send(line)?)
    }
}

/// The step, or the policy and hook, an event is about, for the kinds these tests look at.
fn about(body: &EventBody) -> Option<String> {
    match body {
        EventBody::AttemptStarted { step, .. }
        | EventBody::StepSucceeded { step, .. }
        | EventBody::StepFailed { step, .. }
        | EventBody::StepGivenUp { step, .. } => Some(step.step.to_string()),
        EventBody::HookCalled {
            hook: HookSource::Workflow { policy, hook, .. },
            ..
        } => Some(format!("{policy} {hook}")),
        EventBody::JourneyFailed { step, .. } => Some(step.to_string()),
        _ => None,
    }
}

/// Everything sent so far through the other end of a channel.
fn received(receiver: &Receiver<String>) -> Vec<String> {
    receiver.try_iter().collect()
}
// ANCHOR_END: fakes

/// Prints what was checked, or fails the program when it does not hold.
fn check(what: &str, holds: bool) -> Result<(), String> {
    if !holds {
        return Err(format!("failed: {what}"));
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

/// What a test reads back about the charge.
#[derive(Output)]
struct Charged {
    charged: Option<i64>,
}

// ANCHOR: step_alone
/// The charge step alone, in a workflow of its own.
#[workflow(name = "charge alone")]
#[with_step(Charge, name = "charge")]
struct ChargeAlone;

/// The price of an order, all the charge step needs.
#[derive(InitialData)]
struct Price {
    price: i64,
}

fn charge_alone(price: i64) -> Result<JourneyResult, Box<dyn std::error::Error>> {
    let instance = ChargeAlone::builder()
        .initial_data(Price { price })
        .create()?;
    Ok(LocalExecutor::new().run(instance)?)
}

fn the_charge_step_alone() -> Result<(), Box<dyn std::error::Error>> {
    let paid = charge_alone(40)?;
    let charged: Charged = paid.output()?;
    check(
        "the charge step charges a price of 40",
        charged.charged == Some(40),
    )?;

    let free = charge_alone(0)?;
    let charged: Charged = free.output()?;
    check(
        "the charge step fails a price of 0 with nothing-to-charge",
        failed_with(&free, "nothing-to-charge"),
    )?;
    check(
        "the charge step charges nothing for it",
        charged.charged.is_none(),
    )?;
    Ok(())
}
// ANCHOR_END: step_alone

// ANCHOR: policy_alone
/// A stand-in for the charge, which succeeds or fails as the test says.
#[derive(Step)]
struct StandIn {
    #[input]
    succeeds: bool,
}

#[step]
impl StandIn {
    fn run(self) -> Result<Outcome, Error> {
        if self.succeeds {
            return Self::succeed();
        }
        Self::fail("declined")
    }
}

/// The notify policy alone, attached to a workflow of one stand-in step.
#[workflow(name = "notify alone", policies = [Notify])]
#[with_step(StandIn, name = "charge")]
struct NotifyAlone {
    outbox: Box<dyn Outbox>,
}

#[role]
impl Messages for NotifyAlone {
    fn tell(&self, customer: &str, message: &str) {
        self.outbox.send(customer, message);
    }
}

#[workflow]
impl NotifyAlone {
    #[id_generator]
    fn order_id(&self, data: &DataBag) -> Result<String, Error> {
        Ok(data.required("order")?)
    }
}

/// What the notify policy reads, and how the stand-in ends.
#[derive(InitialData)]
struct Ending {
    order: String,
    customer: String,
    charged: i64,
    succeeds: bool,
}

fn told_after(ending: Ending) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let (sent, outbox) = channel();
    let instance = NotifyAlone::builder()
        .outbox(Box::new(FakeOutbox { sent }))
        .initial_data(ending)
        .create()?;
    LocalExecutor::new().run(instance)?;
    Ok(received(&outbox))
}

fn the_notify_policy_alone() -> Result<(), Box<dyn std::error::Error>> {
    let told = told_after(Ending {
        order: "A-1".to_owned(),
        customer: "Ann".to_owned(),
        charged: 40,
        succeeds: true,
    })?;
    check(
        "notify tells the customer an order is paid",
        told == ["to Ann: A-1: paid 40"],
    )?;

    let told = told_after(Ending {
        order: "A-2".to_owned(),
        customer: "Ben".to_owned(),
        charged: 0,
        succeeds: false,
    })?;
    check(
        "notify tells the customer an order could not be completed",
        told == ["to Ben: A-2: we could not complete your order"],
    )?;
    Ok(())
}
// ANCHOR_END: policy_alone

// ANCHOR: journey
fn a_free_orders_journey() -> Result<(), Box<dyn std::error::Error>> {
    let (sent, outbox) = channel();
    let (lines, recorded) = channel();
    let instance = Orders::builder()
        .outbox(Box::new(FakeOutbox { sent }))
        .initial_data(NewOrder {
            order: "A-2".to_owned(),
            customer: "Ben".to_owned(),
            price: 0,
        })
        .reporter(Recorder { lines })
        .create()?;
    let result = LocalExecutor::new().run(instance)?;

    check(
        "a free order's journey fails with nothing-to-charge",
        failed_with(&result, "nothing-to-charge"),
    )?;
    let expected = [
        "journey_started",
        "attempt_started charge",
        "step_failed charge",
        "step_given_up charge",
        "hook_called notify on workflow failure",
        "journey_failed charge",
    ];
    check(
        "a free order is never shipped, as its events show",
        received(&recorded) == expected,
    )?;
    check(
        "a free order's customer is told",
        received(&outbox) == ["to Ben: A-2: we could not complete your order"],
    )?;
    Ok(())
}
// ANCHOR_END: journey

// ANCHOR: main
fn main() -> Result<(), Box<dyn std::error::Error>> {
    the_charge_step_alone()?;
    the_notify_policy_alone()?;
    a_free_orders_journey()?;
    Ok(())
}
// ANCHOR_END: main
