//! The fallback of Part 4 as a retry hook: shipping has a retry budget of 1, and the `on step
//! retry` hook of a step policy names the backup carrier for the next attempt.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::policy::{FailWorkflow, step_policy};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// A carrier the shop ships through, and the prefix of its tracking numbers. Today the main
/// carrier is down.
struct Carrier {
    prefix: &'static str,
    up: bool,
}

const MAIN: Carrier = Carrier {
    prefix: "T",
    up: false,
};

const BACKUP: Carrier = Carrier {
    prefix: "B",
    up: true,
};

impl Carrier {
    /// Hands the parcel over, and returns its tracking number, or nothing when the carrier is
    /// down.
    fn send(&self, order: &str) -> Option<String> {
        self.up.then(|| format!("{}-{order}", self.prefix))
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
/// Ships an order through the carrier it is given, or the main carrier when none is named.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    address: String,
    #[input]
    carrier: Option<String>,
}

#[step]
impl Ship {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.address.is_empty() {
            return Self::fail("no-address");
        }
        let carrier = match self.carrier.as_deref() {
            Some("backup") => BACKUP,
            _ => MAIN,
        };
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

// ANCHOR: policy
/// Names the backup carrier for the next attempt of the step it is attached to.
#[derive(Default)]
struct Fallback;

#[step_policy(name = "fallback")]
impl Fallback {
    #[on_step_retry]
    fn backup_carrier(
        &self,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Option<FailWorkflow>, Error> {
        contributor.contribute("carrier", "backup".to_owned());
        Self::carry_on()
    }
}
// ANCHOR_END: policy

// ANCHOR: workflow
/// The shop's workflow: shipping is tried twice, and the fallback policy picks the carrier.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship", retry_budget = 1, policies = [Fallback])]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    address: String,
}

/// What the shop reads back once the journey has ended.
#[derive(Output)]
struct Shipped {
    tracking: Option<String>,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        NewOrder {
            order: "A-1".to_owned(),
            price: 40,
            address: "1 High Street".to_owned(),
        },
        NewOrder {
            order: "A-2".to_owned(),
            price: 25,
            address: String::new(),
        },
    ];
    let mut executor = LocalExecutor::new();
    for order in orders {
        println!("order {}", order.order);
        let instance = Orders::builder().initial_data(order).create()?;
        let result = executor.run(instance)?;
        let shipped: Shipped = result.output()?;
        println!("the journey {}", result.status.kind());
        match shipped.tracking {
            Some(tracking) => println!("tracking {tracking}"),
            None => println!("nothing shipped"),
        }
    }
    Ok(())
}
// ANCHOR_END: run
