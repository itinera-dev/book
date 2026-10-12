//! Deciding where a step's data comes from: the workflow works out the amount to charge, the price
//! less any discount, and the charge never knows.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::step::{Outcome, Step, step};
use itinera::value::AnyValue;
use itinera::workflow::workflow;

// ANCHOR: step
/// Charges an amount for an order.
#[derive(Step)]
struct Charge {
    #[input]
    order: String,
    #[input]
    amount: i64,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.amount <= 0 {
            return Self::fail("nothing-to-charge");
        }
        contributor.contribute("charged", self.amount);
        contributor.contribute("receipt", format!("R-{}", self.order));
        Self::succeed()
    }
}
// ANCHOR_END: step

// ANCHOR: workflow
/// The shop's workflow: it charges what its pricing works out.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
struct Orders;

#[workflow]
impl Orders {
    /// Works out the amount to charge: the price, less any discount. Every other input of the
    /// charge is read from the data bag.
    #[input_adapter(steps = ["charge"])]
    fn pricing(
        &self,
        #[key] key: &str,
        #[from_workflow] price: i64,
        #[from_workflow] discount: Option<i64>,
    ) -> Result<Option<AnyValue>, Error> {
        if key != "amount" {
            return Ok(None);
        }
        Ok(Some(AnyValue::new(price - discount.unwrap_or_default())))
    }
}
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    discount: Option<i64>,
}

/// What the shop reads back once the journey has ended: nothing was charged when it failed.
#[derive(Output)]
struct Receipt {
    charged: Option<i64>,
    receipt: Option<String>,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        ("A-1", 40, Some(5)),
        ("A-2", 40, None),
        ("A-3", 40, Some(40)),
    ];
    let mut executor = LocalExecutor::new();
    for (order, price, discount) in orders {
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price,
                discount,
            })
            .create()?;
        let result = executor.run(instance)?;
        let kind = result.status.kind();
        let receipt: Receipt = result.output()?;
        match (receipt.receipt, receipt.charged) {
            (Some(receipt), Some(charged)) => {
                println!("{order}: the journey {kind}, {receipt}: charged {charged}");
            }
            _ => println!("{order}: the journey {kind}"),
        }
    }
    Ok(())
}
// ANCHOR_END: run
