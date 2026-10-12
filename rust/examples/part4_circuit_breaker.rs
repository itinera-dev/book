//! A circuit breaker with Itinera: a policy records each carrier's failures through a role, and
//! the carrier's provider hands out the backup carrier while the breaker is open. The ship step
//! only ships.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::policy::{FailWorkflow, OnSuccess, step_policy};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::{role, workflow};

/// What the shop asks of a carrier: take a parcel and give back its tracking number.
trait Carrier: Send + Sync {
    /// Returns the tracking number, or nothing when the carrier is down.
    fn send(&self, order: &str) -> Option<String>;
}

/// A carrier the shop has a contract with, faked: it says what it does. Today the main one is
/// down.
#[derive(Clone)]
struct Parcels {
    name: &'static str,
    prefix: &'static str,
    up: bool,
}

impl Carrier for Parcels {
    fn send(&self, order: &str) -> Option<String> {
        if !self.up {
            println!("  {} is down", self.name);
            return None;
        }
        println!("  {} takes {order}", self.name);
        Some(format!("{}-{order}", self.prefix))
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

// ANCHOR: role
/// What the shop's breaker offers its policies: recording how each attempt to ship went.
trait CircuitBreaker {
    fn failed(&self, attempt: NonZeroU32);
    fn succeeded(&self, attempt: NonZeroU32);
}
// ANCHOR_END: role

// ANCHOR: policy
/// Tells the breaker how every attempt to ship went.
#[derive(Default)]
struct Watch;

#[step_policy(name = "watch")]
impl Watch {
    #[on_step_success]
    fn shipped(
        &self,
        #[attempt] attempt: NonZeroU32,
        #[role] breaker: &dyn CircuitBreaker,
    ) -> Result<Option<OnSuccess>, Error> {
        breaker.succeeded(attempt);
        Self::carry_on()
    }

    #[on_step_retry]
    fn retried(
        &self,
        #[attempt] attempt: NonZeroU32,
        #[role] breaker: &dyn CircuitBreaker,
    ) -> Result<Option<FailWorkflow>, Error> {
        breaker.failed(attempt);
        Self::carry_on()
    }

    #[on_step_failure]
    fn gave_up(
        &self,
        #[attempt] attempt: NonZeroU32,
        #[role] breaker: &dyn CircuitBreaker,
    ) -> Result<Option<FailWorkflow>, Error> {
        breaker.failed(attempt);
        Self::carry_on()
    }
}
// ANCHOR_END: policy

/// The breaker's state, shared by every journey: the main carrier's failures in a row.
struct Breaker {
    failures: AtomicU32,
    threshold: u32,
}

impl Breaker {
    fn is_open(&self) -> bool {
        self.failures.load(Ordering::Relaxed) >= self.threshold
    }
}

// ANCHOR: workflow
/// The shop's workflow: shipping is tried twice, and the workflow decides the carrier.
#[workflow(name = "orders")]
#[with_step(Ship, name = "ship", retry_budget = 1, policies = [Watch])]
struct Orders {
    main: Parcels,
    backup: Parcels,
    breaker: Arc<Breaker>,
}

impl Orders {
    /// Whether an attempt to ship goes to the main carrier: only attempt 1, and only while the
    /// breaker is closed.
    fn main_carrier_for(&self, attempt: NonZeroU32) -> bool {
        attempt.get() == 1 && !self.breaker.is_open()
    }
}

#[workflow]
impl Orders {
    /// A fresh carrier for every attempt: the main one on attempt 1, unless the breaker is open.
    #[provider(per_attempt)]
    fn carrier(&self, #[attempt] attempt: NonZeroU32) -> Result<Box<dyn Carrier>, Error> {
        if self.main_carrier_for(attempt) {
            return Ok(Box::new(self.main.clone()));
        }
        Ok(Box::new(self.backup.clone()))
    }
}

/// The breaker counts only the main carrier's attempts, which it tells apart as the provider does.
#[role]
impl CircuitBreaker for Orders {
    fn failed(&self, attempt: NonZeroU32) {
        if self.main_carrier_for(attempt) {
            self.breaker.failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn succeeded(&self, attempt: NonZeroU32) {
        if self.main_carrier_for(attempt) {
            self.breaker.failures.store(0, Ordering::Relaxed);
        }
    }
}
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
}

/// What the shop reads back once the journey has ended.
#[derive(Output)]
struct Shipped {
    tracking: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let breaker = Arc::new(Breaker {
        failures: AtomicU32::new(0),
        threshold: 2,
    });
    let mut executor = LocalExecutor::new();
    for order in ["A-1", "A-3", "A-4"] {
        let instance = Orders::builder()
            .main(Parcels {
                name: "the main carrier",
                prefix: "T",
                up: false,
            })
            .backup(Parcels {
                name: "the backup carrier",
                prefix: "B",
                up: true,
            })
            .breaker(Arc::clone(&breaker))
            .initial_data(NewOrder {
                order: order.to_owned(),
            })
            .create()?;
        println!("order {order}:");
        let result = executor.run(instance)?;
        let shipped: Shipped = result.output()?;
        let status = result.status.kind();
        match shipped.tracking {
            Some(tracking) => println!("  the journey {status}, tracking {tracking}"),
            None => println!("  the journey {status}"),
        }
    }
    Ok(())
}
