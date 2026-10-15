//! Running async journeys while others wait: the shop's bank and carriers answer over the
//! network, so the shop runs asynchronously, and its orders go forward together.

use std::fmt;

use itinera::error::Result;
use itinera::event::{Event, EventBody, GiveUpCause, HookSource};
use itinera::executor::{Refusal, local_executor_for};
use itinera::instance::Instance;
use itinera::journey::{Contributor, DataBag, InitialData, JourneyResult};
use itinera::mode::{Asynchronous, async_trait};
use itinera::policy::{OnFailure, OnSuccess, StepFailureCause, step_policy, workflow_policy};
use itinera::report::Reporter;
use itinera::service::{Provided, provide};
use itinera::step::{
    AsyncStepReporter, Attempt, Outcome, Reason, RescueOutcome, RescueStep, Step, StepName,
    rescue_step, step,
};
use itinera::workflow::{Supplied, role, workflow};
use serde::{Deserialize, Serialize};

// ANCHOR: services
/// A payment client: it charges an order's amount to a card, and gives back a receipt, or nothing
/// when the card has expired, or the error of a network that failed; and it says which card
/// replaced one, if any did. Each answer comes over the network.
#[async_trait]
trait Payments: Send + Sync {
    async fn charge(
        &self,
        order: &str,
        card: &str,
        amount: i64,
    ) -> Result<Option<String>, NetworkDown>;
    async fn replacement(&self, card: &str) -> Option<String>;
}

/// A carrier: it books a parcel of this weight in kg and gives back its tracking number, or the
/// error of a booking system that failed, then says whether it collects the parcel. Each answer
/// comes over the network.
#[async_trait]
trait Carrier: Send + Sync {
    async fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed>;
    async fn collects(&self, weight: i64) -> bool;
}
// ANCHOR_END: services

/// The shop's order book: it says whether an order was already completed.
trait OrderBook: Send + Sync {
    fn completed(&self, order: &str) -> bool;
}

/// The error of the bank's network when it fails.
#[derive(Debug)]
struct NetworkDown;

impl fmt::Display for NetworkDown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the payment network is down")
    }
}

impl std::error::Error for NetworkDown {}

/// The error of the carrier's booking system when it fails.
#[derive(Debug)]
struct BookingFailed;

impl fmt::Display for BookingFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the booking system failed")
    }
}

impl std::error::Error for BookingFailed {}

// ANCHOR: network
/// The network the fakes answer over: it hands the thread back once before the answer comes, as a
/// real call does while it waits.
async fn network() {
    tokio::task::yield_now().await;
}

/// The shop's bank, faked: a card is its holder's name and the year it expires, and the bank
/// declines a card that expired before this year. Its network fails on any amount over 100. It
/// prints what it does once it has answered.
struct Bank {
    this_year: u32,
}

#[async_trait]
impl Payments for Bank {
    async fn charge(
        &self,
        order: &str,
        card: &str,
        amount: i64,
    ) -> Result<Option<String>, NetworkDown> {
        network().await;
        if amount > 100 {
            println!("the bank's network fails while charging order {order}");
            return Err(NetworkDown);
        }
        if expiry(card).is_some_and(|year| year < self.this_year) {
            println!("the bank declines card {card}: it has expired");
            return Ok(None);
        }
        println!("the bank charges {amount} for order {order}");
        Ok(Some(format!("R-{order}")))
    }

    /// A card that expired in the last three years was replaced by its holder's card that expires
    /// in three years. An older one was not.
    async fn replacement(&self, card: &str) -> Option<String> {
        network().await;
        if expiry(card)? + 3 < self.this_year {
            println!("the bank says nothing replaced card {card}");
            return None;
        }
        let (holder, _) = card.rsplit_once('-')?;
        let replacement = format!("{holder}-{}", self.this_year + 3);
        println!("the bank says card {card} was replaced by {replacement}");
        Some(replacement)
    }
}
// ANCHOR_END: network

/// The year a card expires: the part of it after its last `-`.
fn expiry(card: &str) -> Option<u32> {
    card.rsplit_once('-')?.1.parse().ok()
}

/// The shop's main carrier, faked: its booking system fails on a parcel that weighs nothing, and
/// it collects a parcel only up to 20 kg.
struct MainCarrier;

