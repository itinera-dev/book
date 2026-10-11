//! Naming your journeys: the workflow makes each journey's ID from its order.

use itinera::error::Error;
use itinera::event::Event;
use itinera::executor::LocalExecutor;
use itinera::instance::WorkflowInstance;
use itinera::journey::{Contributor, DataBag, InitialData};
use itinera::report::Reporter;
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

/// Prints the journey ID and kind of every event.
#[derive(Default)]
struct Printer;

impl Reporter for Printer {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        println!("  {} {}", event.journey_id, event.kind());
        Ok(())
    }
}

// ANCHOR: workflow
/// The shop's workflow: each journey is named after its order.
#[workflow(name = "orders", reporters = [Printer])]
#[with_step(Charge, name = "charge")]
struct Orders;

#[workflow]
impl Orders {
    /// Makes the journey ID from the order in the initial data.
    #[id_generator]
    fn order_id(&self, data: &DataBag) -> Result<String, Error> {
        let order: String = data.required("order")?;
        Ok(format!("order-{order}"))
    }
}
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
}

/// An order that arrived without its ID.
#[derive(InitialData)]
struct PriceOnly {
    price: i64,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        NewOrder {
            order: "A-1".to_owned(),
            price: 40,
        },
        NewOrder {
            order: "A-2".to_owned(),
            price: 0,
        },
    ];
    let mut executor = LocalExecutor::new();
    for order in orders {
        let instance = Orders::builder().initial_data(order).create()?;
        println!("created {}", instance.journey_id());
        let result = executor.run(instance)?;
        println!("{} {}", result.journey_id, result.status.kind());
    }
    let unnamed = Orders::builder()
        .initial_data(PriceOnly { price: 40 })
        .create();
    if let Err(error) = unnamed {
        println!("not created: {error}");
    }
    Ok(())
}
// ANCHOR_END: run
