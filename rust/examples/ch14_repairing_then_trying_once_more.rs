//! Repairing, then trying once more: when the charge fails, a rescue step asks the bank for the
//! card that replaced the order's, and the charge runs once more with it.

use std::num::NonZeroU32;

use itinera::error::Error;
use itinera::event::{Event, EventBody};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, DataBag, InitialData, Output};
use itinera::report::Reporter;
use itinera::service::{Provided, provide};
use itinera::step::{
    Outcome, Reason, RescueOutcome, RescueStep, Step, StepReporter, rescue_step, step,
};
use itinera::workflow::workflow;

// ANCHOR: payments
/// A payment client: it charges an order's amount to a card, and gives back a receipt, or nothing
/// when the card has expired; and it says which card replaced one, if any did.
trait Payments: Send + Sync {
    fn charge(&self, order: &str, card: &str, amount: i64) -> Option<String>;
    fn replacement(&self, card: &str) -> Option<String>;
}
// ANCHOR_END: payments

/// A carrier: it books a parcel and gives back its tracking number, then says whether it
/// collects a parcel of this weight in kg.
trait Carrier: Send + Sync {
    fn book(&self, order: &str) -> String;
    fn collects(&self, weight: i64) -> bool;
}

/// The shop's bank, faked: a card is its holder's name and the year it expires, and the bank
/// declines a card that expired before this year. It prints what it does.
struct Bank {
    this_year: u32,
}

impl Payments for Bank {
    fn charge(&self, order: &str, card: &str, amount: i64) -> Option<String> {
        if expiry(card).is_some_and(|year| year < self.this_year) {
            println!("the bank declines card {card}: it has expired");
            return None;
        }
        println!("the bank charges {amount} for order {order}");
        Some(format!("R-{order}"))
    }

    /// A card that has expired was replaced by its holder's card that expires in three years.
    fn replacement(&self, card: &str) -> Option<String> {
        let (holder, _) = card.rsplit_once('-')?;
        let replacement = format!("{holder}-{}", self.this_year + 3);
        println!("the bank says card {card} was replaced by {replacement}");
        Some(replacement)
    }
}

/// The year a card expires: the part of it after its last `-`.
fn expiry(card: &str) -> Option<u32> {
    card.rsplit_once('-')?.1.parse().ok()
}

/// The shop's main carrier, faked: it books any parcel, and collects one only up to 20 kg.
struct MainCarrier;

impl Carrier for MainCarrier {
    fn book(&self, order: &str) -> String {
        println!("the main carrier books order {order}");
        format!("T-{order}")
    }

    fn collects(&self, weight: i64) -> bool {
        weight <= 20
    }
}

/// The shop's backup carrier, faked: it books any parcel, and collects one only up to 80 kg.
struct BackupCarrier;

impl Carrier for BackupCarrier {
    fn book(&self, order: &str) -> String {
        println!("the backup carrier books order {order}");
        format!("B-{order}")
    }

    fn collects(&self, weight: i64) -> bool {
        weight <= 80
    }
}

/// Charges an order's price to its card through the payment client, and gives back its receipt.
#[derive(Step)]
struct Charge {
    #[input]
    order: String,
    #[input]
    price: i64,
    #[input]
    card: String,
}

#[step]
impl Charge {
    fn run(
        self,
        #[service] payments: &dyn Payments,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        let Some(receipt) = payments.charge(&self.order, &self.card, self.price) else {
            return Self::fail("card-expired");
        };
        contributor.contribute("receipt", receipt);
        Self::succeed()
    }
}

// ANCHOR: rescue
/// Repairs a charge that failed because its card had expired: it asks the bank for the card that
/// replaced it, and gives it back as the order's card.
#[derive(RescueStep)]
struct RenewCard {
    #[input]
    card: String,
    #[reason]
    reason: Reason,
}

#[rescue_step]
impl RenewCard {
    fn run(
        self,
        #[service] payments: &dyn Payments,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<RescueOutcome, Error> {
        if self.reason.code() != "card-expired" {
            return Self::fail("not-repairable");
        }
        let Some(card) = payments.replacement(&self.card) else {
            return Self::fail("not-replaced");
        };
        contributor.contribute("card", card);
        Self::succeed()
    }
}
// ANCHOR_END: rescue

/// Ships an order through the carrier, with its receipt in the parcel, gives back its tracking
/// number, and says when the parcel has left. A parcel the carrier won't collect may be taken on
/// another attempt.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    weight: i64,
    #[input]
    receipt: String,
}

#[step]
impl Ship {
    fn run(
        self,
        #[service] carrier: &dyn Carrier,
        #[contributor] mut contributor: Contributor<'_>,
        #[reporter] mut reporter: StepReporter<'_>,
    ) -> Result<Outcome, Error> {
        let tracking = carrier.book(&self.order);
        contributor.contribute("tracking", tracking);
        if !carrier.collects(self.weight) {
            return Self::fail_retriably("too-heavy");
        }
        reporter.info(format!("the parcel leaves with receipt {}", self.receipt))?;
        Self::succeed()
    }
}

