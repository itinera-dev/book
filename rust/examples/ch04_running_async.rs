//! The small shop of chapter 4, run on the asynchronous executor: its steps are asynchronous.

use itinera::error::Error;
use itinera::executor::AsyncLocalExecutor;
use itinera::journey::{Contributor, DataBag, InitialData};
use itinera::report::DebugReporter;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// The customers' bank, faked: it refuses any charge above this limit.
const BANK_LIMIT: i64 = 1000;

/// Charges the card the price, less any discount.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
    #[input]
    discount: Option<i64>,
}

#[step]
impl Charge {
    async fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        let amount = self.price - self.discount.unwrap_or(0);
        if amount > BANK_LIMIT {
            return Self::fail("declined");
        }
        contributor.contribute("charged", amount);
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
    async fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        contributor.contribute("tracking", format!("T-{}", self.order));
        Self::succeed()
    }
}

// ANCHOR: workflow
/// The shop's workflow, declared asynchronous.
#[workflow(name = "orders", asynchronous, reporters = [DebugReporter])]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: workflow

// ANCHOR: data
/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    discount: Option<i64>,
}
// ANCHOR_END: data

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The order, the price and the discount.
    let orders = [("A-1", 40, Some(5)), ("A-2", 1200, None)];
    let mut executor = AsyncLocalExecutor::new();
    for (order, price, discount) in orders {
        println!("{order}:");
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price,
                discount,
            })
            .create()?;
        let result = futures::executor::block_on(executor.run(instance))?;
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
