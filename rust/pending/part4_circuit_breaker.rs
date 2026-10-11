//! A circuit breaker with Itinera: a policy records each carrier's failures through a role, and
//! the carrier's provider hands out the backup carrier while the breaker is open. The ship step
//! only ships.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData};
use itinera::policy::{FailWorkflow, OnSuccess, step_policy};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::{role, workflow};

/// What the shop asks of a carrier.
trait Carrier: Send + Sync {
    /// The carrier's name, as the shop knows it.
    fn name(&self) -> &'static str;

    /// Returns the tracking number, or nothing when the carrier is down.
    fn send(&self, order: &str) -> Option<String>;
}

/// A carrier the shop has a contract with. Today the main one is down.
#[derive(Clone)]
struct Parcels {
    name: &'static str,
    up: bool,
}

impl Carrier for Parcels {
    fn name(&self) -> &'static str {
        self.name
    }

    fn send(&self, order: &str) -> Option<String> {
        self.up.then(|| format!("{}-{order}", self.name))
    }
}

// ANCHOR: step
/// Ships an order through the carrier it is lent, and says which carrier that was.
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
        contributor.contribute("carrier", carrier.name().to_owned());
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
/// What the shop's breaker offers its policies: recording how each call to a carrier went.
trait CircuitBreaker {
    fn failed(&self, carrier: &str);
    fn succeeded(&self, carrier: &str);
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
        #[from_step] carrier: String,
        #[role] breaker: &dyn CircuitBreaker,
    ) -> Result<Option<OnSuccess>, Error> {
        breaker.succeeded(&carrier);
        Self::carry_on()
    }

    #[on_step_retry]
    fn retried(
        &self,
        #[from_step] carrier: String,
        #[role] breaker: &dyn CircuitBreaker,
    ) -> Result<Option<FailWorkflow>, Error> {
        breaker.failed(&carrier);
        Self::carry_on()
    }

    #[on_step_failure]
    fn gave_up(
        &self,
        #[from_step] carrier: Option<String>,
        #[role] breaker: &dyn CircuitBreaker,
    ) -> Result<Option<FailWorkflow>, Error> {
        if let Some(carrier) = carrier {
            breaker.failed(&carrier);
        }
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

#[workflow]
impl Orders {
    /// A fresh carrier for every attempt: the main one on attempt 1, unless the breaker is open.
    #[provider(per_attempt)]
    fn carrier(&self, #[attempt] attempt: NonZeroU32) -> Result<Box<dyn Carrier>, Error> {
        if attempt.get() == 1 && !self.breaker.is_open() {
            return Ok(Box::new(self.main.clone()));
        }
        Ok(Box::new(self.backup.clone()))
    }
}

#[role]
impl CircuitBreaker for Orders {
    fn failed(&self, carrier: &str) {
        if carrier == self.main.name {
            self.breaker.failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn succeeded(&self, carrier: &str) {
        if carrier == self.main.name {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let breaker = Arc::new(Breaker {
        failures: AtomicU32::new(0),
        threshold: 2,
    });
    let mut executor = LocalExecutor::new();
    for order in ["A-1", "A-3", "A-4"] {
        let instance = Orders::builder()
            .main(Parcels {
                name: "main",
                up: false,
            })
            .backup(Parcels {
                name: "backup",
                up: true,
            })
            .breaker(Arc::clone(&breaker))
            .initial_data(NewOrder {
                order: order.to_owned(),
            })
            .create()?;
        let result = executor.run(instance)?;
        println!("order {order}: the journey {}", result.status.kind());
    }
    Ok(())
}