#[async_trait]
impl Carrier for MainCarrier {
    async fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed> {
        network().await;
        if weight == 0 {
            println!("the main carrier's booking system fails on order {order}");
            return Err(BookingFailed);
        }
        println!("the main carrier books order {order}");
        Ok(format!("T-{order}"))
    }

    async fn collects(&self, weight: i64) -> bool {
        network().await;
        weight <= 20
    }
}

/// The shop's backup carrier, faked: its booking system fails on a parcel that weighs nothing, and
/// it collects a parcel only up to 80 kg.
struct BackupCarrier;

#[async_trait]
impl Carrier for BackupCarrier {
    async fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed> {
        network().await;
        if weight == 0 {
            println!("the backup carrier's booking system fails on order {order}");
            return Err(BookingFailed);
        }
        println!("the backup carrier books order {order}");
        Ok(format!("B-{order}"))
    }

    async fn collects(&self, weight: i64) -> bool {
        network().await;
        weight <= 80
    }
}

/// The shop's order book, faked: it holds the orders completed before today, and prints what it
/// finds.
struct Ledger {
    completed: Vec<&'static str>,
}

impl OrderBook for Ledger {
    fn completed(&self, order: &str) -> bool {
        if self.completed.contains(&order) {
            println!("the order book has order {order} completed already");
            return true;
        }
        println!("the order book has no order {order} yet");
        false
    }
}

/// Looks the order up in the order book, and gives back whether it was already completed.
#[derive(Step)]
struct Lookup {
    #[input]
    order: String,
}

#[step]
impl Lookup {
    async fn run(
        self,
        #[service] order_book: &dyn OrderBook,
        contributor: &mut Contributor,
    ) -> Result<Outcome> {
        contributor.contribute("completed", order_book.completed(&self.order));
        Self::succeed()
    }
}

/// Finishes the journey of an order that was already completed: there is nothing left to do for
/// it.
#[derive(Default)]
struct AlreadyCompleted;

#[step_policy(name = "already_completed")]
impl AlreadyCompleted {
    #[on_step_success]
    async fn looked_up(&self, #[from_step] completed: bool) -> Result<OnSuccess> {
        if completed {
            return Self::finish_workflow();
        }
        Self::keep_default()
    }
}

/// Works out what an order costs: its price, less its discount if it has one, given back as the
/// amount to charge.
#[derive(Step)]
struct Price {
    #[input]
    price: i64,
    #[input]
    discount: Option<i64>,
}

// ANCHOR: price
#[step]
impl Price {
    async fn run(self, contributor: &mut Contributor) -> Result<Outcome> {
        contributor.contribute("amount", self.price - self.discount.unwrap_or(0));
        Self::succeed()
    }
}
// ANCHOR_END: price

/// Charges an order's amount to its card through the payment client, and gives back its receipt,
/// and the gift card's code when the order is a gift card.
#[derive(Step)]
struct Charge {
    #[input]
    order: String,
    #[input]
    amount: i64,
    #[input]
    card: String,
    #[input]
    gift_card: bool,
}

#[step]
impl Charge {
    async fn run(
        self,
        #[service] payments: &dyn Payments,
        contributor: &mut Contributor,
    ) -> Result<Outcome> {
        if self.amount <= 0 {
            return Self::fail("nothing-to-charge");
        }
        let Some(receipt) = payments
            .charge(&self.order, &self.card, self.amount)
            .await?
        else {
            return Self::fail("card-expired");
        };
        contributor.contribute("receipt", receipt);
        if self.gift_card {
            contributor.contribute("code", format!("G-{}", self.order));
        }
        Self::succeed()
    }
}

/// Repairs a charge that failed because its card had expired: it asks the bank for the card that
/// replaced it, and gives it back as the order's card.
#[derive(RescueStep)]
struct RenewCard {
    #[input]
    card: String,
    reason: Reason,
}

#[rescue_step]
impl RenewCard {
    async fn run(
        self,
        #[service] payments: &dyn Payments,
        contributor: &mut Contributor,
    ) -> Result<RescueOutcome> {
        if self.reason.code() != "card-expired" {
            return Self::fail("not-repairable");
        }
        let Some(card) = payments.replacement(&self.card).await else {
            return Self::fail("not-replaced");
        };
        contributor.contribute("card", card);
        Self::succeed()
    }
}

/// Writes an audit line for every step it is attached to that succeeds: the amount the order was
/// charged, and the receipt the step gave back.
#[derive(Default)]
struct Audit;

