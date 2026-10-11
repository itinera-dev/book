//! A counter-example: a charge step that repairs an expired card and tells the customer itself.

use itinera::error::Error;
use itinera::event::Event;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData};
use itinera::report::Reporter;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::workflow;

/// The customer's bank, faked: it accepts every card but the expired one.
fn bank_accepts(card: &str) -> bool {
    card != "card-expired"
}

/// The customer's bank, faked: the card that replaces an expired one.
fn renewed_card(card: &str) -> String {
    card.replace("expired", "renewed")
}

// ANCHOR: step
/// Charges an order, and does a great deal more than charge.
#[derive(Step)]
struct Charge {
    #[input]
    order: String,
    #[input]
    price: i64,
    #[input]
    card: String,
}

#[step]
impl Charge {
    fn run(self, #[contributor] mut contributor: Contributor<'_>) -> Result<Outcome, Error> {
        let mut card = self.card;
        if !bank_accepts(&card) {
            card = renewed_card(&card);
            contributor.contribute("card", card.clone());
        }
        if !bank_accepts(&card) {
            println!("to the customer of {}: your card was refused", self.order);
            return Self::fail("declined");
        }
        contributor.contribute("charged", self.price);
        println!("to the customer of {}: you paid {}", self.order, self.price);
        Self::succeed()
    }
}
// ANCHOR_END: step

/// Prints the kind of each event of the journey.
#[derive(Default)]
struct Printer;

impl Reporter for Printer {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        println!("event: {}", event.kind());
        Ok(())
    }
}

/// The shop's workflow: it only charges.
#[workflow(name = "orders", reporters = [Printer])]
#[with_step(Charge, name = "charge")]
struct Orders;

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    price: i64,
    card: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = Orders::builder()
        .initial_data(NewOrder {
            order: "A-3".to_owned(),
            price: 40,
            card: "card-expired".to_owned(),
        })
        .create()?;
    let result = LocalExecutor::new().run(instance)?;
    println!("A-3: {}", result.status.kind());
    Ok(())
}
