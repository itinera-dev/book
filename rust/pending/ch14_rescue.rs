//! Repairing, then trying once more: when the charge fails because the card has expired, its
//! rescue step asks the customer's bank for the new card, and the charge runs once more.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::step::{Outcome, Reason, Step, step};
use itinera::workflow::workflow;

/// The customer's bank: it knows which cards have expired, and which card replaced each.
struct Bank;

impl Bank {
    /// Whether a card can still be charged.
    fn accepts(card: &str) -> bool {
        card != "card-2019"
    }

    /// The card the bank issued to a customer after their old one expired, if any.
    fn renewed_card(customer: &str) -> Option<String> {
        (customer == "Ada").then(|| "card-2031".to_owned())
    }
}

// ANCHOR: charge
/// Charges an order's price to the customer's card.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
    #[input]
    card: String,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if !Bank::accepts(&self.card) {
            return Self::fail("expired-card");
        }
        contributor.contribute("charged", self.price);
        Self::succeed()
    }
}
// ANCHOR_END: charge

// ANCHOR: rescue
/// Asks the customer's bank for the card that replaced an expired one.
#[derive(Step)]
struct RenewCard {
    #[input]
    customer: String,
    #[reason]
    reason: Reason,
}

#[step]
impl RenewCard {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        if self.reason.code() != "expired-card" {
            return Self::fail("nothing-to-renew");
        }
        match Bank::renewed_card(&self.customer) {
            Some(card) => {
                contributor.contribute("card", card);
                Self::succeed()
            }
            None => Self::fail("no-renewed-card"),
        }
    }
}
// ANCHOR_END: rescue

/// Ships the order.
#[derive(Step)]
struct Ship;

#[step]
impl Ship {
    fn run(self) -> Result<Outcome, Error> {
        Self::succeed()
    }
}

// ANCHOR: workflow
/// The shop's workflow: a charge that fails is rescued once, then shipped.
#[workflow(name = "orders")]
#[with_step(Charge, name = "charge", rescue = RenewCard)]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    price: i64,
    card: String,
}

/// What the shop reads back once the journey has ended.
#[derive(Output)]
struct Charged {
    charged: Option<i64>,
    card: String,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orders = [
        NewOrder {
            order: "A-1".to_owned(),
            customer: "Ada".to_owned(),
            price: 40,
            card: "card-2019".to_owned(),
        },
        NewOrder {
            order: "A-2".to_owned(),
            customer: "Ben".to_owned(),
            price: 25,
            card: "card-2019".to_owned(),
        },
    ];
    let mut executor = LocalExecutor::new();
    for order in orders {
        println!("order {}", order.order);
        let instance = Orders::builder().initial_data(order).create()?;
        let result = executor.run(instance)?;
        let charged: Charged = result.output()?;
        println!("the journey {}", result.status.kind());
        println!("card {}", charged.card);
        match charged.charged {
            Some(amount) => println!("charged {amount}"),
            None => println!("nothing charged"),
        }
    }
    Ok(())
}
// ANCHOR_END: run
