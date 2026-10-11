//! What happens when it runs: a small shop runs one journey per order, its reporter prints every
//! event, and the result says how each journey ended.

use itinera::error::Error;
use itinera::event::{Event, EventBody, HookSource};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, DataBag, InitialData};
use itinera::policy::{OnSuccess, step_policy, workflow_policy};
use itinera::report::Reporter;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::{role, workflow};

/// The warehouse, faked: it has stock for every order.
struct Warehouse;

impl Warehouse {
    fn reserve(&self, _order: &str) -> bool {
        true
    }
}

/// The customers' bank, faked: it refuses the cards it lists.
struct Bank {
    refused: &'static [&'static str],
}

const BANK: Bank = Bank {
    refused: &["ben-card"],
};

impl Bank {
    fn accepts(&self, card: &str) -> bool {
        !self.refused.contains(&card)
    }
}

/// Reserves the order's goods in the warehouse.
#[derive(Step)]
struct Reserve {
    #[input]
    order: String,
}

#[step]
impl Reserve {
    fn run(self) -> Result<Outcome, Error> {
        if Warehouse.reserve(&self.order) {
            Self::succeed()
        } else {
            Self::fail("out-of-stock")
        }
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
        if !BANK.accepts(&self.card) {
            return Self::fail("declined");
        }
        contributor.contribute("charged", self.price - self.discount.unwrap_or(0));
        Self::succeed()
    }
}

/// Ships the order through the carrier, which hands back a tracking number.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
}

#[step]
impl Ship {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        contributor.contribute("tracking", format!("T-{}", self.order));
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
        #[from_step] charged: i64,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<OnSuccess>, Error> {
        contributor.contribute("audit", format!("charged {charged}"));
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

// ANCHOR: reporter
/// Prints every event of every journey, one line each.
#[derive(Default)]
struct Log;

impl Reporter for Log {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        let detail = detail(&event.body);
        println!("  {} {}{detail}", event.sequence, event.kind());
        Ok(())
    }
}
// ANCHOR_END: reporter

/// What the log shows of an event, besides its kind.
fn detail(body: &EventBody) -> String {
    match body {
        EventBody::JourneyStarted { initial_keys, .. } => format!(": {}", initial_keys.join(", ")),
        EventBody::AttemptStarted { step, .. } => {
            format!(": {}, attempt {}", step.step, step.attempt)
        }
        EventBody::OptionalInputAbsent { key, .. } => format!(": {key}"),
        EventBody::StepSucceeded { step, .. } => format!(": {}", step.step),
        EventBody::StepFailed { step, reason, .. } => format!(": {}, {}", step.step, reason.code()),
        EventBody::StepGivenUp { step, cause, .. } => format!(": {}, {cause}", step.step),
        EventBody::ContributionCommitted { key, .. } => format!(": {key}"),
        EventBody::HookCalled { hook, .. } => format!(": {}", hook_name(hook)),
        EventBody::JourneyFailed { step, failure, .. } => {
            format!(": {step}, {}", failure.cause())
        }
        _ => String::new(),
    }
}

/// A hook, named after its policy.
fn hook_name(hook: &HookSource) -> String {
    match hook {
        HookSource::Step { policy, hook, .. } => format!("{policy}, {hook}"),
        HookSource::Workflow { policy, hook, .. } => format!("{policy}, {hook}"),
        _ => String::new(),
    }
}

// ANCHOR: workflow
/// The shop's workflow: reserve, charge and ship, then tell the customer.
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

// ANCHOR: data
/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    card: String,
    price: i64,
    discount: Option<i64>,
}
// ANCHOR_END: data

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        ("A-1", "Ann", "ann-card", 40, Some(5)),
        ("A-2", "Ben", "ben-card", 15, None),
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
        println!("the journey {}", result.status.kind());
        if let Some(data) = result.status.data() {
            println!("its data bag holds: {}", keys(data));
        }
    }
    Ok(())
}
// ANCHOR_END: run

/// The keys of a data bag, in order.
fn keys(data: &DataBag) -> String {
    data.keys().collect::<Vec<_>>().join(", ")
}
