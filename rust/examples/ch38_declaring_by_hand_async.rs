//! The shop declared by hand, in the asynchronous form: the same workflow as the short form, its
//! parts given to the workflow one by one, with its pricing step and its audit written without any
//! attribute.

use std::fmt;

use itinera::error::{Error, Result};
use itinera::event::{Event, EventBody, GiveUpCause, HookSource};
use itinera::executor::{AsyncLocalExecutor, Refusal};
use itinera::instance::Instance;
use itinera::journey::{Contributor, DataBag, InitialData, JourneyId, JourneyResult, Output};
use itinera::mode::{Asynchronous, async_trait};
use itinera::policy::{
    AsyncOnStepSuccess, HookNeeds, OnFailure, OnSuccess, Requested, StepFailureCause, StepPolicy,
    StepPolicyDescriptor, StepSuccess, WorkflowPolicy, step_policy, workflow_policy,
};
use itinera::report::{AsyncReporter, GlobalMetadata, Reporter};
use itinera::service::{
    PerAttempt, PerJourney, Provided, Provider, Service, ServiceProviderDescriptor, provide,
};
use itinera::step::{
    AsyncStep, AsyncStepFactory, AsyncStepReporter, Attempt, DescribedStep, Input, OptionalInput,
    Outcome, Reason, RescueOutcome, RescueStep, Resolved, Step, StepDescriptor, StepName,
    StepNeeds, rescue_step, step, step_name,
};
use itinera::value::AnyValue;
use itinera::workflow::{
    InputAdapter, InputAdapterDescriptor, Provides, Supplied, Violations, WorkflowDescriptor,
};
use serde::{Deserialize, Serialize};

/// A payment client: it charges an order's amount to a card, and gives back a receipt, or nothing
/// when the card has expired, or the error of a network that failed. Each answer comes over the
/// network.
#[async_trait]
trait Payments: Send + Sync {
    async fn charge(
        &self,
        order: &str,
        card: &str,
        amount: i64,
    ) -> Result<Option<String>, NetworkDown>;
}

/// A card registry: it says which card replaced one, if any did. Its answer comes over the network.
#[async_trait]
trait Cards: Send + Sync {
    async fn replacement(&self, card: &str) -> Option<String>;
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
/// error of a booking system that failed, then says whether it collects the parcel. Each answer
/// comes over the network.
#[async_trait]
trait Carrier: Send + Sync {
    async fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed>;
    async fn collects(&self, weight: i64) -> bool;
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
}

/// The bank's card registry for a region, faked: it prints what it does.
struct Registry {
    region: String,
    this_year: u32,
}

#[async_trait]
impl Cards for Registry {
    /// A card that expired in the last three years was replaced by its holder's card that expires
    /// in three years. An older one was not.
    async fn replacement(&self, card: &str) -> Option<String> {
        network().await;
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

// ANCHOR: price
const PRICE: Input<i64> = Input::new("price");
const DISCOUNT: OptionalInput<i64> = OptionalInput::new("discount");

/// Works out what an order costs, for one attempt: its price, less its discount if it has one,
/// given back as the amount to charge.
struct Price<'a> {
    price: i64,
    discount: Option<i64>,
    contributor: &'a mut Contributor,
}

impl AsyncStep for Price<'_> {
    async fn run(self) -> Result<Outcome> {
        let amount = self.price - self.discount.unwrap_or(0);
        self.contributor.contribute("amount", amount);
        Ok(Outcome::success())
    }
}

/// Builds the pricing for each attempt, from what it needs.
struct Pricing;

impl AsyncStepFactory for Pricing {
    type Step<'a> = Price<'a>;

    fn needs(&self) -> StepNeeds {
        StepNeeds::new()
            .input(&PRICE)
            .input(&DISCOUNT)
            .contributor()
    }

    fn build<'a>(&'a self, got: &mut Resolved<'a, Asynchronous>) -> Result<Price<'a>> {
        Ok(Price {
            price: got.input(&PRICE)?,
            discount: got.input(&DISCOUNT)?,
            contributor: got.contributor()?,
        })
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
        mut reporter: AsyncStepReporter<'_>,
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
        reporter
            .set_global_metadata("receipt", receipt.clone())
            .await?;
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
    async fn run(
        self,
        #[service("cards")] registry: &dyn Cards,
        contributor: &mut Contributor,
    ) -> Result<RescueOutcome> {
        if self.reason.code() != "card-expired" {
            return Self::fail("not-repairable");
        }
        let Some(card) = registry.replacement(&self.expired).await else {
            return Self::fail("not-replaced");
        };
        contributor.contribute("card", card);
        Self::succeed()
    }
}

// ANCHOR: audit
const AMOUNT: Input<i64> = Input::new("amount");
const RECEIPT: Input<String> = Input::new("receipt");

/// Writes an audit line for every step it is attached to that succeeds: the journey, the attempt
/// that took the money, the amount the order was charged, and the receipt the step gave back.
struct Audit;

impl AsyncOnStepSuccess<Orders> for Audit {
    fn needs() -> HookNeeds<StepSuccess> {
        HookNeeds::new()
            .from_workflow(&AMOUNT)
            .from_step(&RECEIPT)
            .contributor()
    }

