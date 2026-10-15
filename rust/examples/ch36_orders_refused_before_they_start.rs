//! Orders refused before they start: an order without a number can't be named, and the executor
//! refuses an order without a customer's name, since the log can't file it.

use std::fmt;

use itinera::error::{Error, Result};
use itinera::event::{Event, EventBody, GiveUpCause, HookSource};
use itinera::executor::local_executor_for;
use itinera::journey::{
    Abort, Contributor, DataBag, Failure, InitialData, JourneyId, JourneyResult, JourneyStatus,
    MissingData, Output,
};
use itinera::policy::{OnFailure, OnSuccess, StepFailureCause, step_policy, workflow_policy};
use itinera::report::{GlobalMetadata, Reporter};
use itinera::service::{Provided, provide};
use itinera::step::{
    Attempt, Outcome, Reason, RescueOutcome, RescueStep, Step, StepName, StepReporter, rescue_step,
    step,
};
use itinera::value::AnyValue;
use itinera::workflow::{Supplied, role, workflow};
use serde::{Deserialize, Serialize};

/// A payment client: it charges an order's amount to a card, and gives back a receipt, or nothing
/// when the card has expired, or the error of a network that failed.
trait Payments: Send + Sync {
    fn charge(&self, order: &str, card: &str, amount: i64) -> Result<Option<String>, NetworkDown>;
}

/// A card registry: it says which card replaced one, if any did.
trait Cards: Send + Sync {
    fn replacement(&self, card: &str) -> Option<String>;
}

/// The bank's own library: it connects to the bank's card registry for a region, and hands back
/// its client, boxed.
fn card_registry(region: &str) -> Box<dyn Cards> {
    Box::new(Registry {
        region: region.to_owned(),
        this_year: 2026,
    })
}

/// A carrier: it books a parcel of this weight in kg and gives back its tracking number, or the
/// error of a booking system that failed, then says whether it collects the parcel.
trait Carrier: Send + Sync {
    fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed>;
    fn collects(&self, weight: i64) -> bool;
}

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

/// The shop's bank, faked: a card is its holder's name and the year it expires, and the bank
/// declines a card that expired before this year. Its network fails on any amount over 100. It
/// prints what it does.
struct Bank {
    this_year: u32,
}

impl Payments for Bank {
    fn charge(&self, order: &str, card: &str, amount: i64) -> Result<Option<String>, NetworkDown> {
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
}

/// The bank's card registry for a region, faked: it prints what it does.
struct Registry {
    region: String,
    this_year: u32,
}

impl Cards for Registry {
    /// A card that expired in the last three years was replaced by its holder's card that expires
    /// in three years. An older one was not.
    fn replacement(&self, card: &str) -> Option<String> {
        if expiry(card)? + 3 < self.this_year {
            println!(
                "the card registry in {} says nothing replaced card {card}",
                self.region
            );
            return None;
        }
        let (holder, _) = card.rsplit_once('-')?;
        let replacement = format!("{holder}-{}", self.this_year + 3);
        println!(
            "the card registry in {} says card {card} was replaced by {replacement}",
            self.region
        );
        Some(replacement)
    }
}

/// The year a card expires: the part of it after its last `-`.
fn expiry(card: &str) -> Option<u32> {
    card.rsplit_once('-')?.1.parse().ok()
}

/// The shop's main carrier, faked: its booking system fails on a parcel that weighs nothing, and
/// it collects a parcel only up to 20 kg.
struct MainCarrier;

impl Carrier for MainCarrier {
    fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed> {
        if weight == 0 {
            println!("the main carrier's booking system fails on order {order}");
            return Err(BookingFailed);
        }
        println!("the main carrier books order {order}");
        Ok(format!("T-{order}"))
    }

    fn collects(&self, weight: i64) -> bool {
        weight <= 20
    }
}

/// The shop's backup carrier, faked: its booking system fails on a parcel that weighs nothing, and
/// it collects a parcel only up to 80 kg.
struct BackupCarrier;

impl Carrier for BackupCarrier {
    fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed> {
        if weight == 0 {
            println!("the backup carrier's booking system fails on order {order}");
            return Err(BookingFailed);
        }
        println!("the backup carrier books order {order}");
        Ok(format!("B-{order}"))
    }

