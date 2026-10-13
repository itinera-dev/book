//! The fallback of Part 4 as a retry hook: shipping has a retry budget of 1, the `on step retry`
//! hook of a step policy names the backup carrier for the next attempt, and the carrier's provider
//! reads that choice from the workflow's data.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{
    Contributor, Failure, InitialData, JourneyResult, JourneyStatus, LastFailure,
};
use itinera::policy::{FailWorkflow, step_policy};
use itinera::service::{Provided, provide};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// What the shop asks of a carrier: take a parcel and give back its tracking number.
trait Carrier: Send + Sync {
    /// Returns the tracking number, or nothing when the carrier cannot take the parcel.
    fn send(&self, order: &str, weight: i64) -> Option<String>;
}

/// A carrier the shop has a contract with, faked: it takes parcels up to a weight, and says what
/// it did.
struct Van {
    name: &'static str,
    prefix: &'static str,
    max_weight: i64,
}

const MAIN_CARRIER: Van = Van {
    name: "the main carrier",
    prefix: "T",
    max_weight: 20,
};

const BACKUP_CARRIER: Van = Van {
    name: "the backup carrier",
    prefix: "B",
    max_weight: 50,
};

impl Carrier for Van {
    fn send(&self, order: &str, weight: i64) -> Option<String> {
        if weight > self.max_weight {
            println!("  {} cannot take {weight} kg", self.name);
            return None;
        }
        println!("  {} takes {order}", self.name);
        Some(format!("{}-{order}", self.prefix))
    }
}

/// Ships an order to its address, through the carrier it is lent.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    address: Option<String>,
    #[input]
    weight: i64,
}

#[step]
impl Ship {
    fn run(
        self,
        #[service] carrier: &dyn Carrier,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        if self.address.is_none() {
            return Self::fail("no-address");
        }
        match carrier.send(&self.order, self.weight) {
            Some(tracking) => {
                contributor.contribute("tracking", tracking);
                Self::succeed()
            }
            None => Self::fail_retriably("parcel-refused"),
        }
    }
}

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
/// The shop's workflow: shipping may be tried twice, and the fallback policy picks the carrier.
#[workflow(name = "orders")]
#[with_step(Ship, name = "ship", retry_budget = 1, policies = [Fallback])]
struct Orders;

#[workflow]
impl Orders {
    /// A fresh carrier for every attempt: the one the data names, or the main one.
    #[provider(per_attempt)]
    fn carrier(
        &self,
        #[from_workflow] carrier: Option<String>,
    ) -> Result<Provided<dyn Carrier>, Error> {
        if carrier.as_deref() == Some("backup") {
            return Ok(provide!(BACKUP_CARRIER));
        }
        Ok(provide!(MAIN_CARRIER))
    }
}
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    address: Option<String>,
    weight: i64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The order, its address, and its parcel's weight in kg.
    let orders = [
        ("A-1", Some("1 High Street"), 2),
        ("A-3", Some("3 Mill Lane"), 30),
    ];
    let mut executor = LocalExecutor::new();
    for (order, address, weight) in orders {
        println!("{order}:");
        let instance = Orders::builder()
            .initial_data(NewOrder {
                order: order.to_owned(),
                address: address.map(str::to_owned),
                weight,
            })
            .create()?;
        report(&executor.run(instance)?);
    }
    Ok(())
}

/// Prints how a journey ended, and why when it failed.
fn report(result: &JourneyResult) {
    match &result.status {
        JourneyStatus::Failed {
            failure:
                failure @ (Failure::Failure(reason)
                | Failure::RetriesExhausted(LastFailure::Reason(reason))),
            ..
        } => println!("failed: {}, {}", failure.cause(), reason.code()),
        status => println!("{}", status.kind()),
    }
}
