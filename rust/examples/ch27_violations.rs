//! A workflow put together wrongly is refused with every violation, before any journey.

use itinera::error::Error;
use itinera::journey::Contributor;
use itinera::policy::{HookNeeds, InputAdapter, OnSuccess, Requested, StepPolicy, step_policy};
use itinera::step::{
    DescribedStep, Input, OptionalInput, Outcome, Step, StepName, step, step_name,
};
use itinera::value::AnyValue;
use itinera::workflow::{InputAdapterDescriptor, WorkflowDescriptor};

/// Charges an order's price.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        contributor.contribute("charged", self.price);
        Self::succeed()
    }
}

/// Ships an order through the carrier.
#[derive(Step)]
struct Ship;

#[step]
impl Ship {
    fn run(self) -> Result<Outcome, Error> {
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

/// Writes a receipt for every successful charge.
#[derive(Default)]
struct Receipts;

#[step_policy(name = "receipts")]
impl Receipts {
    #[on_step_success]
    fn written(
        &self,
        #[from_step] charged: i64,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<OnSuccess>, Error> {
        contributor.contribute("receipt", format!("R-{charged}"));
        Self::carry_on()
    }
}

/// The shop's workflow, which holds nothing of its own.
struct Orders;

const PRICE: Input<i64> = Input::new("price");
const DISCOUNT: OptionalInput<i64> = OptionalInput::new("discount");

/// Works out the price to charge: the price, less any discount.
fn pricing(
    _: &Orders,
    mut got: Requested<'_, Orders, InputAdapter>,
) -> Result<Option<AnyValue>, Error> {
    let price = got.from_workflow(&PRICE)?;
    let discount = got.optional_from_workflow(&DISCOUNT)?;
    Ok(Some(AnyValue::new(price - discount.unwrap_or_default())))
}

/// Declares the pricing adapter, attached to the step of this name.
fn pricing_for(step: StepName) -> InputAdapterDescriptor<Orders> {
    InputAdapterDescriptor::new("pricing", step, pricing).needing(
        HookNeeds::new()
            .from_workflow(&PRICE)
            .optional_from_workflow(&DISCOUNT),
    )
}

fn main() {
    // ANCHOR: wrong
    let refused = WorkflowDescriptor::builder("orders")
        .step(
            Charge::descriptor(step_name!("charge"))
                .policy(Audit::descriptor())
                .policy(Receipts::descriptor()),
        )
        .step(Ship::descriptor(step_name!("ship")))
        .step(Ship::descriptor(step_name!("ship")).retry_budget(1))
        .input_adapter(pricing_for(step_name!("charging")))
        .build();
    // ANCHOR_END: wrong
    // ANCHOR: report
    match refused {
        Ok(_) => println!("the workflow was built"),
        Err(violations) => {
            println!("the workflow was refused:");
            for violation in &violations {
                println!("  {violation}");
            }
        }
    }
    // ANCHOR_END: report
    // ANCHOR: fixed
    let built = WorkflowDescriptor::builder("orders")
        .step(Charge::descriptor(step_name!("charge")).policy(Audit::descriptor()))
        .step(Ship::descriptor(step_name!("ship")).retry_budget(1))
        .input_adapter(pricing_for(step_name!("charge")))
        .build();
    match built {
        Ok(orders) => {
            println!("the fixed workflow was built:");
            for listed in orders.listing() {
                println!("  {}. {}", listed.position, listed.step);
            }
        }
        Err(violations) => println!("still refused: {violations}"),
    }
    // ANCHOR_END: fixed
}
