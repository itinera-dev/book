//! Giving a step what it needs: the charge requires a price and takes a discount when there is one.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{
    Abort, Contributor, InitialData, JourneyResult, JourneyStatus, MissingData, Output,
};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

// ANCHOR: step
/// Charges an order's price, less its discount when it has one.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
    #[input]
    discount: Option<i64>,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        let amount = self.price - self.discount.unwrap_or(0);
        if amount <= 0 {
            return Self::fail("nothing-to-charge");
        }
        contributor.contribute("charged", amount);
        Self::succeed()
    }
}
// ANCHOR_END: step

/// The shop's workflow: for now, it only charges.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
struct Orders;

// ANCHOR: data
/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    discount: Option<i64>,
}

/// An order whose price the shop forgot to give.
#[derive(InitialData)]
struct OrderWithoutPrice {
    order: String,
}
// ANCHOR_END: data

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Receipt {
    charged: i64,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut executor = LocalExecutor::new();

    let with_discount = Orders::builder()
        .initial_data(NewOrder {
            order: "A-1".to_owned(),
            price: 40,
            discount: Some(5),
        })
        .create()?;
    report("A-1", &executor.run(with_discount)?);

    let without_discount = Orders::builder()
        .initial_data(NewOrder {
            order: "A-2".to_owned(),
            price: 40,
            discount: None,
        })
        .create()?;
    report("A-2", &executor.run(without_discount)?);

    let without_price = Orders::builder()
        .initial_data(OrderWithoutPrice {
            order: "A-3".to_owned(),
        })
        .create()?;
    report("A-3", &executor.run(without_price)?);
    Ok(())
}
// ANCHOR_END: run

/// Prints how one order's journey ended.
fn report(order: &str, result: &JourneyResult) {
    if let JourneyStatus::Aborted(Abort::RequiredDataMissing(MissingData::Key { key, .. })) =
        &result.status
    {
        println!("{order}: aborted, required data missing: {key}");
    } else if let Ok(receipt) = result.output::<Receipt>() {
        println!(
            "{order}: {}, charged {}",
            result.status.kind(),
            receipt.charged
        );
    } else {
        println!("{order}: {}", result.status.kind());
    }
}
