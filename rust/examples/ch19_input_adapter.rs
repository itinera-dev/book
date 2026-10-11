//! Deciding where a step's data comes from: the workflow works out the amount to charge, the price
//! less any discount, and the charge never knows.

use itinera::error::Error;
use itinera::event::{Event, EventBody};
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::report::Reporter;
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

/// Prints each input an input adapter supplied.
#[derive(Default)]
struct Printer;

impl Reporter for Printer {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        if let EventBody::InputAdapterSupplied {
            step, key, adapter, ..
        } = &event.body
        {
            println!("  {} supplied {key} to {}", adapter, step.step);
        }
        Ok(())
    }
}

// ANCHOR: workflow
/// The shop's workflow: it charges what its pricing works out.
#[workflow(name = "orders", reporters = [Printer])]
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

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Receipt {
    charged: i64,
    receipt: String,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        NewOrder {
            order: "A-1".to_owned(),
            price: 40,
            discount: Some(5),
        },
        NewOrder {
            order: "A-2".to_owned(),
            price: 40,
            discount: None,
        },
    ];
    let mut executor = LocalExecutor::new();
    for order in orders {
        println!("order {}", order.order);
        let instance = Orders::builder().initial_data(order).create()?;
        let receipt: Receipt = executor.run(instance)?.output()?;
        println!("{}: charged {}", receipt.receipt, receipt.charged);
    }
    Ok(())
}
// ANCHOR_END: run