    fn collects(&self, weight: i64) -> bool {
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
    fn run(
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
    fn looked_up(&self, #[from_step] completed: bool) -> Result<OnSuccess> {
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

#[step]
impl Price {
    fn run(self, contributor: &mut Contributor) -> Result<Outcome> {
        contributor.contribute("amount", self.price - self.discount.unwrap_or(0));
        Self::succeed()
    }
}

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
    fn run(
        self,
        #[service] payments: &dyn Payments,
        contributor: &mut Contributor,
        mut reporter: StepReporter<'_>,
    ) -> Result<Outcome> {
        if self.amount <= 0 {
            return Self::fail("nothing-to-charge");
        }
        let Some(receipt) = payments.charge(&self.order, &self.card, self.amount)? else {
            return Self::fail("card-expired");
        };
        reporter.set_global_metadata("receipt", receipt.clone())?;
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
    #[input("card")]
    expired: String,
    reason: Reason,
}

#[rescue_step]
impl RenewCard {
    fn run(
        self,
        #[service("cards")] registry: &dyn Cards,
        contributor: &mut Contributor,
    ) -> Result<RescueOutcome> {
        if self.reason.code() != "card-expired" {
            return Self::fail("not-repairable");
        }
        let Some(card) = registry.replacement(&self.expired) else {
            return Self::fail("not-replaced");
        };
        contributor.contribute("card", card);
        Self::succeed()
    }
}

/// Writes an audit line for every step it is attached to that succeeds: the journey, the attempt
/// that took the money, the amount the order was charged, and the receipt the step gave back.
#[derive(Default)]
struct Audit;

#[step_policy(name = "audit")]
impl Audit {
    #[on_step_success]
    fn charged(
        &self,
        journey: &JourneyId,
        step: StepName,
        attempt: Attempt,
        #[from_workflow] amount: i64,
        #[from_step] receipt: String,
        contributor: &mut Contributor,
    ) -> Result<OnSuccess> {
        contributor.contribute(
            "audit",
            format!("{journey}: {step} took {amount} on attempt {attempt}, receipt {receipt}"),
        );
        Self::keep_default()
    }
}

/// What the shop does for its policies: tell a customer something.
trait Messages {
    fn tell(&self, customer: &str, message: &str);
}

/// What the shop does for its policies: give an order's money back.
trait Refunds {
    fn refund(&self, receipt: &str, amount: i64);
}

/// Tells the customer how their order went, once its journey has ended.
#[derive(Default)]
struct Notify;

#[workflow_policy(name = "notify")]
impl Notify {
    #[on_workflow_success]
    fn succeeded(
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
        messages.tell(&customer, &message);
        Ok(())
    }

    #[on_workflow_failure]
    fn failed(
        &self,
        #[from_workflow] customer: String,
        #[from_workflow] order: String,
        #[role] messages: &dyn Messages,
    ) -> Result<()> {
        messages.tell(
            &customer,
            &format!("Sorry, order {order} could not be completed."),
        );
        Ok(())
    }
}

/// Gives up a step that ends in an error at once: the bank's network is down, and neither a retry
/// nor a rescue can mend that. Since the bank may have taken the money before its network failed,
/// it writes an audit line with the error, for the accountants to check with the bank.
#[derive(Default)]
struct Outage;

#[step_policy(name = "outage")]
impl Outage {
    #[on_step_abnormal_termination]
    fn ended_in_error(
        &self,
        journey: &JourneyId,
        step: StepName,
        attempt: Attempt,
        error: &Error,
        #[from_workflow] amount: i64,
        contributor: &mut Contributor,
    ) -> Result<OnFailure> {
        contributor.contribute(
            "audit",
            format!(
                "{journey}: {step} ended in an error on attempt {attempt} ({error}); \
                 check whether the bank took {amount}"
            ),
        );
        Self::fail_workflow("payment-network-down")
    }
}

/// Deals with a shipping that fails for good. It gives up a parcel over the shop's limit rather
/// than try it again, since the limit is the same on every attempt, whichever carrier comes. And
/// whatever gave the shipping up, it refunds the order's charge.
#[derive(Default)]
struct FailedShipping;

#[step_policy(name = "failed_shipping")]
impl FailedShipping {
    #[on_step_retry]
    fn before_retrying(
        &self,
        reason: Option<Reason>,
        #[from_workflow] receipt: String,
        #[from_workflow] amount: i64,
        #[role] refunds: &dyn Refunds,
        contributor: &mut Contributor,
    ) -> Result<OnFailure> {
        if reason.as_ref().map(Reason::code) == Some("over-parcel-limit") {
            self.refund(refunds, contributor, &receipt, amount, "over-parcel-limit");
            return Self::fail_workflow("no-carrier-can-take-it");
        }
        Self::keep_default()
    }

    #[on_step_failure]
    fn given_up(
        &self,
        cause: StepFailureCause,
        #[from_workflow] receipt: String,
        #[from_workflow] amount: i64,
        #[role] refunds: &dyn Refunds,
        contributor: &mut Contributor,
    ) -> Result<OnFailure> {
        self.refund(refunds, contributor, &receipt, amount, &cause.to_string());
        Self::keep_default()
    }

    /// Gives the order's money back, and records why.
    fn refund(
        &self,
        refunds: &dyn Refunds,
        contributor: &mut Contributor,
        receipt: &str,
        amount: i64,
        why: &str,
    ) {
        refunds.refund(receipt, amount);
        contributor.contribute("refund", format!("{amount} back on {receipt}: {why}"));
    }
}

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

#[step]
impl Ship {
    fn run(
        self,
        #[service] carrier: &dyn Carrier,
        contributor: &mut Contributor,
        mut reporter: StepReporter<'_>,
    ) -> Result<Outcome> {
        if self.gift_card {
            return Self::skip_because("nothing-to-ship");
        }
        if self.weight > 80 {
            return Self::fail("over-parcel-limit");
        }
        let tracking = carrier.book(&self.order, self.weight)?;
        contributor.contribute("tracking", tracking);
        if !carrier.collects(self.weight) {
            return Self::fail("too-heavy");
        }
        reporter.info(format!("the parcel leaves with receipt {}", self.receipt))?;
        Self::succeed()
    }
}

/// The logging service's error: it can't take a line.
#[derive(Debug)]
struct LoggingDown;

impl fmt::Display for LoggingDown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the logging service is down")
    }
}

impl std::error::Error for LoggingDown {}

/// The logging service's error when a journey's lines have no customer to be filed under.
#[derive(Debug)]
struct NoCustomer;

impl fmt::Display for NoCustomer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the logging service files no line without a customer's name")
    }
}

impl std::error::Error for NoCustomer {}

/// The shop's logging service, faked: it prints each line it takes. It is down when the line
/// saying that order A-15's charge succeeded reaches it.
#[derive(Default)]
struct LoggingService;

impl LoggingService {
    /// Files the journey's lines under a customer's name, which must not be empty.
    fn file_under(&self, customer: &str) -> Result<(), NoCustomer> {
        if customer.is_empty() {
            return Err(NoCustomer);
        }
        Ok(())
    }

