//! The shop declared by hand: the same workflow as the short form, written in the long form.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, DataBag, Output, OutputError};
use itinera::policy::{
    HookNeeds, OnStepSuccess, OnSuccess, OnWorkflowSuccess, Provides, Requested,
    StepPolicyDescriptor, StepSuccess, WorkflowPolicyDescriptor, WorkflowSuccess,
};
use itinera::step::{
    Input, Outcome, Reason, Resolved, Step, StepDescriptor, StepFactory, StepNeeds, step_name,
};
use itinera::workflow::{ListedStep, Violations, WorkflowDescriptor};

const PRICE: Input<i64> = Input::new("price");
const CHARGED: Input<i64> = Input::new("charged");
const ORDER: Input<String> = Input::new("order");
const CUSTOMER: Input<String> = Input::new("customer");

// ANCHOR: step
/// Charges an order's price, for one attempt.
struct Charge<'a> {
    price: i64,
    contributor: Contributor<'a>,
}

impl Step for Charge<'_> {
    fn run(mut self) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Ok(Outcome::failure(Reason::new("nothing-to-charge")));
        }
        self.contributor.contribute("charged", self.price);
        Ok(Outcome::success())
    }
}

/// Builds a charge for each attempt, from what it needs.
struct Charging;

impl StepFactory for Charging {
    type Step<'a> = Charge<'a>;

    fn needs(&self) -> StepNeeds {
        StepNeeds::new().input(&PRICE).contributor()
    }

    fn build<'a>(&'a self, got: &mut Resolved<'a>) -> Result<Charge<'a>, Error> {
        Ok(Charge {
            price: got.input(&PRICE)?,
            contributor: got.contributor()?,
        })
    }
}

/// Ships an order through the carrier: a step that needs nothing, so a function is enough.
fn ship() -> Result<Outcome, Error> {
    Ok(Outcome::success())
}
// ANCHOR_END: step

// ANCHOR: policies
/// Audits every successful charge.
struct Audit;

impl OnStepSuccess<Orders> for Audit {
    fn needs() -> HookNeeds<StepSuccess> {
        HookNeeds::new().from_step(&CHARGED).contributor()
    }

    fn on_step_success(
        &self,
        mut got: Requested<'_, Orders, StepSuccess>,
    ) -> Result<Option<OnSuccess>, Error> {
        let charged = got.from_step(&CHARGED)?;
        got.contributor()?
            .contribute("audit", format!("charged {charged}"));
        Ok(None)
    }
}

/// The shop's way of telling a customer something.
trait Messages {
    fn tell(&self, customer: &str, message: &str);
}

/// Tells the customer once the order is complete.
struct Notify;

impl OnWorkflowSuccess<Orders> for Notify {
    fn needs() -> HookNeeds<WorkflowSuccess> {
        HookNeeds::new()
            .from_workflow(&CUSTOMER)
            .from_workflow(&ORDER)
    }

    fn on_workflow_success(
        &self,
        mut got: Requested<'_, Orders, WorkflowSuccess>,
    ) -> Result<(), Error> {
        let customer = got.from_workflow(&CUSTOMER)?;
        let order = got.from_workflow(&ORDER)?;
        let messages: &dyn Messages = got.role();
        messages.tell(&customer, &format!("order {order} is complete"));
        Ok(())
    }
}
// ANCHOR_END: policies

// ANCHOR: workflow
/// The shop's workflow, which provides the `Messages` role.
struct Orders;

impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        println!("to {customer}: {message}");
    }
}

impl Provides<dyn Messages> for Orders {
    fn role(&self) -> &(dyn Messages + 'static) {
        self
    }
}

/// Declares the shop's workflow, part by part, and checks it.
fn orders() -> Result<WorkflowDescriptor<Orders>, Violations> {
    let audit = StepPolicyDescriptor::new("audit", || Audit).on_step_success();
    let notify = WorkflowPolicyDescriptor::new("notify", || Notify).on_workflow_success();
    WorkflowDescriptor::builder("orders")
        .step(StepDescriptor::new(step_name!("charge"), Charging).policy(audit))
        .step(StepDescriptor::new(step_name!("ship"), ship).retry_budget(1))
        .policy(notify)
        .build()
}
// ANCHOR_END: workflow

// ANCHOR: data
/// What the shop reads back once the journey has succeeded.
struct Summary {
    charged: i64,
    audit: String,
}

impl Output for Summary {
    fn from_data(data: &DataBag) -> Result<Self, OutputError> {
        Ok(Self {
            charged: data.required("charged")?,
            audit: data.required("audit")?,
        })
    }
}
// ANCHOR_END: data

/// One line of the workflow's listing: the step's position, its name and its policies.
fn line(listed: &ListedStep) -> String {
    let policies: Vec<String> = listed.policies.iter().map(ToString::to_string).collect();
    if policies.is_empty() {
        return format!("{}. {}", listed.position, listed.step);
    }
    format!(
        "{}. {}, with {}",
        listed.position,
        listed.step,
        policies.join(", ")
    )
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = orders()?;
    for listed in orders.listing() {
        println!("{}", line(&listed));
    }
    let instance = orders
        .instance(Orders)
        .data("order", "A-1".to_owned())
        .data("customer", "Ann".to_owned())
        .data("price", 40_i64)
        .create()?;
    let result = LocalExecutor::new().run(instance)?;
    let summary: Summary = result.output()?;
    println!("the journey {}", result.status.kind());
    println!("charged {}; audit: {}", summary.charged, summary.audit);
    Ok(())
}
// ANCHOR_END: run