    async fn on_step_success(
        &self,
        mut got: Requested<'_, Orders, StepSuccess, Asynchronous>,
    ) -> Result<OnSuccess> {
        let journey = got.journey_id();
        let step = got.step_name();
        let attempt = got.attempt();
        let amount = got.from_workflow(&AMOUNT)?;
        let receipt = got.from_step(&RECEIPT)?;
        got.contributor()?.contribute(
            "audit",
            format!("{journey}: {step} took {amount} on attempt {attempt}, receipt {receipt}"),
        );
        Ok(OnSuccess::KeepDefault)
    }
}
// ANCHOR_END: audit

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
/// nor a rescue can mend that. Since the bank may have taken the money before its network failed,
/// it writes an audit line with the error, for the accountants to check with the bank.
#[derive(Default)]
struct Outage;

#[step_policy(name = "outage")]
impl Outage {
    #[on_step_abnormal_termination]
    async fn ended_in_error(
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

/// The shop's logging service, faked: it prints each line it takes, once the network has
/// answered. It is down when the line saying that order A-15's charge succeeded reaches it.
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

    async fn write(&self, line: String) -> Result<(), LoggingDown> {
        network().await;
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

impl AsyncReporter for Log {
    fn set_global_metadata(&mut self, key: &str, value: &AnyValue) -> Result<()> {
        if key == "customer" {
            let customer = value.downcast_ref::<String>().map_or("", String::as_str);
            self.service.file_under(customer)?;
        }
        self.metadata.insert(key, value.clone());
        Ok(())
    }

    async fn report(&mut self, event: &Event) -> Result<()> {
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
        self.service.write(line).await?;
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

// ANCHOR: parts
/// The shop's workflow, whose own value is the channel the shop tells its customers through.
struct Orders {
    channel: &'static str,
}

#[async_trait]
impl Refunds for Orders {
    async fn refund(&self, receipt: &str, amount: i64) {
        network().await;
        println!("the bank gives {amount} back on receipt {receipt}");
    }
}

impl Provides<dyn Refunds> for Orders {
    fn role(&self) -> &(dyn Refunds + 'static) {
        self
    }
}

#[async_trait]
impl Messages for Orders {
    async fn tell(&self, customer: &str, message: &str) {
        network().await;
        println!("{} to {customer}: {message}", self.channel);
    }
}

impl Provides<dyn Messages> for Orders {
    fn role(&self) -> &(dyn Messages + 'static) {
        self
    }
}

const PARCEL: Input<Parcel> = Input::new("parcel");

impl Orders {
    /// Names each journey after its order.
    fn order_number(&self, data: &DataBag) -> Result<String> {
        Ok(data.required("order")?)
    }

    /// Gives the shipping the weight of the order's parcel, as the warehouse describes it. Every
    /// other input of the shipping is read from the data bag.
    fn parcels(&self, mut got: Requested<'_, Self, InputAdapter>) -> Result<Supplied> {
        if got.key() != "weight" {
            return Ok(Supplied::FromDataBag);
        }
        let parcel = got.from_workflow(&PARCEL)?;
        Ok(Supplied::value(parcel.weight))
    }

    /// The order book, which has the orders completed before today.
    async fn ledger(
        &self,
        _: Requested<'_, Self, Provider<PerJourney>, Asynchronous>,
    ) -> Result<Provided<dyn OrderBook>> {
        Ok(provide!(Ledger {
            completed: vec!["A-13"],
        }))
    }

    /// The bank, as the payment client of the journey.
    async fn bank(
        &self,
        _: Requested<'_, Self, Provider<PerJourney>, Asynchronous>,
    ) -> Result<Provided<dyn Payments>> {
        Ok(provide!(Bank { this_year: 2026 }))
    }

    /// The bank's card registry in Europe, for the charge's rescue step only.
    async fn cards(
        &self,
        _: Requested<'_, Self, Provider<PerJourney>, Asynchronous>,
    ) -> Result<Provided<dyn Cards>> {
        Ok(Provided::from_box(card_registry("eu")).with_data("eu".to_owned()))
    }

    /// A carrier for every attempt of a step: the main carrier first, the backup carrier after.
    async fn carriers(
        &self,
        got: Requested<'_, Self, Provider<PerAttempt>, Asynchronous>,
    ) -> Result<Provided<dyn Carrier>> {
        if got.attempt().is_first() {
            return Ok(provide!(MainCarrier));
        }
        Ok(provide!(BackupCarrier))
    }
}
// ANCHOR_END: parts

// ANCHOR: workflow
const ORDER_BOOK: Service<dyn OrderBook> = Service::new("order_book");
const PAYMENTS: Service<dyn Payments> = Service::new("payments");
const CARDS: Service<dyn Cards> = Service::new("cards");
const CARRIER: Service<dyn Carrier> = Service::new("carrier");

/// Declares the shop's asynchronous workflow, part by part, and checks it. It looks the order up,
/// and finishes an order already completed. Otherwise it works out what the order costs, then
/// charges it, repairing a charge that fails and auditing one that succeeds, then ships, and
/// shipping may be tried once more, its charge refunded if it is given up. Once a journey has
/// ended, the customer is told.
fn orders() -> Result<WorkflowDescriptor<Orders, Asynchronous>, Violations> {
    let audit = StepPolicyDescriptor::new_async("audit", || Audit).on_step_success();
    let parcels = InputAdapterDescriptor::new("parcels", step_name!("ship"), Orders::parcels)
        .needing(HookNeeds::new().from_workflow(&PARCEL));
    let cards = ServiceProviderDescriptor::new_async("cards", &CARDS, Orders::cards)
        .step(step_name!("charge_rescue"));
    WorkflowDescriptor::async_builder("orders")
        .step(Lookup::descriptor(step_name!("lookup")).policy(AlreadyCompleted::descriptor()))
        .step(StepDescriptor::new_async(step_name!("price"), Pricing))
        .step(
            Charge::descriptor(step_name!("charge"))
                .policy(audit)
                .policy(Outage::descriptor())
                .rescued_by::<RenewCard>(),
        )
        .step(
            Ship::descriptor(step_name!("ship"))
                .retry_budget(1)
                .policy(FailedShipping::descriptor()),
        )
        .policy(Notify::descriptor())
        .reporter::<Desk>()
        .async_reporter::<Log>()
        .id_generator(Orders::order_number)
        .input_adapter(parcels)
        .provider(ServiceProviderDescriptor::new_async(
            "ledger",
            &ORDER_BOOK,
            Orders::ledger,
        ))
        .provider(ServiceProviderDescriptor::new_async(
            "bank",
            &PAYMENTS,
            Orders::bank,
        ))
        .provider(cards)
        .provider(ServiceProviderDescriptor::per_attempt_async(
            "carriers",
            &CARRIER,
            Orders::carriers,
        ))
        .build()
}
// ANCHOR_END: workflow

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

/// How the warehouse describes an order's parcel: its weight in kg, and its length, width and
/// height in cm.
#[derive(Clone, Serialize, Deserialize)]
struct Parcel {
    weight: i64,
    size: [i64; 3],
}

// ANCHOR: instance
/// An instance of the shop's workflow for an order paid with this card.
fn order(
    orders: &WorkflowDescriptor<Orders, Asynchronous>,
    order: Option<&str>,
    customer: &str,
    price: i64,
    discount: Option<i64>,
    parcel: Parcel,
    card: &str,
) -> Result<Instance<Orders, Asynchronous>, Box<dyn std::error::Error>> {
    Ok(orders
        .instance(Orders { channel: "email" })
        .reporters_global_metadata("customer", customer.to_owned())
        .initial_data(NewOrder {
            order: order.map(str::to_owned),
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
    AsyncLocalExecutor::new().run(instance).await
}
// ANCHOR_END: instance

/// What the shop reads back once the journey has ended: the audit's line.
#[derive(Output)]
struct Audited {
    audit: Option<String>,
}

/// What `main` prints before an order: whose it is and its number, or what it lacks.
fn heading(order: Option<&str>, customer: &str) -> String {
    match (order, customer) {
        (None, _) => format!("{customer}'s order, without a number"),
        (Some(order), "") => format!("order {order}, without a customer's name"),
        (Some(order), _) => format!("{customer}'s order {order}"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = orders()?;
    // The order, if it has a number, the customer, the price, the discount, the parcel's weight in
    // kg and its size in cm, and the card.
    let placed = [
        (
            Some("A-1"),
            "Ann",
            40,
            Some(10),
            2,
            [30, 20, 10],
            "ann-2028",
        ),
        (Some("A-8"), "Jo", 30, None, 1, [20, 15, 5], "jo-2024"),
    ];
    for (number, customer, price, discount, weight, size, card) in placed {
        println!("{}:", heading(number, customer));
        let parcel = Parcel { weight, size };
        let instance = match order(&orders, number, customer, price, discount, parcel, card) {
            Ok(instance) => instance,
            Err(error) => {
                println!("no instance: {error}");
                continue;
            }
        };
        match journey(instance).await {
            Ok(result) => {
                let data: Audited = result.output()?;
                println!("order {} {}", result.journey_id, result.status.kind());
                println!("audit: {}", data.audit.as_deref().unwrap_or("none"));
            }
            Err(refusal) => println!("the journey was refused: {refusal}"),
        }
    }
    Ok(())
}
