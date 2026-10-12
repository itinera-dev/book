//! The shop's workflow, whole: three steps, a rescue step for the charge, a retry for shipping
//! with the backup carrier on the second attempt, the audit and notify policies, and the services
//! the steps use.

use std::num::NonZeroU32;

use itinera::error::Error;
use itinera::event::{Event, EventBody, HookSource};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, Failure, InitialData, JourneyStatus, LastFailure};
use itinera::policy::{HookReporter, OnSuccess, step_policy, workflow_policy};
use itinera::report::Reporter;
use itinera::step::{Outcome, Reason, RescueOutcome, RescueStep, Step, rescue_step, step};
use itinera::workflow::{ListedService, ListedStep, Workflow, role, workflow};

// ANCHOR: services
/// Why the bank refused a charge.
enum Declined {
    ExpiredCard,
    Refused,
}

/// A payment client: it charges a card, and knows the card that replaced an expired one.
trait Payments: Send + Sync {
    fn charge(&self, card: &str, amount: i64) -> Result<String, Declined>;
    fn renewed_card(&self, card: &str) -> Option<String>;
}

/// A stock store: it reserves an order's goods, then hands them over for shipping.
trait StockStore: Send + Sync {
    fn reserve(&self, order: &str) -> bool;
    fn hand_over(&self, order: &str);
}

/// A carrier: it takes a parcel and gives back its tracking number, when it can take it.
trait Carrier: Send + Sync {
    fn send(&self, order: &str, weight: i64) -> Option<String>;
}
// ANCHOR_END: services

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

impl Payments for Bank {
    fn charge(&self, card: &str, amount: i64) -> Result<String, Declined> {
        if expiry(card).is_some_and(|year| year < self.this_year) {
            return Err(Declined::ExpiredCard);
        }
        if amount > self.limit {
            return Err(Declined::Refused);
        }
        Ok(format!("R-{card}-{amount}"))
    }

    fn renewed_card(&self, card: &str) -> Option<String> {
        let (holder, _) = card.rsplit_once('-')?;
        Some(format!("{holder}-{}", self.this_year + 3))
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

impl StockStore for Warehouse {
    fn reserve(&self, order: &str) -> bool {
        !self.out_of_stock.contains(&order)
    }

    fn hand_over(&self, _order: &str) {}
}

/// A carrier, faked: it takes parcels up to a weight.
struct Van {
    tracking_prefix: &'static str,
    max_weight: i64,
}

const MAIN_CARRIER: Van = Van {
    tracking_prefix: "T",
    max_weight: 20,
};

const BACKUP_CARRIER: Van = Van {
    tracking_prefix: "B",
    max_weight: 50,
};

impl Carrier for Van {
    fn send(&self, order: &str, weight: i64) -> Option<String> {
        if weight > self.max_weight {
            return None;
        }
        Some(format!("{}-{order}", self.tracking_prefix))
    }
}

/// Reserves the order's goods in the stock store.
#[derive(Step)]
struct Reserve {
    #[input]
    order: String,
}

#[step]
impl Reserve {
    fn run(self, #[service] stock: &dyn StockStore) -> Result<Outcome, Error> {
        if !stock.reserve(&self.order) {
            return Self::fail("out-of-stock");
        }
        Self::succeed()
    }
}

// ANCHOR: charge
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
    fn run(
        self,
        #[service] payments: &dyn Payments,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        let amount = self.price - self.discount.unwrap_or(0);
        match payments.charge(&self.card, amount) {
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
// ANCHOR_END: charge

// ANCHOR: rescue
/// The charge's rescue step: asks the customer's bank for the card that replaced an expired one.
/// A charge that failed for any other reason, or ended in an abnormal termination, which has no
/// reason, cannot be repaired this way.
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
        #[service] payments: &dyn Payments,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<RescueOutcome, Error> {
        if self.reason.as_ref().map(Reason::code) != Some("expired-card") {
            return Self::fail("nothing-to-renew");
        }
        let Some(card) = payments.renewed_card(&self.card) else {
            return Self::fail("nothing-to-renew");
        };
        contributor.contribute("card", card);
        Self::succeed()
    }
}
// ANCHOR_END: rescue

// ANCHOR: ship
/// Ships the order through the carrier it is given. A carrier that cannot take the parcel
/// might be replaced on another try.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    weight: i64,
}

#[step]
impl Ship {
    fn run(
        self,
        #[service] carrier: &dyn Carrier,
        #[service] stock: &dyn StockStore,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        let Some(tracking) = carrier.send(&self.order, self.weight) else {
            return Self::fail_retriably("parcel-refused");
        };
        stock.hand_over(&self.order);
        contributor.contribute("tracking", tracking);
        Self::succeed()
    }
}
// ANCHOR_END: ship

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
            EventBody::StepRescuing { step, rescue, .. } => {
                println!("  INFO {} rescued by {rescue}", step.step);
            }
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