/// The shop's log: it writes a line for every event of a journey, starting with the workflow's
/// name and the journey's ID.
#[derive(Default)]
struct Log;

impl Reporter for Log {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        match detail(&event.body) {
            Some(detail) => println!(
                "log {} {} #{} {} {detail}",
                event.workflow,
                event.journey_id,
                event.sequence,
                event.kind()
            ),
            None => println!(
                "log {} {} #{} {}",
                event.workflow,
                event.journey_id,
                event.sequence,
                event.kind()
            ),
        }
        Ok(())
    }
}

/// What the log says after an event's kind, for the kinds it says more about: the step the event
/// is about, and what it names.
fn detail(body: &EventBody) -> Option<String> {
    match body {
        EventBody::JourneyStarted { initial_keys, .. } => Some(initial_keys.join(", ")),
        EventBody::AttemptStarted { step, .. } => {
            Some(format!("{}, attempt {}", step.step, step.attempt))
        }
        EventBody::StepSucceeded { step, .. } | EventBody::StepRetrying { step, .. } => {
            Some(step.step.to_string())
        }
        EventBody::ServiceProvided {
            step,
            key,
            provider,
            implementation,
            ..
        } => Some(format!(
            "{} gets {key} from {provider}, a {implementation}",
            step.step
        )),
        EventBody::StepFailed {
            step,
            retriable,
            reason,
            ..
        } => Some(format!(
            "{}: {}, {}",
            step.step,
            reason.code(),
            if *retriable {
                "retriable"
            } else {
                "not retriable"
            }
        )),
        EventBody::StepGivenUp { step, cause, .. } => Some(format!("{}: {cause}", step.step)),
        // ANCHOR: rescue_events
        EventBody::StepRescuing {
            step,
            cause,
            rescue,
            ..
        } => Some(format!("{}: {cause}, rescued by {rescue}", step.step)),
        EventBody::StepRescued { step, rescue, .. } => Some(format!("{} by {rescue}", step.step)),
        EventBody::ContributionCommitted { key, .. } | EventBody::DataOverwritten { key, .. } => {
            Some(key.clone())
        }
        // ANCHOR_END: rescue_events
        EventBody::StepInfo { step, message, .. } => Some(format!("{}: {message}", step.step)),
        EventBody::JourneyFailed { step, .. } => Some(step.to_string()),
        _ => None,
    }
}

/// The customer service desk: of all the events of a journey, it keeps only a failed journey's.
#[derive(Default)]
struct Desk;

impl Reporter for Desk {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        if let EventBody::JourneyFailed { step, .. } = &event.body {
            println!("desk: order {} failed at {step}", event.journey_id);
        }
        Ok(())
    }
}

// ANCHOR: workflow
/// The shop's workflow: it charges, repairing a charge that fails, then ships, and shipping may be
/// tried once more.
#[workflow(name = "orders", reporters = [Log, Desk])]
#[with_step(Charge, name = "charge", rescue = RenewCard)]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders;
// ANCHOR_END: workflow

#[workflow]
impl Orders {
    /// Names each journey after its order.
    #[id_generator]
    fn order_number(&self, data: &DataBag) -> Result<String, Error> {
        Ok(data.required("order")?)
    }

    /// The bank, as the payment client of the journey.
    #[provider(key = "payments")]
    fn bank(&self) -> Result<Provided<dyn Payments>, Error> {
        Ok(provide!(Bank { this_year: 2026 }))
    }

    /// A carrier for every attempt of a step: the main carrier first, the backup carrier after.
    #[provider(key = "carrier", per_attempt)]
    fn carriers(&self, #[attempt] attempt: NonZeroU32) -> Result<Provided<dyn Carrier>, Error> {
        if attempt == NonZeroU32::MIN {
            return Ok(provide!(MainCarrier));
        }
        Ok(provide!(BackupCarrier))
    }
}

/// What the shop knows about an order when its journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    weight: i64,
    card: String,
}

/// What the shop reads back once the journey has ended.
#[derive(Output)]
struct Order {
    receipt: Option<String>,
    tracking: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The order, the customer, the price, the parcel's weight in kg and the card.
    let orders = [
        ("A-1", "Ann", 40, 2, "ann-2028"),
        ("A-7", "Gil", 45, 60, "gil-2027"),
        ("A-8", "Jo", 30, 1, "jo-2024"),
    ];
    for (order, customer, price, weight, card) in orders {
        println!("{customer}'s order:");
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price,
                weight,
                card: card.to_owned(),
            })
            .create()?;
        let result = LocalExecutor::new().run(instance)?;
        let data: Order = result.output()?;
        let receipt = data.receipt.as_deref().unwrap_or("none");
        let tracking = data.tracking.as_deref().unwrap_or("none");
        println!(
            "the journey {}, with receipt {receipt} and tracking {tracking}",
            result.status.kind(),
        );
    }
    Ok(())
}
