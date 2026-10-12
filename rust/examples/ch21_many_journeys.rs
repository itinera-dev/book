//! Running many journeys: one executor runs the day's orders, one new instance for each.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, DataBag, InitialData};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

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

// ANCHOR: workflow
/// The shop's workflow: charge, then ship. Each journey is named after its order.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;

#[workflow]
impl Orders {
    #[id_generator]
    fn order_id(&self, data: &DataBag) -> Result<String, Error> {
        Ok(data.required("order")?)
    }
}
// ANCHOR_END: workflow

// ANCHOR: data
/// What the shop knows about an order when its journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
}

/// The orders that came in today.
fn todays_orders() -> Vec<NewOrder> {
    vec![
        NewOrder {
            order: "A-1".to_owned(),
            price: 40,
        },
        NewOrder {
            order: "A-2".to_owned(),
            price: 0,
        },
        NewOrder {
            order: "A-3".to_owned(),
            price: 25,
        },
    ]
}
// ANCHOR_END: data

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut executor = LocalExecutor::new();
    for order in todays_orders() {
        let instance = Orders::builder().initial_data(order).create()?;
        let result = executor.run(instance)?;
        println!("{}: {}", result.journey_id, result.status.kind());
    }
    Ok(())
}
// ANCHOR_END: run