// ANCHOR: workflow
/// The shop's workflow: reserve, charge and ship, auditing the charge, rescuing it when the
/// card has expired, and telling the customer how the order went.
#[workflow(name = "orders", policies = [Notify], reporters = [Log])]
#[with_step(Reserve, name = "reserve")]
#[with_step(Charge, name = "charge", rescue = RenewCard, policies = [Audit])]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders;
// ANCHOR_END: workflow

// ANCHOR: providers
#[workflow]
impl Orders {
    /// One payment client for the journey, used by the charge and its rescue step.
    #[provider(key = "payments")]
    fn bank(&self) -> Result<Box<dyn Payments>, Error> {
        Ok(Box::new(BANK))
    }

    /// One stock store for the journey, shared by reserve and ship.
    #[provider(key = "stock")]
    fn warehouse(&self) -> Result<Box<dyn StockStore>, Error> {
        Ok(Box::new(WAREHOUSE))
    }

    /// A carrier for every attempt: the main carrier first, the backup carrier after.
    #[provider(per_attempt)]
    fn carrier(&self, #[attempt] attempt: NonZeroU32) -> Result<Box<dyn Carrier>, Error> {
        if attempt.get() == 1 {
            Ok(Box::new(MAIN_CARRIER))
        } else {
            Ok(Box::new(BACKUP_CARRIER))
        }
    }
}
// ANCHOR_END: providers

#[role]
impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        println!("  to {customer}: {message}");
    }
}

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

/// A service in the listing: its key, and the provider that serves it.
fn service(listed: &ListedService) -> String {
    format!("{} from {}", listed.key, listed.provider)
}

/// The services of a listing's entry, separated by commas.
fn services(listed: &[ListedService]) -> String {
    listed.iter().map(service).collect::<Vec<_>>().join(", ")
}

/// One entry of the listing: a step's position and name, its policies, its services with the
/// provider that serves each, and its rescue step with its services.
fn entry(listed: &ListedStep) -> String {
    let mut entry = format!("{}. {}", listed.position, listed.step);
    if !listed.policies.is_empty() {
        let policies: Vec<String> = listed.policies.iter().map(ToString::to_string).collect();
        entry.push_str(&format!(", with {}", policies.join(", ")));
    }
    entry.push_str(&format!("; services: {}", services(&listed.services)));
    if let Some(rescue) = &listed.rescue {
        entry.push_str(&format!(
            "; rescued by {}, services: {}",
            rescue.step,
            services(&rescue.services)
        ));
    }
    entry
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ANCHOR: list
    for listed in Orders::descriptor().listing() {
        println!("{}", entry(&listed));
    }
    // ANCHOR_END: list
    // ANCHOR: orders
    // The order, the customer, the card, the price, the discount and the parcel's weight in kg.
    let orders = [
        ("A-1", "Ann", "ann-card-2028", 40, Some(5), 2),
        ("A-2", "Ben", "ben-card-2024", 25, None, 1),
        ("A-3", "Cat", "cat-card-2027", 30, None, 30),
        ("A-4", "Dan", "dan-card-2025", 1200, None, 3),
    ];
    // ANCHOR_END: orders
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

/// Why a journey failed: the reason a step gave, or the cause alone.
fn why(failure: &Failure) -> String {
    match failure {
        Failure::Failure(reason) | Failure::RetriesExhausted(LastFailure::Reason(reason)) => {
            format!("{}, {}", failure.cause(), reason.code())
        }
        _ => failure.cause().to_string(),
    }
}