    fn write(&self, line: String) -> Result<(), LoggingDown> {
        if line.contains(" A-15 ") && line.ends_with(" step_succeeded charge") {
            return Err(LoggingDown);
        }
        println!("{line}");
        Ok(())
    }
}

/// The shop's log: it writes a line for every event of a journey to the logging service, starting
/// with the workflow's name, the journey's ID, the customer's name and, once the bank has given
/// it, the receipt. A line the service refuses fails the report.
#[derive(Default)]
struct Log {
    service: LoggingService,
    metadata: GlobalMetadata,
}

impl Reporter for Log {
    // ANCHOR: log
    fn set_global_metadata(&mut self, key: &str, value: &AnyValue) -> Result<()> {
        if key == "customer" {
            let customer = value.downcast_ref::<String>().map_or("", String::as_str);
            self.service.file_under(customer)?;
        }
        self.metadata.insert(key, value.clone());
        Ok(())
    }
    // ANCHOR_END: log

    fn report(&mut self, event: &Event) -> Result<()> {
        let line = format!(
            "log {} {} {} #{} {}",
            event.workflow,
            event.journey_id,
            self.whose(),
            event.sequence,
            event.kind()
        );
        let line = match detail(&event.body) {
            Some(detail) => format!("{line} {detail}"),
            None => line,
        };
        self.service.write(line)?;
        Ok(())
    }
}

impl Log {
    /// The customer the journey is for and, once the bank has given it, the charge's receipt.
    fn whose(&self) -> String {
        let customer = self.text("customer").unwrap_or("someone");
        match self.text("receipt") {
            Some(receipt) => format!("for {customer}, receipt {receipt}"),
            None => format!("for {customer}"),
        }
    }

    /// The entry of `key`, when it is text.
    fn text(&self, key: &str) -> Option<&str> {
        let value = self.metadata.get(key)?;
        value.downcast_ref::<String>().map(String::as_str)
    }
}

/// The data a provider supplied with a service, when it is text.
fn supplied(data: Option<&AnyValue>) -> String {
    data.and_then(|data| data.downcast_ref::<String>())
        .map(|data| format!(", with {data}"))
        .unwrap_or_default()
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
            data,
            ..
        } => Some(format!(
            "{} gets {key} from {provider}, a {implementation}{}",
            step.step,
            supplied(data.as_ref())
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
        EventBody::GlobalMetadataSet { key, .. } => Some(key.clone()),
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

#[role]
impl Refunds for Orders {
    fn refund(&self, receipt: &str, amount: i64) {
        println!("the bank gives {amount} back on receipt {receipt}");
    }
}

#[role]
impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        println!("{} to {customer}: {message}", self.channel);
    }
}

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

