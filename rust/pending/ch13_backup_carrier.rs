//! Trying again with services: the carrier's provider gives the second attempt of shipping the
//! backup carrier, so the ship step never knows there are two.

use std::num::NonZeroU32;

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

// ANCHOR: carrier
/// What the shop asks of a carrier: take a parcel and give back its tracking number.
trait Carrier: Send + Sync {
    /// Returns the tracking number, or nothing when the carrier is down.
    fn send(&self, order: &str) -> Option<String>;
}
// ANCHOR_END: carrier

/// A carrier the shop has a contract with. Today the main one is down.
#[derive(Clone)]
struct Parcels {
    name: &'static str,
    up: bool,
}

impl Carrier for Parcels {
    fn send(&self, order: &str) -> Option<String> {
        self.up.then(|| format!("{}-{order}", self.name))
    }
}

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

// ANCHOR: step
/// Ships an order through the carrier it is lent.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
}

#[step]
impl Ship {
    fn run(
        self,
        #[service] carrier: &dyn Carrier,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        match carrier.send(&self.order) {
            Some(tracking) => {
                contributor.contribute("tracking", tracking);
                Self::succeed()
            }
            None => Self::fail_retriably("carrier-down"),
        }
    }
}
// ANCHOR_END: step

// ANCHOR: workflow
/// The shop's workflow: shipping is tried twice, and the workflow decides the carrier.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship", retry_budget = 1)]
struct Orders {
    main: Parcels,
    backup: Parcels,
}

#[workflow]
impl Orders {
    /// A fresh carrier for every attempt: the main one first, the backup one after.
    #[provider(per_attempt)]
    fn carrier(&self, #[attempt] attempt: NonZeroU32) -> Result<Box<dyn Carrier>, Error> {
        if attempt.get() == 1 {
            return Ok(Box::new(self.main.clone()));
        }
        Ok(Box::new(self.backup.clone()))
    }
}
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
}

/// What the shop reads back once the journey has ended.
#[derive(Output)]
struct Shipped {
    tracking: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = Orders::builder()
        .main(Parcels {
            name: "main",
            up: false,
        })
        .backup(Parcels {
            name: "backup",
            up: true,
        })
        .initial_data(NewOrder {
            order: "A-1".to_owned(),
            price: 40,
        })
        .create()?;
    let result = LocalExecutor::new().run(instance)?;
    let shipped: Shipped = result.output()?;
    println!("the journey {}", result.status.kind());
    match shipped.tracking {
        Some(tracking) => println!("tracking {tracking}"),
        None => println!("nothing shipped"),
    }
    Ok(())
}
