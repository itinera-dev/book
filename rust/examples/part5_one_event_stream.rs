//! With Itinera: the same checkout as steps, and one reporter that logs every journey the same
//! way.

use itinera::error::Error;
use itinera::event::{Event, EventBody};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, DataBag, InitialData};
use itinera::report::Reporter;
use itinera::step::{Outcome, Reason, Step, step};
use itinera::workflow::workflow;

// ANCHOR: steps
/// Reserves the order's stock in the warehouse.
#[derive(Step)]
struct Reserve;

#[step]
impl Reserve {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        contributor.contribute("reserved", true);
        Self::succeed()
    }
}

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
    #[input]
    customer: String,
}

#[step]
impl Ship {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        match carrier(&self.order, &self.customer) {
            Ok(tracking) => {
                contributor.contribute("tracking", tracking);
                Self::succeed()
            }
            Err(refusal) => Self::fail(Reason::new("carrier-down").with_message(refusal)),
        }
    }
}
// ANCHOR_END: steps

/// The carrier, faked: it is down when Ben's parcel comes in.
fn carrier(order: &str, customer: &str) -> Result<String, String> {
    if customer == "Ben" {
        return Err("the carrier is down".to_owned());
    }
    Ok(format!("T-{order}"))
}

// ANCHOR: log
/// Writes every event of every journey to the shop's log, one line each.
#[derive(Default)]
struct Log;

impl Reporter for Log {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        let line = format!("{} {}", event.journey_id, event.kind());
        match detail(&event.body) {
            Some(detail) => println!("{line} {detail}"),
            None => println!("{line}"),
        }
        Ok(())
    }
}

/// The part of an event the log shows after its kind, for the kinds it says more about.
fn detail(body: &EventBody) -> Option<String> {
    match body {
        EventBody::AttemptStarted { step, .. } | EventBody::StepSucceeded { step, .. } => {
            Some(step.step.to_string())
        }
        EventBody::StepFailed { step, reason, .. } => match reason.message() {
            Some(message) => Some(format!("{}: {} ({message})", step.step, reason.code())),
            None => Some(format!("{}: {}", step.step, reason.code())),
        },
        EventBody::StepGivenUp { step, cause, .. } => Some(format!("{}: {cause}", step.step)),
        EventBody::ContributionCommitted { key, .. } => Some(key.clone()),
        EventBody::JourneyFailed { step, .. } => Some(step.to_string()),
        _ => None,
    }
}

/// The shop's workflow: reserve, charge, ship, with every journey logged.
#[workflow(name = "orders", reporters = [Log])]
#[with_step(Reserve, name = "reserve")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: log

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
    price: i64,
    customer: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        NewOrder {
            order: "A-1".to_owned(),
            price: 40,
            customer: "Ann".to_owned(),
        },
        NewOrder {
            order: "A-2".to_owned(),
            price: 25,
            customer: "Ben".to_owned(),
        },
        NewOrder {
            order: "A-3".to_owned(),
            price: 0,
            customer: "Cat".to_owned(),
        },
    ];
    let mut executor = LocalExecutor::new();
    for order in orders {
        executor.run(Orders::builder().initial_data(order).create()?)?;
    }
    Ok(())
}
