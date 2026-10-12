//! Working with agents: the shop's paths, the outcomes and decisions a journey goes through, are
//! approved by a person and checked by tests, so a flow change that alters them cannot pass
//! unnoticed.

use std::num::NonZeroU32;

use itinera::error::Error;
use itinera::event::{Event, EventBody};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData};
use itinera::report::RecordingReporter;
use itinera::step::{Outcome, Reason, RescueOutcome, RescueStep, Step, rescue_step, step};
use itinera::workflow::workflow;

/// A payment client: it charges a card, or gives back nothing when the card has expired.
trait Payments: Send + Sync {
    fn charge(&self, card: &str, amount: i64) -> Option<String>;
}

/// The bank that issued a card: it knows the card that replaced an expired one.
trait Issuer: Send + Sync {
    fn renewed_card(&self, card: &str) -> Option<String>;
}

/// A carrier: it takes a parcel and gives back its tracking number, when it can take it.
trait Carrier: Send + Sync {
    fn send(&self, order: &str) -> Option<String>;
}

/// The bank, faked: Ben's card has expired, and has been renewed.
struct Bank;

impl Payments for Bank {
    fn charge(&self, card: &str, amount: i64) -> Option<String> {
        (card != "ben-card").then(|| format!("R-{card}-{amount}"))
    }
}

impl Issuer for Bank {
    fn renewed_card(&self, card: &str) -> Option<String> {
        Some(format!("{card}-2"))
    }
}

/// A carrier, faked: up or down, as the test says.
struct Van {
    up: bool,
}

impl Carrier for Van {
    fn send(&self, order: &str) -> Option<String> {
        self.up.then(|| format!("T-{order}"))
    }
}

/// Charges the card the price. An agent may rewrite this business rule, as long as the paths stay
/// what was approved.
#[derive(Step)]
struct Charge {
    #[input]
    card: String,
    #[input]
    price: i64,
}

#[step]
impl Charge {
    fn run(
        self,
        #[service] payments: &dyn Payments,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        let Some(receipt) = payments.charge(&self.card, self.price) else {
            return Self::fail("expired-card");
        };
        contributor.contribute("receipt", receipt);
        Self::succeed()
    }
}

/// The charge's rescue step: asks the card's bank for the card that replaced an expired one.
#[derive(RescueStep)]
struct RenewCard {
    #[input]
    card: String,
    #[reason]
    reason: Option<Reason>,
}

#[rescue_step]
impl RenewCard {
    fn run(
        self,
        #[service] issuer: &dyn Issuer,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<RescueOutcome, Error> {
        if self.reason.as_ref().map(Reason::code) != Some("expired-card") {
            return Self::fail("nothing-to-renew");
        }
        let Some(card) = issuer.renewed_card(&self.card) else {
            return Self::fail("nothing-to-renew");
        };
        contributor.contribute("card", card);
        Self::succeed()
    }
}

/// Ships the order through the carrier it is lent.
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
        let Some(tracking) = carrier.send(&self.order) else {
            return Self::fail_retriably("carrier-down");
        };
        contributor.contribute("tracking", tracking);
        Self::succeed()
    }
}

/// The shop's workflow. Changing it changes the flow, which a person approves.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge", rescue = RenewCard)]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders {
    main_carrier_up: bool,
}

#[workflow]
impl Orders {
    #[provider(key = "payments")]
    fn bank(&self) -> Result<Box<dyn Payments>, Error> {
        Ok(Box::new(Bank))
    }

    #[provider(key = "issuer", steps = ["charge_rescue"])]
    fn issuer(&self) -> Result<Box<dyn Issuer>, Error> {
        Ok(Box::new(Bank))
    }

    /// The main carrier on the first attempt, the backup carrier, which is always up, after.
    #[provider(per_attempt)]
    fn carrier(&self, #[attempt] attempt: NonZeroU32) -> Result<Box<dyn Carrier>, Error> {
        let up = attempt.get() > 1 || self.main_carrier_up;
        Ok(Box::new(Van { up }))
    }
}

/// What the shop knows about an order when its journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    card: String,
    price: i64,
}

// ANCHOR: paths
/// The path of an order paid with an expired card: the charge is rescued, then the order ships.
const EXPIRED_CARD: &[&str] = &[
    "step_failed charge",
    "step_rescuing charge",
    "step_succeeded charge_rescue",
    "step_rescued charge",
    "step_succeeded charge",
    "step_succeeded ship",
    "journey_succeeded",
];

/// The path of an order whose main carrier is down: shipping is tried once more.
const CARRIER_DOWN: &[&str] = &[
    "step_succeeded charge",
    "step_failed ship",
    "step_retrying ship",
    "step_succeeded ship",
    "journey_succeeded",
];
// ANCHOR_END: paths

// ANCHOR: recorder
/// The line an event adds to the path, with the step it is about, or nothing when it is not on
/// the path.
fn on_the_path(event: &Event) -> Option<String> {
    let kind = event.kind();
    match &event.body {
        EventBody::StepSucceeded { step, .. }
        | EventBody::StepFailed { step, .. }
        | EventBody::StepSkipped { step, .. }
        | EventBody::StepAbnormalTermination { step, .. }
        | EventBody::StepRetrying { step, .. }
        | EventBody::StepRescuing { step, .. }
        | EventBody::StepRescued { step, .. }
        | EventBody::StepGivenUp { step, .. } => Some(format!("{kind} {}", step.step)),
        EventBody::JourneyFailed { step, .. } => Some(format!("{kind} {step}")),
        EventBody::JourneySucceeded { .. } | EventBody::JourneyAborted { .. } => {
            Some(kind.to_owned())
        }
        _ => None,
    }
}
// ANCHOR_END: recorder

// ANCHOR: checks
/// Runs one order and gives back the path its journey went through.
fn path_of(
    order: NewOrder,
    main_carrier_up: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let recorder = RecordingReporter::default();
    let instance = Orders::builder()
        .main_carrier_up(main_carrier_up)
        .reporter(recorder.clone())
        .initial_data(order)
        .create()?;
    LocalExecutor::new().run(instance)?;
    Ok(recorder.events().iter().filter_map(on_the_path).collect())
}

/// Prints the path that was checked, or fails the program when the journey went another way.
fn approved(what: &str, path: &[String], expected: &[&str]) -> Result<(), String> {
    if path != expected {
        return Err(format!("not approved: {what}: {path:?}"));
    }
    println!("ok: {what}");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ben = NewOrder {
        order: "A-2".to_owned(),
        card: "ben-card".to_owned(),
        price: 25,
    };
    approved(
        "an expired card is renewed, then charged",
        &path_of(ben, true)?,
        EXPIRED_CARD,
    )?;
    let cat = NewOrder {
        order: "A-3".to_owned(),
        card: "cat-card".to_owned(),
        price: 30,
    };
    approved(
        "a carrier that is down is tried once more",
        &path_of(cat, false)?,
        CARRIER_DOWN,
    )?;
    Ok(())
}
// ANCHOR_END: checks
