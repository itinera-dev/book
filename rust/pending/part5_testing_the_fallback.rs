//! With Itinera: the ship step is tested alone with one fake carrier, and the fallback to the
//! backup carrier, which belongs to the workflow's provider, is tested by the journey's events.

use std::num::NonZeroU32;
use std::sync::mpsc::{Receiver, Sender, channel};

use itinera::error::Error;
use itinera::event::{Event, EventBody};
use itinera::executor::LocalExecutor;
use itinera::journey::{
    Contributor, Failure, InitialData, JourneyResult, JourneyStatus, LastFailure, Output,
};
use itinera::report::Reporter;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// What the shop asks of a carrier: take a parcel and give back its tracking number.
trait Carrier: Send + Sync {
    /// Returns the tracking number, or nothing when the carrier is down.
    fn send(&self, order: &str) -> Option<String>;
}

// ANCHOR: fake
/// A fake carrier, up or down as the test says, whose tracking numbers start with its prefix. It
/// counts nothing.
#[derive(Clone)]
struct Parcels {
    prefix: &'static str,
    up: bool,
}

impl Carrier for Parcels {
    fn send(&self, order: &str) -> Option<String> {
        self.up.then(|| format!("{}-{order}", self.prefix))
    }
}
// ANCHOR_END: fake

/// Ships an order through the carrier it is lent.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
}

#[step]
impl Ship {
    fn run(
        self,
        #[service] carrier: &dyn Carrier,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        match carrier.send(&self.order) {
            Some(tracking) => {
                contributor.contribute("tracking", tracking);
                Self::succeed()
            }
            None => Self::fail_retriably("carrier-down"),
        }
    }
}

/// The shop's workflow: shipping is tried twice, and the workflow decides the carrier.
#[workflow(name = "orders")]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders {
    main: Parcels,
    backup: Parcels,
}

#[workflow]
impl Orders {
    /// A fresh carrier for every attempt: the main one first, the backup one after.
    #[provider(per_attempt)]
    fn carrier(&self, #[attempt] attempt: NonZeroU32) -> Result<Box<dyn Carrier>, Error> {
        if attempt.get() == 1 {
            return Ok(Box::new(self.main.clone()));
        }
        Ok(Box::new(self.backup.clone()))
    }
}

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
}

/// What a test reads back about shipping.
#[derive(Output)]
struct Shipped {
    tracking: Option<String>,
}

/// A reporter that keeps a line for each event about a step's attempt, and ignores the rest.
struct Recorder {
    lines: Sender<String>,
}

impl Reporter for Recorder {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        match &event.body {
            EventBody::AttemptStarted { step, .. }
            | EventBody::StepSucceeded { step, .. }
            | EventBody::StepFailed { step, .. }
            | EventBody::StepRetrying { step, .. } => {
                Ok(self.lines.send(format!("{} {}", event.kind(), step.step))?)
            }
            _ => Ok(()),
        }
    }
}

/// Prints what was checked, or fails the program when it does not hold.
fn check(what: &str, holds: bool) -> Result<(), String> {
    if !holds {
        return Err(format!("failed: {what}"));
    }
    println!("ok: {what}");
    Ok(())
}

/// Whether the last attempt of the step that failed the journey reported this code.
fn last_failed_with(result: &JourneyResult, code: &str) -> bool {
    match &result.status {
        JourneyStatus::Failed {
            failure: Failure::RetriesExhausted(LastFailure::Reason(reason)),
            ..
        } => reason.code() == code,
        _ => false,
    }
}

// ANCHOR: step_alone
/// The ship step alone, lent the one carrier the test gives it.
#[workflow(name = "ship alone")]
#[with_step(Ship, name = "ship")]
struct ShipAlone {
    carrier: Parcels,
}

#[workflow]
impl ShipAlone {
    #[provider]
    fn carrier(&self) -> Result<Box<dyn Carrier>, Error> {
        Ok(Box::new(self.carrier.clone()))
    }
}

fn ship_alone(up: bool) -> Result<JourneyResult, Box<dyn std::error::Error>> {
    let instance = ShipAlone::builder()
        .carrier(Parcels { prefix: "T", up })
        .initial_data(NewOrder {
            order: "A-1".to_owned(),
        })
        .create()?;
    Ok(LocalExecutor::new().run(instance)?)
}

fn the_ship_step_alone() -> Result<(), Box<dyn std::error::Error>> {
    let shipped: Shipped = ship_alone(true)?.output()?;
    check(
        "a carrier that is up gives a tracking number",
        shipped.tracking.as_deref() == Some("T-A-1"),
    )?;
    check(
        "a carrier that is down is a failure worth trying again",
        last_failed_with(&ship_alone(false)?, "carrier-down"),
    )?;
    Ok(())
}
// ANCHOR_END: step_alone

// ANCHOR: journey
fn the_fallback() -> Result<(), Box<dyn std::error::Error>> {
    let (lines, recorded) = channel();
    let instance = Orders::builder()
        .main(Parcels {
            prefix: "T",
            up: false,
        })
        .backup(Parcels {
            prefix: "B",
            up: true,
        })
        .initial_data(NewOrder {
            order: "A-1".to_owned(),
        })
        .reporter(Recorder { lines })
        .create()?;
    let shipped: Shipped = LocalExecutor::new().run(instance)?.output()?;

    check(
        "the order ships through the backup carrier",
        shipped.tracking.as_deref() == Some("B-A-1"),
    )?;
    let expected = [
        "attempt_started ship",
        "step_failed ship",
        "step_retrying ship",
        "attempt_started ship",
        "step_succeeded ship",
    ];
    check(
        "shipping is tried twice, as its events show",
        received(&recorded) == expected,
    )?;
    Ok(())
}
// ANCHOR_END: journey

/// Everything sent so far through the other end of a channel.
fn received(receiver: &Receiver<String>) -> Vec<String> {
    receiver.try_iter().collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    the_ship_step_alone()?;
    the_fallback()?;
    Ok(())
}
