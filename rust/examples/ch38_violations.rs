//! Declaring by hand: a workflow put together wrongly is refused with every violation, before any
//! journey.

use itinera::error::Result;
use itinera::journey::Contributor;
use itinera::policy::{HookNeeds, OnSuccess, Requested, StepPolicy, step_policy};
use itinera::step::{DescribedStep, Input, Outcome, Step, StepName, step, step_name};
use itinera::workflow::{InputAdapter, InputAdapterDescriptor, Supplied, WorkflowDescriptor};
use serde::{Deserialize, Serialize};

// ANCHOR: shop
/// Charges an order's price.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
}

#[step]
impl Charge {
    fn run(self, contributor: &mut Contributor) -> Result<Outcome> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        contributor.contribute("charged", self.price);
        Self::succeed()
    }
}

/// Ships an order's parcel, of this weight in kg, through the carrier.
#[derive(Step)]
struct Ship {
    #[input]
    #[expect(dead_code, reason = "the shipping is declared, never run")]
    weight: i64,
}

#[step]
impl Ship {
    fn run(self) -> Result<Outcome> {
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
        contributor: &mut Contributor,
    ) -> Result<OnSuccess> {
        contributor.contribute("audit", format!("charged {charged}"));
        Self::keep_default()
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
        contributor: &mut Contributor,
    ) -> Result<OnSuccess> {
        contributor.contribute("receipt", format!("R-{charged}"));
        Self::keep_default()
    }
}

/// The shop's workflow, which holds nothing of its own.
struct Orders;

/// How the warehouse describes an order's parcel: its weight in kg, and its size in cm.
#[derive(Clone, Serialize, Deserialize)]
struct Parcel {
    weight: i64,
    size: [i64; 3],
}

const PARCEL: Input<Parcel> = Input::new("parcel");

/// Gives the shipping the weight of the order's parcel. Every other input is read from the data
/// bag.
fn parcels(_: &Orders, mut got: Requested<'_, Orders, InputAdapter>) -> Result<Supplied> {
    if got.key() != "weight" {
        return Ok(Supplied::FromDataBag);
    }
    Ok(Supplied::value(got.from_workflow(&PARCEL)?.weight))
}

/// Declares the parcels adapter, attached to the step of this name.
fn parcels_for(step: StepName) -> InputAdapterDescriptor<Orders> {
    InputAdapterDescriptor::new("parcels", step, parcels)
        .needing(HookNeeds::new().from_workflow(&PARCEL))
}
// ANCHOR_END: shop

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
        .input_adapter(parcels_for(step_name!("shipping")))
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
        .input_adapter(parcels_for(step_name!("ship")))
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