#[step_policy(name = "audit")]
impl Audit {
    #[on_step_success]
    async fn charged(
        &self,
        step: StepName,
        #[from_workflow] amount: i64,
        #[from_step] receipt: String,
        contributor: &mut Contributor,
    ) -> Result<OnSuccess> {
        contributor.contribute("audit", format!("{step} took {amount}, receipt {receipt}"));
        Self::keep_default()
    }
}

// ANCHOR: roles
/// What the shop does for its policies: tell a customer something.
#[async_trait]
trait Messages: Sync {
    async fn tell(&self, customer: &str, message: &str);
}

/// What the shop does for its policies: give an order's money back.
#[async_trait]
trait Refunds: Sync {
    async fn refund(&self, receipt: &str, amount: i64);
}
// ANCHOR_END: roles

/// Tells the customer how their order went, once its journey has ended.
#[derive(Default)]
struct Notify;

#[workflow_policy(name = "notify")]
impl Notify {
    #[on_workflow_success]
    async fn succeeded(
        &self,
        #[from_workflow] customer: String,
        #[from_workflow] order: String,
        #[from_workflow] tracking: Option<String>,
        #[from_workflow] code: Option<String>,
        #[role] messages: &dyn Messages,
    ) -> Result<()> {
        let message = match (tracking, code) {
            (Some(tracking), _) => format!("Order {order} is on its way, tracking {tracking}."),
            (None, Some(code)) => {
                format!("Order {order} is paid: your gift card's code is {code}.")
            }
            (None, None) => format!("Order {order} is complete."),
        };
        messages.tell(&customer, &message).await;
        Ok(())
    }

    #[on_workflow_failure]
    async fn failed(
        &self,
        #[from_workflow] customer: String,
        #[from_workflow] order: String,
        #[role] messages: &dyn Messages,
    ) -> Result<()> {
        messages
            .tell(
                &customer,
                &format!("Sorry, order {order} could not be completed."),
            )
            .await;
        Ok(())
    }
}

/// Gives up a step that ends in an error at once: the bank's network is down, and neither a retry
/// nor a rescue can mend that.
#[derive(Default)]
struct Outage;

#[step_policy(name = "outage")]
impl Outage {
    #[on_step_abnormal_termination]
    async fn ended_in_error(&self) -> Result<OnFailure> {
        Self::fail_workflow("payment-network-down")
    }
}

// ANCHOR: failed_shipping
/// Deals with a shipping that fails for good. It gives up a parcel over the shop's limit rather
/// than try it again, since the limit is the same on every attempt, whichever carrier comes. And
/// whatever gave the shipping up, it refunds the order's charge.
#[derive(Default)]
struct FailedShipping;

#[step_policy(name = "failed_shipping")]
impl FailedShipping {
    #[on_step_retry]
    async fn before_retrying(
        &self,
        reason: Option<Reason>,
        #[from_workflow] receipt: String,
        #[from_workflow] amount: i64,
        #[role] refunds: &dyn Refunds,
        contributor: &mut Contributor,
    ) -> Result<OnFailure> {
        if reason.as_ref().map(Reason::code) == Some("over-parcel-limit") {
            self.refund(refunds, contributor, &receipt, amount, "over-parcel-limit")
                .await;
            return Self::fail_workflow("no-carrier-can-take-it");
        }
        Self::keep_default()
    }

    #[on_step_failure]
    async fn given_up(
        &self,
        cause: StepFailureCause,
        #[from_workflow] receipt: String,
        #[from_workflow] amount: i64,
        #[role] refunds: &dyn Refunds,
        contributor: &mut Contributor,
    ) -> Result<OnFailure> {
        self.refund(refunds, contributor, &receipt, amount, &cause.to_string())
            .await;
        Self::keep_default()
    }

    /// Gives the order's money back, and records why.
    async fn refund(
        &self,
        refunds: &dyn Refunds,
        contributor: &mut Contributor,
        receipt: &str,
        amount: i64,
        why: &str,
    ) {
        refunds.refund(receipt, amount).await;
        contributor.contribute("refund", format!("{amount} back on {receipt}: {why}"));
    }
}
// ANCHOR_END: failed_shipping

