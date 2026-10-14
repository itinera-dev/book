//! One step, two workflows: the shop delivers orders and lets customers collect them at the
//! counter. Both workflows use the same reserve and charge steps and the same audit policy, each
//! with its own flow and its own services.

use itinera::error::Error;
use itinera::event::{Event, EventBody};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, Failure, InitialData, JourneyStatus};
use itinera::policy::{OnSuccess, step_policy, workflow_policy};
use itinera::report::Reporter;
use itinera::service::{Provided, provide};
use itinera::step::{Outcome, Reason, RescueOutcome, RescueStep, Step, rescue_step, step};
use itinera::workflow::{ListedService, ListedStep, Workflow, role, workflow};

/// A payment client: it charges a card and gives back a receipt, or nothing when the card has
/// expired.
trait Payments: Send + Sync {
    fn charge(&self, card: &str, amount: i64) -> Option<String>;
}

/// The bank that issued a card: it knows the card that replaced an expired one.
trait Issuer: Send + Sync {
    fn renewed_card(&self, card: &str) -> Option<String>;
}

/// A stock store: it reserves an order's goods, then hands them over.
trait StockStore: Send + Sync {
    fn reserve(&self, order: &str) -> bool;
    fn hand_over(&self, order: &str);
}

/// A carrier: it takes a parcel and gives back its tracking number.
trait Carrier: Send + Sync {
    fn send(&self, order: &str) -> Option<String>;
}

/// A way of charging cards, faked: online through the card network, or at the shop's counter.
struct Terminal {
    receipt_prefix: &'static str,
}

const CARD_NETWORK: Terminal = Terminal {
    receipt_prefix: "R",
};

const COUNTER: Terminal = Terminal {
    receipt_prefix: "C",
};

impl Payments for Terminal {
    fn charge(&self, card: &str, amount: i64) -> Option<String> {
        (card != "ben-card").then(|| format!("{}-{card}-{amount}", self.receipt_prefix))
    }
}

/// The customers' bank, faked: every expired card has been renewed.
struct Bank;

impl Issuer for Bank {
    fn renewed_card(&self, card: &str) -> Option<String> {
        Some(format!("{card}-2"))
    }
}

/// Where the goods are kept, faked: the warehouse for deliveries, the back room for collections.
struct Store {
    place: &'static str,
}

impl StockStore for Store {
    fn reserve(&self, order: &str) -> bool {
        println!("  the {} reserves order {order}", self.place);
        true
    }

    fn hand_over(&self, order: &str) {
        println!("  the {} hands over order {order}", self.place);
    }
}

/// The carrier, faked: it takes every parcel.
struct Van;

impl Carrier for Van {
    fn send(&self, order: &str) -> Option<String> {
        Some(format!("T-{order}"))
    }
}

// ANCHOR: shared
/// Reserves the order's goods in whichever stock store the workflow provides.
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

/// Charges the card the price, through whichever payment client the workflow provides.
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

/// Audits every successful charge.
#[derive(Default)]
struct Audit;

#[step_policy(name = "audit")]
impl Audit {
    #[on_step_success]
    fn audited(
        &self,
        #[from_step] receipt: String,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<OnSuccess>, Error> {
        contributor.contribute("audit", format!("receipt {receipt}"));
        Self::carry_on()
    }
}
// ANCHOR_END: shared

/// The charge's rescue step, in deliveries: asks the card's bank for the card that replaced an
/// expired one.
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

/// Ships the order, in deliveries.
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
            return Self::fail("carrier-down");
        };
        stock.hand_over(&self.order);
        contributor.contribute("tracking", tracking);
        Self::succeed()
    }
}

/// Puts the order aside at the counter, in collections.
#[derive(Step)]
struct PutAside {
    #[input]
    order: String,
}

#[step]
impl PutAside {
    fn run(self, #[service] stock: &dyn StockStore) -> Result<Outcome, Error> {
        stock.hand_over(&self.order);
        Self::succeed()
    }
}

/// A role both workflows provide: telling a customer something.
trait Messages {
    fn tell(&self, customer: &str, message: &str);
}

/// Tells the customer their parcel is on its way, or that the order could not be completed.
#[derive(Default)]
struct Delivered;

#[workflow_policy(name = "notify")]
impl Delivered {
    #[on_workflow_success]
    fn shipped(
        &self,
        #[from_workflow] customer: String,
        #[from_workflow] tracking: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, &format!("your parcel is on its way: {tracking}"));
        Ok(())
    }

    #[on_workflow_failure]
    fn failed(
        &self,
        #[from_workflow] customer: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, "your order could not be completed");
        Ok(())
    }
}

/// Tells the customer their order is waiting at the counter, or to bring another card.
#[derive(Default)]
struct ReadyToCollect;

#[workflow_policy(name = "notify")]
impl ReadyToCollect {
    #[on_workflow_success]
    fn ready(
        &self,
        #[from_workflow] customer: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, "your order is waiting at the counter");
        Ok(())
    }

    #[on_workflow_failure]
    fn failed(
        &self,
        #[from_workflow] customer: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, "please pay with another card at the counter");
        Ok(())
    }
}

/// The shop's log: every failure and every decision about one, for either workflow.
#[derive(Default)]
struct Log;

