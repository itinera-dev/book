//! The checkout of chapter 1, rewritten as a workflow to the end: the steps reach the bank, the
//! warehouse and the carriers through services, the carrier's provider gives the second attempt
//! of shipping the backup carrier, and the charge's rescue step renews an expired card.

use std::num::NonZeroU32;

use itinera::error::Error;
use itinera::event::{Event, EventBody};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, Failure, InitialData, JourneyStatus, LastFailure};
use itinera::policy::{OnSuccess, step_policy, workflow_policy};
use itinera::report::Reporter;
use itinera::step::{Outcome, Reason, Step, step};
use itinera::workflow::{role, workflow};

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
    fn send(&self, order: &str) -> Option<String>;
}
// ANCHOR_END: services

/// The customers' bank, faked: it knows which cards have expired and which it refuses.
struct Bank {
    expired: &'static [&'static str],
    refused: &'static [&'static str],
}

const BANK: Bank = Bank {
    expired: &["bea-card", "dan-card"],
    refused: &["dan-card-renewed"],
};

impl Payments for Bank {
    fn charge(&self, card: &str, amount: i64) -> Result<String, Declined> {
        if self.expired.contains(&card) {
            return Err(Declined::ExpiredCard);
        }
        if self.refused.contains(&card) {
            return Err(Declined::Refused);
        }
        Ok(format!("R-{card}-{amount}"))
    }

    fn renewed_card(&self, card: &str) -> Option<String> {
        Some(format!("{card}-renewed"))
    }
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

/// A carrier, faked: it cannot take the parcels of the orders it lists.
struct Van {
    name: &'static str,
    cannot_take: &'static [&'static str],
}

const MAIN_CARRIER: Van = Van {
    name: "RAPID",
    cannot_take: &["A-3"],
};

const BACKUP_CARRIER: Van = Van {
    name: "STEADY",
    cannot_take: &[],
};

impl Carrier for Van {
    fn send(&self, order: &str) -> Option<String> {
        if self.cannot_take.contains(&order) {
            return None;
        }
        Some(format!("{}-{order}", self.name))
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
/// Asks the customer's bank for the card that replaced an expired one. A charge that failed for
/// any other reason, or ended without one, cannot be repaired this way.
#[derive(Step)]
struct RenewCard {
    #[input]
    card: String,
    #[reason]
    reason: Option<Reason>,
}

#[step]
impl RenewCard {
    fn run(
        self,
        #[service] payments: &dyn Payments,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        if self.reason.as_ref().map(Reason::code) != Some("expired-card") {
            return Self::fail("nothing-to-renew");
        }
        match payments.renewed_card(&self.card) {
            Some(card) => {
                contributor.contribute("card", card);
                Self::succeed()
            }
            None => Self::fail("no-renewed-card"),
        }
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
}

#[step]
impl Ship {
    fn run(
        self,
        #[service] carrier: &dyn Carrier,
        #[service] stock: &dyn StockStore,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        let Some(tracking) = carrier.send(&self.order) else {
            return Self::fail_retriably("carrier-unavailable");
        };
        stock.hand_over(&self.order);
        contributor.contribute("tracking", tracking);
        Self::succeed()
    }
}
// ANCHOR_END: ship

/// Audits every successful charge.
#[derive(Default)]
struct Audit;

#[step_policy(name = "audit")]
impl Audit {
    #[on_step_success]
    fn audited(
        &self,
        #[from_step] charged: i64,
        #[from_step] receipt: String,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<OnSuccess>, Error> {
        contributor.contribute("audit", format!("charged {charged}, receipt {receipt}"));
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

/// The shop's log: every failure and every decision of every journey, in one format.
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
    #[provider]
    fn payments(&self) -> Result<Box<dyn Payments>, Error> {
        Ok(Box::new(BANK))
    }

    /// One stock store for the journey, shared by reserve and ship.
    #[provider]
    fn stock(&self) -> Result<Box<dyn StockStore>, Error> {
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        ("A-1", "Ann", "ann-card", 40, Some(5)),
        ("A-2", "Bea", "bea-card", 25, None),
        ("A-3", "Cal", "cal-card", 30, None),
        ("A-4", "Dan", "dan-card", 15, None),
    ];
    let mut executor = LocalExecutor::new();
    for (order, customer, card, price, discount) in orders {
        println!("{order}:");
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                customer: customer.to_owned(),
                card: card.to_owned(),
                price,
                discount,
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