/// Ships an order through the carrier, with its receipt in the parcel, gives back its tracking
/// number, and says when the parcel has left. A parcel the carrier won't collect may be taken on
/// another attempt. A gift card has nothing to ship, and the shop ships no parcel over 80 kg.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    weight: i64,
    #[input]
    receipt: String,
    #[input]
    gift_card: bool,
}

// ANCHOR: ship
#[step]
impl Ship {
    async fn run(
        self,
        #[service] carrier: &dyn Carrier,
        contributor: &mut Contributor,
        mut reporter: AsyncStepReporter<'_>,
    ) -> Result<Outcome> {
        if self.gift_card {
            return Self::skip_because("nothing-to-ship");
        }
        if self.weight > 80 {
            return Self::fail("over-parcel-limit");
        }
        let tracking = carrier.book(&self.order, self.weight).await?;
        contributor.contribute("tracking", tracking);
        if !carrier.collects(self.weight).await {
            return Self::fail("too-heavy");
        }
        reporter
            .info(format!("the parcel leaves with receipt {}", self.receipt))
            .await?;
        Self::succeed()
    }
}
// ANCHOR_END: ship

/// The shop's log: it writes a line for every event of a journey, starting with the workflow's
/// name and the journey's ID.
#[derive(Default)]
struct Log;

impl Reporter for Log {
    fn report(&mut self, event: &Event) -> Result<()> {
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
        EventBody::StepFailed { step, reason, .. } => {
            Some(format!("{}: {}", step.step, reason.code()))
        }
        EventBody::StepSkipped {
            step,
            reason: Some(reason),
            ..
        } => Some(format!("{}: {}", step.step, reason.code())),
        EventBody::ContributionsDiscarded { step, .. } => Some(step.step.to_string()),
        EventBody::StepAbnormalTermination { step, message, .. } => {
            Some(format!("{}: {message}", step.step))
        }
        EventBody::StepGivenUp {
            step,
            cause: GiveUpCause::FailWorkflow {
                decided_by, reason, ..
            },
            ..
        } => Some(format!(
            "{}: FailWorkflow from {}, {}",
            step.step,
            decided_by.policy,
            reason.code()
        )),
        EventBody::StepGivenUp { step, cause, .. } => Some(format!("{}: {cause}", step.step)),
        EventBody::StepRescuing {
            step,
            cause,
            rescue,
            ..
        } => Some(format!("{}: {cause}, rescued by {rescue}", step.step)),
        EventBody::StepRescued { step, rescue, .. } => Some(format!("{} by {rescue}", step.step)),
        EventBody::HookCalled {
            hook: HookSource::Step {
                policy, hook, step, ..
            },
            ..
        } => Some(format!("{policy}: {hook} of {}", step.step)),
        EventBody::HookCalled {
            hook: HookSource::Workflow { policy, hook, .. },
            ..
        } => Some(format!("{policy}: {hook}")),
        EventBody::OptionalInputAbsent { key, .. } => Some(key.clone()),
        EventBody::InputAdapterSupplied {
            step, key, adapter, ..
        } => Some(format!("{} gets {key} from {adapter}", step.step)),
        EventBody::ContributionCommitted { key, .. } | EventBody::DataOverwritten { key, .. } => {
            Some(key.clone())
        }
        EventBody::StepInfo { step, message, .. } => Some(format!("{}: {message}", step.step)),
        EventBody::JourneyFailed { step, .. } => Some(step.to_string()),
        EventBody::JourneySucceeded {
            decided_by: Some(policy),
            ..
        } => Some(format!("FinishWorkflow from {policy}")),
        _ => None,
    }
}

/// The customer service desk: of all the events of a journey, it keeps only a failed journey's.
#[derive(Default)]
struct Desk;

impl Reporter for Desk {
    fn report(&mut self, event: &Event) -> Result<()> {
        if let EventBody::JourneyFailed { step, .. } = &event.body {
            println!("desk: order {} failed at {step}", event.journey_id);
        }
        Ok(())
    }
}

/// The shop's workflow: it looks the order up, and finishes an order already completed. Otherwise
/// it works out what the order costs, then charges it, repairing a charge that fails and auditing
/// one that succeeds, then ships, and shipping may be tried once more, its charge refunded if it
/// is given up.
/// Once a journey has ended, the customer is told, through the channel the shop configures.
#[workflow(name = "orders", reporters = [Log, Desk], policies = [Notify])]
#[with_step(Lookup, name = "lookup", policies = [AlreadyCompleted])]
#[with_step(Price, name = "price")]
#[with_step(Charge, name = "charge", rescue = RenewCard, policies = [Audit, Outage])]
#[with_step(Ship, name = "ship", retry_budget = 1, policies = [FailedShipping])]
struct Orders {
    channel: &'static str,
}