impl Reporter for Log {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        match &event.body {
            EventBody::StepFailed { step, reason, .. } => {
                let code = reason.code();
                println!("  WARN {}, attempt {}: {code}", step.step, step.attempt);
            }
            EventBody::StepRescuing { step, rescue, .. } => {
                println!("  INFO {} to be rescued by {rescue}", step.step);
            }
            EventBody::StepGivenUp { step, cause, .. } => {
                println!("  WARN {} given up: {cause}", step.step);
            }
            _ => {}
        }
        Ok(())
    }
}

// ANCHOR: deliveries
/// Orders delivered to the customer: the warehouse's stock, the card network, a rescue step
/// that renews an expired card, and shipping.
#[workflow(name = "deliveries", policies = [Delivered], reporters = [Log])]
#[with_step(Reserve, name = "reserve")]
#[with_step(Charge, name = "charge", rescue = RenewCard, policies = [Audit])]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Deliveries;

#[workflow]
impl Deliveries {
    #[provider(key = "stock")]
    fn warehouse(&self) -> Result<Provided<dyn StockStore>, Error> {
        Ok(provide!(Store { place: "warehouse" }))
    }

    #[provider(key = "payments")]
    fn card_network(&self) -> Result<Provided<dyn Payments>, Error> {
        Ok(provide!(CARD_NETWORK))
    }

    #[provider(key = "issuer", steps = ["charge_rescue"])]
    fn bank(&self) -> Result<Provided<dyn Issuer>, Error> {
        Ok(provide!(Bank))
    }

    #[provider(per_attempt)]
    fn carrier(&self) -> Result<Provided<dyn Carrier>, Error> {
        Ok(provide!(Van))
    }
}
// ANCHOR_END: deliveries

#[role]
impl Messages for Deliveries {
    fn tell(&self, customer: &str, message: &str) {
        println!("  to {customer}: {message}");
    }
}

// ANCHOR: collections
/// Orders collected at the counter: the back room's stock, the counter's card terminal, no
/// rescue step, since the customer is there to pay another way, and putting the order aside.
#[workflow(name = "collections", policies = [ReadyToCollect], reporters = [Log])]
#[with_step(Reserve, name = "reserve")]
#[with_step(Charge, name = "charge", policies = [Audit])]
#[with_step(PutAside, name = "put_aside")]
struct Collections;

#[workflow]
impl Collections {
    #[provider(key = "stock")]
    fn back_room(&self) -> Result<Provided<dyn StockStore>, Error> {
        Ok(provide!(Store { place: "back room" }))
    }

    #[provider(key = "payments")]
    fn counter(&self) -> Result<Provided<dyn Payments>, Error> {
        Ok(provide!(COUNTER))
    }
}
// ANCHOR_END: collections

#[role]
impl Messages for Collections {
    fn tell(&self, customer: &str, message: &str) {
        println!("  to {customer}: {message}");
    }
}

/// What the shop knows about an order when the journey starts, whichever way it is fulfilled.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    card: String,
    price: i64,
}

/// An order of the day: its number, its customer, their card and its price.
type Order = (&'static str, &'static str, &'static str, i64);

const ORDERS: [Order; 2] = [
    ("A-1", "Ann", "ann-card", 40),
    ("A-2", "Ben", "ben-card", 25),
];

/// The initial data of an order of the day.
fn new_order((order, customer, card, price): Order) -> NewOrder {
    NewOrder {
        order: order.to_owned(),
        customer: customer.to_owned(),
        card: card.to_owned(),
        price,
    }
}

/// A service in a listing: its key, and the provider that serves it.
fn service(listed: &ListedService) -> String {
    format!("{} from {}", listed.key, listed.provider)
}

/// The services of a listing's entry, separated by commas.
fn services(listed: &[ListedService]) -> String {
    listed.iter().map(service).collect::<Vec<_>>().join(", ")
}

/// One line of a workflow's listing: a step, its policies, its services with the provider that
/// serves each, and its rescue step with its services.
fn line(listed: &ListedStep) -> String {
    let mut line = format!("  {}. {}", listed.position, listed.step);
    if !listed.policies.is_empty() {
        let policies: Vec<String> = listed.policies.iter().map(ToString::to_string).collect();
        line.push_str(&format!(", with {}", policies.join(", ")));
    }
    line.push_str(&format!("; services: {}", services(&listed.services)));
    if let Some(rescue) = &listed.rescue {
        line.push_str(&format!(
            "; rescued by {}, services: {}",
            rescue.step,
            services(&rescue.services)
        ));
    }
    line
}

/// How a journey ended: its status, and the reason a step gave when it failed.
fn ending(status: &JourneyStatus) -> String {
    match status {
        JourneyStatus::Failed {
            failure: Failure::Failure(reason),
            ..
        } => format!("failed: {}", reason.code()),
        status => status.kind().to_string(),
    }
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("deliveries:");
    for listed in Deliveries::descriptor().listing() {
        println!("{}", line(&listed));
    }
    println!("collections:");
    for listed in Collections::descriptor().listing() {
        println!("{}", line(&listed));
    }
    let mut executor = LocalExecutor::new();
    for order in ORDERS {
        println!("{} delivered:", order.0);
        let delivery = Deliveries::builder()
            .initial_data(new_order(order))
            .create()?;
        println!("  {}", ending(&executor.run(delivery)?.status));
        println!("{} collected:", order.0);
        let collection = Collections::builder()
            .initial_data(new_order(order))
            .create()?;
        println!("  {}", ending(&executor.run(collection)?.status));
    }
    Ok(())
}
// ANCHOR_END: run
