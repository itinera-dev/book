//! The shop declared by hand, asynchronous: the same workflow, run on the asynchronous executor.

use itinera::error::Error;
use itinera::executor::AsyncLocalExecutor;
use itinera::journey::{Contributor, DataBag, Output, OutputError};
use itinera::mode::Asynchronous;
use itinera::policy::{
    AsyncOnStepSuccess, AsyncOnWorkflowSuccess, HookNeeds, OnSuccess, Provides, Requested,
    StepPolicyDescriptor, StepSuccess, WorkflowPolicyDescriptor, WorkflowSuccess,
};
use itinera::step::{
    AsyncStep, AsyncStepFactory, Input, Outcome, Reason, Resolved, StepDescriptor, StepNeeds,
    step_name,
};
use itinera::workflow::{ListedStep, Violations, WorkflowDescriptor};

const PRICE: Input<i64> = Input::new("price");
const CHARGED: Input<i64> = Input::new("charged");
const ORDER: Input<String> = Input::new("order");

// ANCHOR: step
/// Charges an order's price, for one attempt.
struct Charge<'a> {
    price: i64,
    contributor: Contributor<'a>,
}

impl AsyncStep for Charge<'_> {
    async fn run(mut self) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Ok(Outcome::failure(Reason::new("nothing-to-charge")));
        }
        self.contributor.contribute("charged", self.price);
        Ok(Outcome::success())
    }
}

/// Builds a charge for each attempt, from what it needs.
struct Charging;

impl AsyncStepFactory for Charging {
    type Step<'a> = Charge<'a>;

    fn needs(&self) -> StepNeeds {
        StepNeeds::new().input(&PRICE).contributor()
    }

    fn build<'a>(&'a self, got: &mut Resolved<'a, Asynchronous>) -> Result<Charge<'a>, Error> {
        Ok(Charge {
            price: got.input(&PRICE)?,
            contributor: got.contributor()?,
        })
    }
}

/// Ships an order through the carrier: a step that needs nothing, so a function is enough.
async fn ship() -> Result<Outcome, Error> {
    Ok(Outcome::success())
}
// ANCHOR_END: step

// ANCHOR: policies
/// Audits every successful charge.
struct Audit;

impl AsyncOnStepSuccess<Orders> for Audit {
    fn needs() -> HookNeeds<StepSuccess> {
        HookNeeds::new().from_step(&CHARGED).contributor()
    }

    async fn on_step_success(
        &self,
        mut got: Requested<'_, Orders, StepSuccess, Asynchronous>,
    ) -> Result<Option<OnSuccess>, Error> {
        let charged = got.from_step(&CHARGED)?;
        got.contributor()?
            .contribute("audit", format!("charged {charged}"));
        Ok(None)
    }
}

/// The shop's way of telling a customer something.
trait Messages: Sync {
    fn tell(&self, order: &str, message: &str);
}

/// Tells the customer once the order is complete.
struct Notify;

impl AsyncOnWorkflowSuccess<Orders> for Notify {
    fn needs() -> HookNeeds<WorkflowSuccess> {
        HookNeeds::new().from_workflow(&ORDER)
    }

    async fn on_workflow_success(
        &self,
        mut got: Requested<'_, Orders, WorkflowSuccess, Asynchronous>,
    ) -> Result<(), Error> {
        let order = got.from_workflow(&ORDER)?;
        let messages: &dyn Messages = got.role();
        messages.tell(&order, "your order is complete");
        Ok(())
    }
}
// ANCHOR_END: policies

// ANCHOR: workflow
/// The shop's workflow, which provides the `Messages` role.
struct Orders;

impl Messages for Orders {
    fn tell(&self, order: &str, message: &str) {
        println!("to the customer of {order}: {message}");
    }
}

impl Provides<dyn Messages> for Orders {
    fn role(&self) -> &(dyn Messages + 'static) {
        self
    }
}

/// Declares the shop's asynchronous workflow, part by part, and checks it.
fn orders() -> Result<WorkflowDescriptor<Orders, Asynchronous>, Violations> {
    let audit = StepPolicyDescriptor::new_async("audit", || Audit).on_step_success();
    let notify = WorkflowPolicyDescriptor::new_async("notify", || Notify).on_workflow_success();
    WorkflowDescriptor::async_builder("orders")
        .step(StepDescriptor::new_async(step_name!("charge"), Charging).policy(audit))
        .step(StepDescriptor::new_async(step_name!("ship"), ship).retry_budget(1))
        .policy(notify)
        .build()
}
// ANCHOR_END: workflow

/// What the shop reads back once the journey has succeeded.
struct Receipt {
    charged: i64,
    audit: String,
}

impl Output for Receipt {
    fn from_data(data: &DataBag) -> Result<Self, OutputError> {
        Ok(Self {
            charged: data.required("charged")?,
            audit: data.required("audit")?,
        })
    }
}

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
        .data("price", 40_i64)
        .create()?;
    let result = futures::executor::block_on(AsyncLocalExecutor::new().run(instance))?;
    let receipt: Receipt = result.output()?;
    println!("the journey {}", result.status.kind());
    println!("charged {}; audit: {}", receipt.charged, receipt.audit);
    Ok(())
}
// ANCHOR_END: run
