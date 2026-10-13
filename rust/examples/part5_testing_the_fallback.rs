//! With Itinera: the ship step is tested alone, called with one fake carrier, and the fallback to
//! the backup carrier, which belongs to the workflow's provider, is tested by the journey's events.

use std::num::NonZeroU32;

use itinera::error::Error;
use itinera::event::{Event, EventBody};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributions, Contributor, InitialData, Output, Read};
use itinera::report::RecordingReporter;
use itinera::service::{Provided, provide};
use itinera::step::{Outcome, Reason, Step, step};
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
    fn carrier(&self, #[attempt] attempt: NonZeroU32) -> Result<Provided<dyn Carrier>, Error> {
        if attempt.get() == 1 {
            return Ok(provide!(self.main.clone()));
        }
        Ok(provide!(self.backup.clone()))
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

/// A line for an event about a step's attempt: its kind and the step. Other events have none.
fn attempt_line(event: &Event) -> Option<String> {
    match &event.body {
        EventBody::AttemptStarted { step, .. }
        | EventBody::StepSucceeded { step, .. }
        | EventBody::StepFailed { step, .. }
        | EventBody::StepRetrying { step, .. } => Some(format!("{} {}", event.kind(), step.step)),
        _ => None,
    }
}

/// Prints what was checked, or fails the program when it does not hold.
fn check(what: &str, holds: bool) -> Result<(), Error> {
    if !holds {
        return Err(Error::msg(format!("failed: {what}")));
    }
    println!("ok: {what}");
    Ok(())
}

// ANCHOR: step_alone
/// Runs the ship step for order A-1, outside any journey, lent the carrier the test gives it.
fn ship(carrier: &Parcels, contributions: &mut Contributions) -> Result<Outcome, Error> {
    let step = Ship {
        order: "A-1".to_owned(),
    };
    step.run(carrier, contributions.contributor())
}

fn the_ship_step_alone() -> Result<(), Error> {
    let mut contributions = Contributions::default();
    ship(
        &Parcels {
            prefix: "T",
            up: true,
        },
        &mut contributions,
    )?;
    check(
        "a carrier that is up gives a tracking number",
        contributions.read::<String>("tracking") == Read::Present("T-A-1".to_owned()),
    )?;

    let down = ship(
        &Parcels {
            prefix: "T",
            up: false,
        },
        &mut Contributions::default(),
    )?;
    check(
        "a carrier that is down is a failure worth trying again",
        down.is_retriable() && down.reason().map(Reason::code) == Some("carrier-down"),
    )?;
    Ok(())
}
// ANCHOR_END: step_alone

// ANCHOR: journey
fn the_fallback() -> Result<(), Error> {
    let recorder = RecordingReporter::default();
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
        .reporter(recorder.clone())
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
    let lines: Vec<String> = recorder.events().iter().filter_map(attempt_line).collect();
    check(
        "shipping is tried twice, as its events show",
        lines == expected,
    )?;
    Ok(())
}
// ANCHOR_END: journey

fn main() -> Result<(), Error> {
    the_ship_step_alone()?;
    the_fallback()?;
    Ok(())
}