    /// The bank's card registry in Europe, for the charge's rescue step only.
    #[provider(steps = ["charge_rescue"])]
    fn cards(&self) -> Result<Provided<dyn Cards>> {
        Ok(Provided::from_box(card_registry("eu")).with_data("eu".to_owned()))
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

// ANCHOR: data
/// What the shop knows about an order when its journey starts. An order with no card leaves it
/// out.
#[derive(InitialData)]
struct NewOrder {
    order: Option<String>,
    customer: String,
    price: i64,
    discount: Option<i64>,
    parcel: Parcel,
    card: Option<String>,
    gift_card: bool,
}
// ANCHOR_END: data

/// How the warehouse describes an order's parcel: its weight in kg, and its length, width and
/// height in cm.
#[derive(Clone, Serialize, Deserialize)]
struct Parcel {
    weight: i64,
    size: [i64; 3],
}

/// What the shop reads back once the journey has ended.
#[derive(Output)]
struct Order {
    receipt: Option<String>,
    tracking: Option<String>,
    audit: Option<String>,
}

/// Says how an order's journey ended, and what explains it.
fn follow_up(result: &JourneyResult) -> Result<(), Box<dyn std::error::Error>> {
    match &result.status {
        JourneyStatus::Succeeded { .. } => {
            let data: Order = result.output()?;
            let receipt = data.receipt.as_deref().unwrap_or("none");
            let tracking = data.tracking.as_deref().unwrap_or("none");
            println!("the journey succeeded, with receipt {receipt} and tracking {tracking}");
            println!("audit: {}", data.audit.as_deref().unwrap_or("none"));
        }
        JourneyStatus::Failed { failure, .. } => {
            let data: Order = result.output()?;
            let receipt = data.receipt.as_deref().unwrap_or("none");
            println!(
                "the journey failed ({}), with receipt {receipt}",
                why(failure)
            );
            println!("audit: {}", data.audit.as_deref().unwrap_or("none"));
        }
        JourneyStatus::Aborted(abort) => {
            println!("the journey was aborted ({})", what_went_wrong(abort));
        }
        _ => println!("the journey ended as {}", result.status.kind()),
    }
    Ok(())
}

/// The cause of a failure, with the reason a step gave for it when it gave one.
fn why(failure: &Failure) -> String {
    match failure {
        Failure::Failure(reason) => format!("{}: {}", failure.cause(), reason.code()),
        _ => failure.cause().to_string(),
    }
}

/// The reason of an abort, with the key that was missing or the reporter's error when that is what
/// happened.
fn what_went_wrong(abort: &Abort) -> String {
    match abort {
        Abort::RequiredDataMissing(MissingData::Key { key, .. }) => {
            format!("{}: {key}", abort.reason())
        }
        Abort::ReporterFailed(error) => format!("{}: {error}", abort.reason()),
        _ => abort.reason().to_string(),
    }
}

/// What `main` prints before an order: whose it is and its number, or what it lacks.
fn heading(order: Option<&str>, customer: &str) -> String {
    match (order, customer) {
        (None, _) => format!("{customer}'s order, without a number"),
        (Some(order), "") => format!("order {order}, without a customer's name"),
        (Some(order), _) => format!("{customer}'s order {order}"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut executor = local_executor_for!(Orders);
    // The order, if it has a number, the customer, the price, the discount, the parcel's weight in
    // kg and its size in cm, the card, if the order has one, and whether the order is a gift card.
    let orders = [
        (
            None,
            "Quin",
            20,
            None,
            1,
            [20, 10, 10],
            Some("quin-2027"),
            false,
        ),
        (
            Some("A-16"),
            "",
            25,
            None,
            1,
            [20, 10, 10],
            Some("rae-2028"),
            false,
        ),
    ];
    for (order, customer, price, discount, weight, size, card, gift_card) in orders {
        println!("{}:", heading(order, customer));
        // ANCHOR: run
        let created = Orders::builder()
            .channel("email")
            .reporters_global_metadata("customer", customer.to_owned())
            .initial_data(NewOrder {
                order: order.map(str::to_owned),
                customer: customer.to_owned(),
                price,
                discount,
                parcel: Parcel { weight, size },
                card: card.map(str::to_owned),
                gift_card,
            })
            .create();
        let instance = match created {
            Ok(instance) => instance,
            Err(error) => {
                println!("no instance: {error}");
                continue;
            }
        };
        match executor.run(instance) {
            Ok(result) => follow_up(&result)?,
            Err(refusal) => println!("the journey was refused: {refusal}"),
        }
        // ANCHOR_END: run
    }
    Ok(())
}
