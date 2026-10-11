//! Giving a step the tools it needs: the workflow provides a stock store, a payment client and a
//! carrier, and the steps use them without knowing which ones they got.

use std::sync::Mutex;

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

// ANCHOR: services
/// Why the bank refused a charge.
enum Declined {
    ExpiredCard,
    Refused,
}

/// A payment client: it charges a card, and knows the card that replaced an expired one.
trait Payments: Send + Sync {
    fn charge(&self, card: &str, amount: i64) -> Result<String, Declined>;
    fn renewed_card(&self, card: &str) -> Option<String>;
}

/// A stock store: it reserves an order's goods, then hands them over for shipping.
trait StockStore: Send + Sync {
    fn reserve(&self, order: &str) -> bool;
    fn hand_over(&self, order: &str);
}

/// A carrier: it takes a parcel and gives back its tracking number, when it can take it.
trait Carrier: Send + Sync {
    fn send(&self, order: &str) -> Option<String>;
}
// ANCHOR_END: services

/// The warehouse's stock store, faked: it remembers what it has reserved.
#[derive(Default)]
struct Shelves {
    reserved: Mutex<Vec<String>>,
}

impl StockStore for Shelves {
    fn reserve(&self, order: &str) -> bool {
        let Ok(mut reserved) = self.reserved.lock() else {
            return false;
        };
        reserved.push(order.to_owned());
        println!("the stock store reserves order {order}");
        true
    }

    fn hand_over(&self, order: &str) {
        let found = self
            .reserved
            .lock()
            .is_ok_and(|reserved| reserved.contains(&order.to_owned()));
        if found {
            println!("the stock store hands over order {order}");
        } else {
            println!("the stock store has nothing reserved for order {order}");
        }
    }
}

/// The customers' bank, faked: it charges every card, and renews none.
struct Bank;

impl Payments for Bank {
    fn charge(&self, card: &str, amount: i64) -> Result<String, Declined> {
        println!("the bank charges {amount}");
        Ok(format!("R-{card}-{amount}"))
    }

    fn renewed_card(&self, _card: &str) -> Option<String> {
        None
    }
}

/// The shop's carrier, faked: it takes every parcel.
struct MainCarrier;

impl Carrier for MainCarrier {
    fn send(&self, order: &str) -> Option<String> {
        println!("the carrier collects order {order}");
        Some(format!("T-{order}"))
    }
}

// ANCHOR: steps
/// Reserves an order's items in the stock store.
#[derive(Step)]
struct Reserve {
    #[input]
    order: String,
}

#[step]
impl Reserve {
    fn run(self, #[service] stock: &dyn StockStore) -> Result<Outcome, Error> {
        if !stock.reserve(&self.order) {
            return Self::fail("out-of-stock");
        }
        Self::succeed()
    }
}

/// Charges an order's price to the customer's card, through the payment client.
#[derive(Step)]
struct Charge {
    #[input]
    card: String,
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
        match payments.charge(&self.card, self.price) {
            Ok(receipt) => {
                contributor.contribute("charged", self.price);
                contributor.contribute("receipt", receipt);
                Self::succeed()
            }
            Err(Declined::ExpiredCard) => Self::fail("expired-card"),
            Err(Declined::Refused) => Self::fail("declined"),
        }
    }
}

/// Ships an order through the carrier, and hands its goods over from the stock store.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
}

#[step]
impl Ship {
    fn run(
        self,
        #[service] stock: &dyn StockStore,
        #[service] carrier: &dyn Carrier,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        let Some(tracking) = carrier.send(&self.order) else {
            return Self::fail_retriably("carrier-unavailable");
        };
        stock.hand_over(&self.order);
        contributor.contribute("tracking", tracking);
        Self::succeed()
    }
}
// ANCHOR_END: steps

// ANCHOR: workflow
/// The shop's workflow: it reserves, charges and ships, with the services each step needs.
#[workflow(name = "orders")]
#[with_step(Reserve, name = "reserve")]
#[with_step(Charge, name = "charge")]
#[with_step(Ship, name = "ship")]
struct Orders;

#[workflow]
impl Orders {
    /// One stock store for the whole journey, shared by reserve and ship only.
    #[provider(steps = ["reserve", "ship"])]
    fn stock(&self) -> Result<Box<dyn StockStore>, Error> {
        println!("the warehouse opens a stock store");
        Ok(Box::new(Shelves::default()))
    }

    /// One payment client for the whole journey.
    #[provider]
    fn payments(&self) -> Result<Box<dyn Payments>, Error> {
        Ok(Box::new(Bank))
    }

    /// A fresh carrier for every attempt.
    #[provider(per_attempt)]
    fn carrier(&self) -> Result<Box<dyn Carrier>, Error> {
        println!("the shop books the carrier");
        Ok(Box::new(MainCarrier))
    }
}
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    card: String,
    price: i64,
}

/// What the shop reads back once the journey has succeeded.
#[derive(Output)]
struct Shipped {
    receipt: String,
    tracking: String,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = Orders::builder()
        .initial_data(NewOrder {
            order: "A-1".to_owned(),
            card: "ann-card".to_owned(),
            price: 40,
        })
        .create()?;
    let result = LocalExecutor::new().run(instance)?;
    println!("the journey {}", result.status.kind());
    let shipped: Shipped = result.output()?;
    println!("receipt {}, tracking {}", shipped.receipt, shipped.tracking);
    Ok(())
}
// ANCHOR_END: run
