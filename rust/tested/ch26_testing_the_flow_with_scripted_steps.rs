//! Testing the flow with scripted steps: the shop of chapter 23, with tests that run its workflow
//! with what its steps do written in a script, so they check the flow without the steps' rules.

use std::fmt;
use std::num::NonZeroU32;

use itinera::error::Error;
use itinera::event::{Event, EventBody, GiveUpCause, HookSource};
use itinera::executor::LocalExecutor;
use itinera::journey::{
    Abort, Contributor, DataBag, Failure, InitialData, JourneyResult, JourneyStatus, MissingData,
    Output,
};
use itinera::policy::{FailWorkflow, OnSuccess, step_policy, workflow_policy};
use itinera::report::Reporter;
use itinera::service::{Provided, provide};
use itinera::step::{
    Outcome, Reason, RescueOutcome, RescueStep, Step, StepName, StepReporter, rescue_step, step,
};
use itinera::value::AnyValue;
use itinera::workflow::{role, workflow};
use serde::{Deserialize, Serialize};

/// A payment client: it charges an order's amount to a card, and gives back a receipt, or nothing
/// when the card has expired, or the error of a network that failed; and it says which card
/// replaced one, if any did.
trait Payments: Send + Sync {
    fn charge(&self, order: &str, card: &str, amount: i64) -> Result<Option<String>, NetworkDown>;
    fn replacement(&self, card: &str) -> Option<String>;
}

/// A carrier: it books a parcel of this weight in kg and gives back its tracking number, or the
/// error of a booking system that failed, then says whether it collects the parcel.
trait Carrier: Send + Sync {
    fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed>;
    fn collects(&self, weight: i64) -> bool;
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