// ANCHOR: role_impls
#[role]
#[async_trait]
impl Refunds for Orders {
    async fn refund(&self, receipt: &str, amount: i64) {
        network().await;
        println!("the bank gives {amount} back on receipt {receipt}");
    }
}

#[role]
#[async_trait]
impl Messages for Orders {
    async fn tell(&self, customer: &str, message: &str) {
        network().await;
        println!("{} to {customer}: {message}", self.channel);
    }
}
// ANCHOR_END: role_impls

#[workflow]
impl Orders {
    /// Names each journey after its order.
    #[id_generator]
    fn order_number(&self, data: &DataBag) -> Result<String> {
        Ok(data.required("order")?)
    }

    /// Gives the shipping the weight of the order's parcel, as the warehouse describes it. Every
    /// other input of the shipping is read from the data bag.
    #[input_adapter(steps = ["ship"])]
    fn parcels(&self, #[key] key: &str, #[from_workflow] parcel: Parcel) -> Result<Supplied> {
        if key != "weight" {
            return Self::from_data_bag();
        }
        Self::supply(parcel.weight)
    }

    /// The order book, which has the orders completed before today.
    #[provider(key = "order_book")]
    fn ledger(&self) -> Result<Provided<dyn OrderBook>> {
        Ok(provide!(Ledger {
            completed: vec!["A-13"],
        }))
    }

    /// The bank, as the payment client of the journey.
    #[provider(key = "payments")]
    fn bank(&self) -> Result<Provided<dyn Payments>> {
        Ok(provide!(Bank { this_year: 2026 }))
    }

    /// A carrier for every attempt of a step: the main carrier first, the backup carrier after.
    #[provider(key = "carrier", per_attempt)]
    fn carriers(&self, attempt: Attempt) -> Result<Provided<dyn Carrier>> {
        if attempt.is_first() {
            return Ok(provide!(MainCarrier));
        }
        Ok(provide!(BackupCarrier))
    }
}

/// What the shop knows about an order when its journey starts. An order with no card leaves it
/// out.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    price: i64,
    discount: Option<i64>,
    parcel: Parcel,
    card: Option<String>,
    gift_card: bool,
}

/// How the warehouse describes an order's parcel: its weight in kg, and its length, width and
/// height in cm.
#[derive(Clone, Serialize, Deserialize)]
struct Parcel {
    weight: i64,
    size: [i64; 3],
}

// ANCHOR: run
/// An instance of the shop's workflow for an order paid with this card.
fn order(
    order: &str,
    customer: &str,
    price: i64,
    discount: Option<i64>,
    parcel: Parcel,
    card: &str,
) -> Result<Instance<Orders, Asynchronous>, Box<dyn std::error::Error>> {
    Ok(Orders::builder()
        .channel("email")
        .initial_data(NewOrder {
            order: order.to_owned(),
            customer: customer.to_owned(),
            price,
            discount,
            parcel,
            card: Some(card.to_owned()),
            gift_card: false,
        })
        .create()?)
}

/// Runs an order's journey with an executor of its own.
async fn journey(instance: Instance<Orders, Asynchronous>) -> Result<JourneyResult, Refusal> {
    local_executor_for!(Orders).run(instance).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (ann, gil, jo) = tokio::join!(
        journey(order(
            "A-1",
            "Ann",
            40,
            Some(10),
            Parcel {
                weight: 2,
                size: [30, 20, 10],
            },
            "ann-2028",
        )?),
        journey(order(
            "A-7",
            "Gil",
            45,
            None,
            Parcel {
                weight: 60,
                size: [120, 80, 60],
            },
            "gil-2027",
        )?),
        journey(order(
            "A-8",
            "Jo",
            30,
            None,
            Parcel {
                weight: 1,
                size: [20, 15, 5],
            },
            "jo-2024",
        )?),
    );
    for result in [ann?, gil?, jo?] {
        println!("order {} {}", result.journey_id, result.status.kind());
    }
    Ok(())
}
// ANCHOR_END: run
