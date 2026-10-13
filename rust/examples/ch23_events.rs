//! Seeing what happened: a reporter logs every event of every journey, and a reporter that fails
//! aborts the journey it reports.

use itinera::error::Error;
use itinera::event::{Event, EventBody, Source};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, DataBag, InitialData};
use itinera::report::{GlobalMetadata, Reporter};
use itinera::step::{Outcome, Step, StepReporter, step};
use itinera::value::AnyValue;
use itinera::workflow::workflow;

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

// ANCHOR: ship
/// Ships an order through the carrier, says so in an event of its own, and labels the rest of
/// the journey's log with the tracking number.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
}

#[step]
impl Ship {
    fn run(
        self,
        #[contributor] mut contributor: Contributor<'_>,
        #[reporter] mut reporter: StepReporter<'_>,
    ) -> Result<Outcome, Error> {
        let tracking = format!("T-{}", self.order);
        reporter.info("handed to the carrier")?;
        reporter.set_global_metadata("tracking", tracking.clone())?;
        contributor.contribute("tracking", tracking);
        Self::succeed()
    }
}
// ANCHOR_END: ship

// ANCHOR: log
/// Writes every event of a journey to the shop's log, one line each, labelled with the customer
/// and, once the parcel has shipped, its tracking number.
#[derive(Default)]
struct Log {
    metadata: GlobalMetadata,
}

impl Reporter for Log {
    fn set_global_metadata(&mut self, key: &str, value: &AnyValue) -> Result<(), Error> {
        self.metadata.insert(key, value.clone());
        Ok(())
    }

    fn report(&mut self, event: &Event) -> Result<(), Error> {
        let line = format!(
            "{} {} #{} {}",
            event.journey_id,
            labels(&self.metadata),
            event.sequence,
            event.kind()
        );
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
        EventBody::JourneyStarted { initial_keys, .. } => Some(initial_keys.join(", ")),
        EventBody::AttemptStarted { step, .. } | EventBody::StepSucceeded { step, .. } => {
            Some(format!("{} (attempt {})", step.step, step.attempt))
        }
        EventBody::StepInfo { step, message, .. } => Some(format!("{}: {message}", step.step)),
        EventBody::ContributionCommitted { key, .. } => Some(key.clone()),
        EventBody::GlobalMetadataSet {
            key,
            source: Source::Step(step),
            ..
        } => Some(format!("{key}, set by {}", step.step)),
        EventBody::JourneyAborted { abort, .. } => match abort.error() {
            Some(error) => Some(format!("{}: {error}", abort.reason())),
            None => Some(abort.reason().to_string()),
        },
        _ => None,
    }
}

/// The labels of a line: the customer, then the tracking number once the parcel has shipped.
fn labels(metadata: &GlobalMetadata) -> String {
    let customer = text(metadata, "customer").unwrap_or("-");
    match text(metadata, "tracking") {
        Some(tracking) => format!("{customer} {tracking}"),
        None => customer.to_owned(),
    }
}

/// The text of a global metadata entry, if the journey has one.
fn text<'a>(metadata: &'a GlobalMetadata, key: &str) -> Option<&'a str> {
    metadata
        .get(key)
        .and_then(AnyValue::downcast_ref::<String>)
        .map(String::as_str)
}
// ANCHOR_END: log

// ANCHOR: workflow
/// The shop's workflow: reserve, charge, ship, with every journey logged.
#[workflow(name = "orders", reporters = [Log])]
#[with_step(Reserve, name = "reserve")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: workflow

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
}

// ANCHOR: archive
/// Keeps the events of one journey for the accountants, in an archive with room for only a few.
struct Archive {
    room: usize,
    kept: Vec<String>,
}

impl Reporter for Archive {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        if self.kept.len() == self.room {
            return Err(Error::msg("the archive is full"));
        }
        self.kept.push(event.kind().to_owned());
        Ok(())
    }
}
// ANCHOR_END: archive

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut executor = LocalExecutor::new();

    let logged = Orders::builder()
        .initial_data(NewOrder {
            order: "A-1".to_owned(),
            price: 40,
        })
        .reporters_global_metadata("customer", "Ann".to_owned())
        .create()?;
    let result = executor.run(logged)?;
    println!("A-1 {}", result.status.kind());

    let archived = Orders::builder()
        .initial_data(NewOrder {
            order: "A-2".to_owned(),
            price: 25,
        })
        .reporters_global_metadata("customer", "Ben".to_owned())
        .reporter(Archive {
            room: 4,
            kept: Vec::new(),
        })
        .create()?;
    let result = executor.run(archived)?;
    println!("A-2 {}", result.status.kind());
    Ok(())
}
// ANCHOR_END: run
