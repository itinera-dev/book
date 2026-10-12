//! The checkout of chapter 1 rewritten as a workflow, as far as the rewrite goes before services
//! and rescue steps: three steps, the audit and notify policies, and a reporter for the log.

use itinera::error::Error;
use itinera::event::{Event, EventBody, HookSource};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, Failure, InitialData, JourneyStatus, LastFailure};
use itinera::policy::{HookReporter, OnSuccess, step_policy, workflow_policy};
use itinera::report::Reporter;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::{role, workflow};

/// The customers' bank, faked. A card ends with the year it expires, such as `ann-card-2028`; the
/// bank refuses a card that expired before this year, and any charge above its limit.
struct Bank {
    this_year: u32,
    limit: i64,
}

const BANK: Bank = Bank {
    this_year: 2026,
    limit: 1000,
};

/// Why the bank refused a charge.
enum Declined {
    ExpiredCard,
    Refused,
}

impl Bank {
    fn charge(&self, card: &str, amount: i64) -> Result<String, Declined> {
        if expiry(card).is_some_and(|year| year < self.this_year) {
            return Err(Declined::ExpiredCard);
        }
        if amount > self.limit {
            return Err(Declined::Refused);
        }
        Ok(format!("R-{card}-{amount}"))
    }
}

/// The year a card expires, from the end of its number.
fn expiry(card: &str) -> Option<u32> {
    card.rsplit_once('-')?.1.parse().ok()
}

/// The warehouse, faked: it lists the orders it has no stock for.
struct Warehouse {
    out_of_stock: &'static [&'static str],
}

const WAREHOUSE: Warehouse = Warehouse { out_of_stock: &[] };

impl Warehouse {
    fn reserve(&self, order: &str) -> bool {
        !self.out_of_stock.contains(&order)
    }
}

/// A carrier, faked: it takes parcels up to a weight.
struct Carrier {
    tracking_prefix: &'static str,
    max_weight: i64,
}

const MAIN_CARRIER: Carrier = Carrier {
    tracking_prefix: "T",
    max_weight: 20,
};

impl Carrier {
    fn ship(&self, order: &str, weight: i64) -> Option<String> {
        if weight > self.max_weight {
            return None;
        }
        Some(format!("{}-{order}", self.tracking_prefix))
    }
}

// ANCHOR: steps
/// Reserves the order's goods in the warehouse.
#[derive(Step)]
struct Reserve {
    #[input]
    order: String,
}

#[step]
impl Reserve {
    fn run(self) -> Result<Outcome, Error> {
        if !WAREHOUSE.reserve(&self.order) {
            return Self::fail("out-of-stock");
        }
        Self::succeed()
    }
}

/// Charges the card the price, less any discount.
#[derive(Step)]
struct Charge {
    #[input]
    card: String,
    #[input]
    price: i64,
    #[input]
    discount: Option<i64>,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        let amount = self.price - self.discount.unwrap_or(0);
        match BANK.charge(&self.card, amount) {
            Ok(receipt) => {
                contributor.contribute("charged", amount);
                contributor.contribute("receipt", receipt);
                Self::succeed()
            }
            Err(Declined::ExpiredCard) => Self::fail("expired-card"),
            Err(Declined::Refused) => Self::fail("declined"),
        }
    }
}

/// Ships the order through the carrier. A carrier that cannot take the parcel might be replaced
/// on another try.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    weight: i64,
}

#[step]
impl Ship {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        match MAIN_CARRIER.ship(&self.order, self.weight) {
            Some(tracking) => {
                contributor.contribute("tracking", tracking);
                Self::succeed()
            }
            None => Self::fail_retriably("parcel-refused"),
        }
    }
}
// ANCHOR_END: steps

// ANCHOR: policies
/// Audits every successful charge, in the journey's events.
#[derive(Default)]
struct Audit;

#[step_policy(name = "audit")]
impl Audit {
    #[on_step_success]
    fn audited(
        &self,
        #[from_step] charged: i64,
        #[from_step] receipt: String,
        #[reporter] mut reporter: HookReporter<'_>,
    ) -> Result<Option<OnSuccess>, Error> {
        reporter.info(format!("charged {charged}, receipt {receipt}"))?;
        Self::carry_on()
    }
}

/// A role the shop provides: telling a customer something.
trait Messages {
    fn tell(&self, customer: &str, message: &str);
}

/// Tells the customer how the order went.
#[derive(Default)]
struct Notify;

#[workflow_policy(name = "notify")]
impl Notify {
    #[on_workflow_success]
    fn shipped(
        &self,
        #[from_workflow] customer: String,
        #[from_workflow] order: String,
        #[from_workflow] tracking: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(
            &customer,
            &format!("order {order} is on its way: {tracking}"),
        );
        Ok(())
    }

    #[on_workflow_failure]
    fn failed(
        &self,
        #[from_workflow] customer: String,
        #[from_workflow] order: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, &format!("order {order} could not be completed"));
        Ok(())
    }
}
// ANCHOR_END: policies

// ANCHOR: reporter
/// The shop's log: every failure, every decision and what the hooks report, for every journey, in
/// one format.
#[derive(Default)]
struct Log;

impl Reporter for Log {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        match &event.body {
            EventBody::StepFailed { step, reason, .. } => {
                let code = reason.code();
                println!("  WARN {}, attempt {}: {code}", step.step, step.attempt);
            }
            EventBody::StepRetrying { step, .. } => println!("  INFO {} tried again", step.step),
            EventBody::StepGivenUp { step, cause, .. } => {
                println!("  WARN {} given up: {cause}", step.step);
            }
            EventBody::JourneyInfo {
                hook: HookSource::Step { policy, .. },
                message,
                ..
            } => println!("  INFO {policy}: {message}"),
            _ => {}
        }
        Ok(())
    }
}
// ANCHOR_END: reporter

// ANCHOR: workflow
/// The shop's workflow: reserve, charge and ship, auditing the charge and telling the customer.
#[workflow(name = "orders", policies = [Notify], reporters = [Log])]
#[with_step(Reserve, name = "reserve")]
#[with_step(Charge, name = "charge", policies = [Audit])]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders;

#[role]
impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        println!("  to {customer}: {message}");
    }
}
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    card: String,
    price: i64,
    discount: Option<i64>,
    weight: i64,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The order, the customer, the card, the price, the discount and the parcel's weight in kg.
    let orders = [
        ("A-1", "Ann", "ann-card-2028", 40, Some(5), 2),
        ("A-2", "Ben", "ben-card-2024", 25, None, 1),
        ("A-3", "Cat", "cat-card-2027", 30, None, 30),
        ("A-4", "Dan", "dan-card-2025", 1200, None, 3),
    ];
    let mut executor = LocalExecutor::new();
    for (order, customer, card, price, discount, weight) in orders {
        println!("{order}:");
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                customer: customer.to_owned(),
                card: card.to_owned(),
                price,
                discount,
                weight,
            })
            .create()?;
        let result = executor.run(instance)?;
        match &result.status {
            JourneyStatus::Failed { failure, .. } => println!("failed: {}", why(failure)),
            status => println!("{}", status.kind()),
        }
    }
    Ok(())
}
// ANCHOR_END: run

/// Why a journey failed: the reason a step gave, or the cause alone.
fn why(failure: &Failure) -> String {
    match failure {
        Failure::Failure(reason) | Failure::RetriesExhausted(LastFailure::Reason(reason)) => {
            format!("{}, {}", failure.cause(), reason.code())
        }
        _ => failure.cause().to_string(),
    }
}