    /// A card that expired in the last three years was replaced by its holder's card that expires
    /// in three years. An older one was not.
    fn replacement(&self, card: &str) -> Option<String> {
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

/// The year a card expires: the part of it after its last `-`.
fn expiry(card: &str) -> Option<u32> {
    card.rsplit_once('-')?.1.parse().ok()
}

/// The shop's main carrier, faked: its booking system fails on a parcel over 100 kg, and it
/// collects a parcel only up to 20 kg.
struct MainCarrier;

impl Carrier for MainCarrier {
    fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed> {
        if weight > 100 {
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

/// The shop's backup carrier, faked: its booking system fails on a parcel over 100 kg, and it
/// collects a parcel only up to 80 kg.
struct BackupCarrier;

impl Carrier for BackupCarrier {
    fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed> {
        if weight > 100 {
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
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
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
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        if self.amount <= 0 {
            return Self::fail("nothing-to-charge");
        }
        let Some(receipt) = payments.charge(&self.order, &self.card, self.amount)? else {
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

/// Writes an audit line for every step it is attached to that succeeds: the amount the order was
/// charged, and the receipt the step gave back.
#[derive(Default)]
struct Audit;

#[step_policy(name = "audit")]
impl Audit {
    #[on_step_success]
    fn charged(
        &self,
        #[step_name] step: StepName,
        #[from_workflow] amount: i64,
        #[from_step] receipt: String,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<OnSuccess>, Error> {
        contributor.contribute("audit", format!("{step} took {amount}, receipt {receipt}"));
        Self::carry_on()
    }
}

/// What the shop does for its policies: tell a customer something.
trait Messages {
    fn tell(&self, customer: &str, message: &str);
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
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
        messages.tell(
            &customer,
            &format!("Sorry, order {order} could not be completed."),
        );
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
    fn ended_in_error(&self) -> Result<Option<FailWorkflow>, Error> {
        Self::fail_workflow("payment-network-down")
    }
}

/// Gives up the shipping of a parcel over the shop's limit, rather than try it again: the limit
/// is the same on every attempt, whichever carrier comes.
#[derive(Default)]
struct HeavyParcels;

#[step_policy(name = "heavy_parcels")]
impl HeavyParcels {
    #[on_step_retry]
    fn before_retrying(
        &self,
        #[reason] reason: Option<Reason>,
    ) -> Result<Option<FailWorkflow>, Error> {
        if reason.as_ref().map(Reason::code) == Some("over-parcel-limit") {
            return Self::fail_workflow("no-carrier-can-take-it");
        }
        Self::carry_on()
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
        #[contributor] mut contributor: Contributor<'_>,
        #[reporter] mut reporter: StepReporter<'_>,
    ) -> Result<Outcome, Error> {
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

/// The shop's workflow: it works out what the order costs, then charges it, repairing a charge
/// that fails and auditing one that succeeds, then ships, and shipping may be tried once more.
/// Once a journey has ended, the customer is told, through the channel the shop configures.
#[workflow(name = "orders", reporters = [Log, Desk], policies = [Notify])]
#[with_step(Price, name = "price")]
#[with_step(Charge, name = "charge", rescue = RenewCard, policies = [Audit, Outage])]
#[with_step(Ship, name = "ship", retry_budget = 1, policies = [HeavyParcels])]
struct Orders {
    channel: &'static str,
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
    fn order_number(&self, data: &DataBag) -> Result<String, Error> {
        Ok(data.required("order")?)
    }

    /// Gives the shipping the weight of the order's parcel, as the warehouse describes it. Every
    /// other input of the shipping is read from the data bag.
    #[input_adapter(steps = ["ship"])]
    fn parcels(
        &self,
        #[key] key: &str,
        #[from_workflow] parcel: Parcel,
    ) -> Result<Option<AnyValue>, Error> {
        if key != "weight" {
            return Self::from_data_bag();
        }
        Self::supply(parcel.weight)
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

/// The reason of an abort, with the key that was missing when that is what happened.
fn what_went_wrong(abort: &Abort) -> String {
    match abort {
        Abort::RequiredDataMissing(MissingData::Key { key, .. }) => {
            format!("{}: {key}", abort.reason())
        }
        _ => abort.reason().to_string(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut executor = LocalExecutor::new();
    // The order, the customer, the price, the discount, the parcel's weight in kg and its size in
    // cm, the card, if the order has one, and whether the order is a gift card.
    let orders = [
        (
            "A-1",
            "Ann",
            40,
            Some(10),
            2,
            [30, 20, 10],
            Some("ann-2028"),
            false,
        ),
        (
            "A-7",
            "Gil",
            45,
            None,
            60,
            [120, 80, 60],
            Some("gil-2027"),
            false,
        ),
        (
            "A-8",
            "Jo",
            30,
            None,
            1,
            [20, 15, 5],
            Some("jo-2024"),
            false,
        ),
        ("A-9", "Kim", 25, None, 0, [0, 0, 0], Some("kim-2027"), true),
        (
            "A-10",
            "Lee",
            35,
            None,
            3,
            [40, 30, 20],
            Some("lee-2019"),
            false,
        ),
        ("A-11", "Max", 20, None, 1, [20, 10, 5], None, false),
    ];
    for (order, customer, price, discount, weight, size, card, gift_card) in orders {
        println!("{customer}'s order:");
        let instance = Orders::builder()
            .channel("email")
            .initial_data(NewOrder {
                order: order.to_owned(),
                customer: customer.to_owned(),
                price,
                discount,
                parcel: Parcel { weight, size },
                card: card.map(str::to_owned),
                gift_card,
            })
            .create()?;
        let result = executor.run(instance)?;
        follow_up(&result)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use itinera::instance::{Instance, WorkflowInstance};
    use itinera::journey::{Read, ScriptedRun};

    use super::*;

    // ANCHOR: order
    /// An instance of the shop's workflow for an order of 30, with this card and this parcel.
    fn order(order: &str, card: &str, weight: i64) -> Result<Instance<Orders>, Error> {
        Ok(Orders::builder()
            .channel("email")
            .initial_data(NewOrder {
                order: order.to_owned(),
                customer: "Ann".to_owned(),
                price: 30,
                discount: None,
                parcel: Parcel {
                    weight,
                    size: [20, 15, 5],
                },
                card: Some(card.to_owned()),
                gift_card: false,
            })
            .create()?)
    }
    // ANCHOR_END: order

    // ANCHOR: rescued
    #[test]
    fn a_declined_card_is_renewed_and_the_charge_tried_again_with_it() -> Result<(), Error> {
        let run = order("A-1", "ann-2024", 1)?
            .scripted()
            .step("charge")
            .will_fail("card-expired")
            .on_attempt(2)
            .will_contribute("receipt", "R-A-1".to_owned())
            .will_succeed()
            .and_step("charge_rescue")
            .will_contribute("card", "ann-2029".to_owned())
            .will_succeed()
            .and_step("ship")
            .will_contribute("tracking", "T-A-1".to_owned())
            .will_succeed()
            .run()?;
        assert!(matches!(run.result.status, JourneyStatus::Succeeded { .. }));
        let retried = run.step("charge").attempt(2);
        assert_eq!(
            retried.map(|attempt| attempt.input::<String>("card")),
            Some(Read::Present("ann-2029".to_owned()))
        );
        Ok(())
    }
    // ANCHOR_END: rescued

    // ANCHOR: given_up
    /// The reason a policy failed the journey with, if one did.
    fn failed_by_a_policy(status: &JourneyStatus) -> Option<&str> {
        match status {
            JourneyStatus::Failed {
                failure: Failure::FailWorkflow(reason),
                ..
            } => Some(reason.code()),
            _ => None,
        }
    }

    /// Runs an order of 60 kg whose shipping fails on attempt 1 with this reason, and would
    /// succeed on attempt 2.
    fn shipping_that_fails_with(reason: &str) -> Result<ScriptedRun, Error> {
        Ok(order("A-2", "ann-2028", 60)?
            .scripted()
            .step("charge")
            .will_contribute("receipt", "R-A-2".to_owned())
            .will_succeed()
            .and_step("ship")
            .will_fail(reason)
            .on_attempt(2)
            .will_contribute("tracking", "B-A-2".to_owned())
            .will_succeed()
            .run()?)
    }

    #[test]
    fn a_shipping_that_fails_with_too_heavy_is_tried_again() -> Result<(), Error> {
        let run = shipping_that_fails_with("too-heavy")?;
        assert_eq!(run.step("ship").attempts().len(), 2);
        assert!(matches!(run.result.status, JourneyStatus::Succeeded { .. }));
        Ok(())
    }

    #[test]
    fn a_shipping_that_fails_with_over_parcel_limit_is_given_up_after_attempt_1()
    -> Result<(), Error> {
        let run = shipping_that_fails_with("over-parcel-limit")?;
        assert_eq!(run.step("ship").attempts().len(), 1);
        assert_eq!(
            failed_by_a_policy(&run.result.status),
            Some("no-carrier-can-take-it")
        );
        Ok(())
    }

    #[test]
    fn a_scripted_shipping_is_still_built_with_the_weight_the_adapter_supplies() -> Result<(), Error>
    {
        let run = shipping_that_fails_with("too-heavy")?;
        let first = run.step("ship").attempt(1);
        assert_eq!(
            first.map(|attempt| attempt.input::<i64>("weight")),
            Some(Read::Present(60))
        );
        Ok(())
    }
    // ANCHOR_END: given_up

    // ANCHOR: outage
    #[test]
    fn a_charge_that_ends_in_an_error_is_given_up_and_never_rescued() -> Result<(), Error> {
        let run = order("A-3", "ann-2028", 1)?
            .scripted()
            .step("charge")
            .will_terminate_abnormally("the payment network is down")
            .on_attempt(2)
            .will_contribute("receipt", "R-A-3".to_owned())
            .will_succeed()
            .and_step("charge_rescue")
            .will_succeed()
            .run()?;
        assert!(run.step("charge_rescue").attempts().is_empty());
        assert_eq!(
            failed_by_a_policy(&run.result.status),
            Some("payment-network-down")
        );
        Ok(())
    }
    // ANCHOR_END: outage
}
