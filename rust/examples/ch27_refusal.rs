//! A journey refused by `run`: the notify policy cannot be built, so no journey starts.

use itinera::error::Error;
use itinera::event::Event;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, InitialData};
use itinera::policy::workflow_policy;
use itinera::report::Reporter;
use itinera::step::{Outcome, Step, step};
use itinera::workflow::{role, workflow};

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

/// The shop's way of telling a customer something.
trait Messages {
    fn tell(&self, customer: &str, message: &str);
}

// ANCHOR: policy
/// The shop's message templates, by occasion. The one for a completed order is missing.
const TEMPLATES: &[(&str, &str)] = &[("failed", "we could not complete your order")];

/// The template for an occasion, if the shop has one.
fn template(occasion: &str) -> Option<&'static str> {
    TEMPLATES
        .iter()
        .find(|(known, _)| *known == occasion)
        .map(|(_, text)| *text)
}

/// Tells the customer that the order is complete, with the shop's template.
struct Notify {
    completed: &'static str,
}

#[workflow_policy(name = "notify")]
impl Notify {
    #[factory]
    fn from_templates() -> Result<Self, Error> {
        let completed = template("completed")
            .ok_or_else(|| Error::msg("no message template for a completed order"))?;
        Ok(Self { completed })
    }

    #[on_workflow_success]
    fn completed(
        &self,
        #[from_workflow] customer: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, self.completed);
        Ok(())
    }
}
// ANCHOR_END: policy

/// Prints the kind of each event of the journey.
#[derive(Default)]
struct Printer;

impl Reporter for Printer {
    fn report(&mut self, event: &Event) -> Result<(), Error> {
        println!("event: {}", event.kind());
        Ok(())
    }
}

/// The shop's workflow: it charges, and tells the customer.
#[workflow(name = "orders", policies = [Notify], reporters = [Printer])]
#[with_step(Charge, name = "charge")]
struct Orders;

#[role]
impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        println!("to {customer}: {message}");
    }
}

/// What the shop knows about an order when the journey starts.
#[derive(InitialData)]
struct NewOrder {
    order: String,
    customer: String,
    price: i64,
}

// ANCHOR: run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = Orders::builder()
        .initial_data(NewOrder {
            order: "A-1".to_owned(),
            customer: "Ann".to_owned(),
            price: 40,
        })
        .create()?;
    match LocalExecutor::new().run(instance) {
        Ok(result) => println!("the journey {}", result.status.kind()),
        Err(refusal) => println!("refused: {refusal}"),
    }
    Ok(())
}
// ANCHOR_END: run
