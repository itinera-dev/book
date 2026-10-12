//! A refusal: the notify policy cannot be built, so `run` starts no journey for the order.

use itinera::error::Error;
use itinera::executor::LocalExecutor;
use itinera::journey::{Contributor, DataBag, InitialData, JourneyId};
use itinera::policy::workflow_policy;
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

/// The role the shop provides to policies: telling a customer something.
trait Messages {
    fn tell(&self, customer: &str, message: &str);
}

// ANCHOR: policy
/// The texts the shop sends its customers. Here the file that holds them is missing.
struct Templates {
    paid: String,
}

impl Templates {
    fn load() -> Result<Self, Error> {
        Err(Error::msg("templates.txt is missing"))
    }
}

/// Tells the customer how the order went, with the shop's own texts.
struct Notify {
    templates: Templates,
}

#[workflow_policy(name = "notify")]
impl Notify {
    #[factory]
    fn with_templates() -> Result<Self, Error> {
        Ok(Self {
            templates: Templates::load()?,
        })
    }

    #[on_workflow_success]
    fn paid(
        &self,
        #[journey_id] order: &JourneyId,
        #[from_workflow] customer: String,
        #[role] messages: &dyn Messages,
    ) -> Result<(), Error> {
        messages.tell(&customer, &format!("{order}: {}", self.templates.paid));
        Ok(())
    }
}
// ANCHOR_END: policy

/// The shop's workflow: it charges, then tells the customer.
#[workflow(name = "orders", policies = [Notify])]
#[with_step(Charge, name = "charge")]
struct Orders;

#[role]
impl Messages for Orders {
    fn tell(&self, customer: &str, message: &str) {
        println!("to {customer}: {message}");
    }
}

#[workflow]
impl Orders {
    #[id_generator]
    fn order_id(&self, data: &DataBag) -> Result<String, Error> {
        Ok(data.required("order")?)
    }
}

/// What the shop knows about an order when its journey starts.
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
        Ok(result) => println!("{}: {}", result.journey_id, result.status.kind()),
        Err(refusal) => println!("refused, nothing ran: {refusal}"),
    }
    Ok(())
}
// ANCHOR_END: run
