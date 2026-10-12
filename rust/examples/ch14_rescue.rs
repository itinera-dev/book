//! Repairing, then trying once more: when the charge fails because the card has expired, its
//! rescue step asks the customer's bank for the new card, and the charge runs once more.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData, Output};
use itinera::report::DebugReporter;
use itinera::step::{Outcome, Reason, RescueOutcome, RescueStep, Step, rescue_step, step};
use itinera::workflow::workflow;

/// The customers' bank, faked. A card ends with the year it expires, such as `ben-card-2024`; the
/// bank refuses a card that expired before this year, and any charge above its limit.
struct Bank {
    this_year: u32,
    limit: i64,
}

const BANK: Bank = Bank {
    this_year: 2026,
    limit: 1000,
};

impl Bank {
    /// Charges an amount to a card, or says why it cannot.
    fn charge(&self, card: &str, amount: i64) -> Result<(), &'static str> {
        if expiry(card).is_some_and(|year| year < self.this_year) {
            return Err("expired-card");
        }
        if amount > self.limit {
            return Err("declined");
        }
        Ok(())
    }

    /// The card that replaced an expired one: the same card, expiring three years from now.
    fn renewed_card(&self, card: &str) -> Option<String> {
        let (holder, _) = card.rsplit_once('-')?;
        Some(format!("{holder}-{}", self.this_year + 3))
    }
}

/// The year a card expires, from the end of its number.
fn expiry(card: &str) -> Option<u32> {
    card.rsplit_once('-')?.1.parse().ok()
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
        if let Err(code) = BANK.charge(&self.card, self.price) {
            return Self::fail(code);
        }
        contributor.contribute("charged", self.price);
        Self::succeed()
    }
}
// ANCHOR_END: charge

// ANCHOR: rescue
/// Asks the customer's bank for the card that replaced an expired one. A charge that failed for
/// any other reason, or ended in an abnormal termination, which has no reason, leaves nothing to
/// renew, and so does a bank that has issued no new card.
#[derive(RescueStep)]
struct RenewCard {
    #[input]
    card: String,
    #[reason]
    reason: Option<Reason>,
}

#[rescue_step]
impl RenewCard {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<RescueOutcome, Error> {
        let renewed = match &self.reason {
            Some(reason) if reason.code() == "expired-card" => BANK.renewed_card(&self.card),
            _ => None,
        };
        let Some(card) = renewed else {
            return Self::fail("nothing-to-renew");
        };
        contributor.contribute("card", card);
        Self::succeed()
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
/// The shop's workflow: a charge that fails is rescued once, then the order is shipped.
#[workflow(name = "orders", reporters = [DebugReporter])]
#[with_step(Charge, name = "charge", rescue = RenewCard)]
#[with_step(Ship, name = "ship")]
struct Orders;
// ANCHOR_END: workflow

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
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
            order: "A-2".to_owned(),
            price: 40,
            card: "ben-card-2024".to_owned(),
        },
        NewOrder {
            order: "A-4".to_owned(),
            price: 1200,
            card: "dan-card-2025".to_owned(),
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
