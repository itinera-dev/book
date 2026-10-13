//! Giving a step the tools it needs: the workflow provides the bank and the carrier, and the
//! steps use them without knowing which ones they got.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::service::{Provided, provide};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

// ANCHOR: services
/// A payment client: it charges an order's amount, and gives back a receipt.
trait Payments: Send + Sync {
    fn charge(&self, order: &str, amount: i64) -> String;
}

/// A carrier: it books a parcel and gives back its tracking number, then says whether it
/// collects a parcel of this weight in kg.
trait Carrier: Send + Sync {
    fn book(&self, order: &str) -> String;
    fn collects(&self, weight: i64) -> bool;
}
// ANCHOR_END: services

/// The shop's bank, faked: it prints what it charges, and gives a receipt for it.
struct Bank;

impl Payments for Bank {
    fn charge(&self, order: &str, amount: i64) -> String {
        println!("the bank charges {amount} for order {order}");
        format!("R-{order}")
    }
}

/// The shop's carrier, faked: it books any parcel, and collects one only up to its weight limit.
struct MainCarrier {
    max_weight: i64,
}

impl Carrier for MainCarrier {
    fn book(&self, order: &str) -> String {
        println!("the carrier books order {order}");
        format!("T-{order}")
    }

    fn collects(&self, weight: i64) -> bool {
        weight <= self.max_weight
    }
}

// ANCHOR: steps
/// Charges an order's price through the payment client, and gives back its receipt.
#[derive(Step)]
struct Charge {
    #[input]
    order: String,
    #[input]
    price: i64,
}

#[step]
impl Charge {
    fn run(
        self,
        #[service] payments: &dyn Payments,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        let receipt = payments.charge(&self.order, self.price);
        contributor.contribute("receipt", receipt);
        Self::succeed()
    }
}

/// Ships an order through the carrier, with its receipt in the parcel, and gives back its
/// tracking number.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    weight: i64,
    #[input]
    receipt: String,
}

#[step]
impl Ship {
    fn run(
        self,
        #[service] carrier: &dyn Carrier,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        let tracking = carrier.book(&self.order);
        contributor.contribute("tracking", tracking);
        if !carrier.collects(self.weight) {
            return Self::fail("too-heavy");
        }
        println!("the parcel leaves with receipt {}", self.receipt);
        Self::succeed()
    }
}
// ANCHOR_END: steps

// ANCHOR: workflow
/// The shop's workflow: it charges, then ships, with the services each step needs.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;

#[workflow]
impl Orders {
    /// The bank, as the payment client of the journey.
    #[provider(key = "payments")]
    fn bank(&self) -> Result<Provided<dyn Payments>, Error> {
        println!("the shop connects to the bank");
        Ok(provide!(Bank))
    }

    /// The main carrier, for the journey.
    #[provider]
    fn carrier(&self) -> Result<Provided<dyn Carrier>, Error> {
        println!("the shop calls the carrier");
        Ok(provide!(MainCarrier { max_weight: 20 }))
    }
}
// ANCHOR_END: workflow

/// What the shop knows about an order when its journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    weight: i64,
}

/// What the shop reads back once the journey has ended.
#[derive(Output)]
struct Order {
    receipt: String,
    tracking: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The order, the customer, the price and the parcel's weight in kg.
    for (order, customer, price, weight) in [("A-1", "Ann", 40, 2), ("A-7", "Gil", 45, 60)] {
        println!("{customer}'s order:");
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                price,
                weight,
            })
            .create()?;
        let result = LocalExecutor::new().run(instance)?;
        let data: Order = result.output()?;
        let tracking = data.tracking.as_deref().unwrap_or("none");
        println!(
            "the journey {}, with receipt {} and tracking {tracking}",
            result.status.kind(),
            data.receipt,
        );
    }
    Ok(())
}
