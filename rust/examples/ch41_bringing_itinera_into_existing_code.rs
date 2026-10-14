//! Chapter 1's checkout with its payment moved into a workflow: the charge is a step, renewing an
//! expired card is its rescue step and the audit is a policy, while the bank's client, the audit
//! trail, the log and the rest of the checkout stay as they were.

use checkout::{Order, Shop};
use outside::{Carrier, Messages, Warehouse};

/// The outside world the shop talks to, faked so that every run is the same.
mod outside {
    /// Why the bank refused a charge.
    #[derive(Debug)]
    pub enum Declined {
        /// The card has expired; the bank may know the card that replaced it.
        ExpiredCard,
        /// The bank refused the charge.
        Refused,
    }

    /// The customers' bank. A card ends with the year it expires, such as `ann-card-2028`; the
    /// bank refuses a card that expired before this year, and any charge above its limit.
    pub struct Bank {
        pub this_year: u32,
        pub limit: i64,
    }

    impl Bank {
        /// Charges an amount to a card, and returns the receipt.
        pub fn charge(&self, card: &str, amount: i64) -> Result<String, Declined> {
            if expiry(card).is_some_and(|year| year < self.this_year) {
                return Err(Declined::ExpiredCard);
            }
            if amount > self.limit {
                return Err(Declined::Refused);
            }
            Ok(format!("R-{card}-{amount}"))
        }

        /// The card that replaced an expired one: the same card, expiring three years from now.
        pub fn renewed_card(&self, card: &str) -> Option<String> {
            let (holder, _) = card.rsplit_once('-')?;
            Some(format!("{holder}-{}", self.this_year + 3))
        }
    }

    /// The year a card expires, from the end of its number.
    fn expiry(card: &str) -> Option<u32> {
        card.rsplit_once('-')?.1.parse().ok()
    }

    /// The warehouse, which holds the stock.
    pub struct Warehouse {
        pub out_of_stock: Vec<&'static str>,
    }

    impl Warehouse {
        /// Sets the order's goods aside.
        pub fn reserve(&self, order: &str) -> Result<(), String> {
            if self.out_of_stock.contains(&order) {
                return Err(format!("nothing left for {order}"));
            }
            Ok(())
        }
    }

    /// A carrier, which takes parcels up to a weight.
    pub struct Carrier {
        pub name: &'static str,
        pub tracking_prefix: &'static str,
        pub max_weight: i64,
    }

    impl Carrier {
        /// Takes the order's parcel, and returns its tracking number.
        pub fn ship(&self, order: &str, weight: i64) -> Result<String, String> {
            if weight > self.max_weight {
                return Err(format!(
                    "{weight} kg is over its {} kg limit",
                    self.max_weight
                ));
            }
            Ok(format!("{}-{order}", self.tracking_prefix))
        }
    }

    /// Sends messages to customers.
    pub struct Messages;

    impl Messages {
        /// Tells a customer something.
        pub fn tell(&self, customer: &str, message: &str) {
            println!("to {customer}: {message}");
        }
    }
}

/// The shop's logging, used here and there.
mod log {
    pub fn info(message: &str) {
        println!("INFO {message}");
    }

    pub fn warn(message: &str) {
        println!("WARN {message}");
    }
}

/// The shop's audit trail.
mod audit {
    pub fn record(entry: &str) {
        println!("AUDIT {entry}");
    }
}

/// Taking the payment for an order, as a workflow.
mod payment {
    use itinera::error::Error;
    use itinera::event::{Event, EventBody};
    use itinera::journey::{Contributor, DataBag, InitialData, Output};
    use itinera::policy::{OnSuccess, step_policy};
    use itinera::report::Reporter;
    use itinera::service::{Provided, provide};
    use itinera::step::{Outcome, Reason, RescueOutcome, RescueStep, Step, rescue_step, step};
    use itinera::workflow::workflow;

    use crate::outside::{Bank, Declined};
    use crate::{audit, log};

    // ANCHOR: payments
    /// What the payment's steps ask of the bank.
    pub trait Payments: Send + Sync {
        fn charge(&self, card: &str, amount: i64) -> Result<String, Declined>;
        fn renewed_card(&self, card: &str) -> Option<String>;
    }

    /// The bank's client, as it was, behind the steps' trait.
    impl Payments for Bank {
        fn charge(&self, card: &str, amount: i64) -> Result<String, Declined> {
            Bank::charge(self, card, amount)
        }

        fn renewed_card(&self, card: &str) -> Option<String> {
            Bank::renewed_card(self, card)
        }
    }
    // ANCHOR_END: payments

    // ANCHOR: steps
    /// Charges the amount to the card, and gives back the receipt.
    #[derive(Step)]
    struct Charge {
        #[input]
        card: String,
        #[input]
        amount: i64,
    }

    #[step]
    impl Charge {
        fn run(
            self,
            #[service] payments: &dyn Payments,
            #[contributor] mut contributor: Contributor<'_>,
        ) -> Result<Outcome, Error> {
            match payments.charge(&self.card, self.amount) {
                Ok(receipt) => {
                    contributor.contribute("receipt", receipt);
                    Self::succeed()
                }
                Err(Declined::ExpiredCard) => Self::fail("card-expired"),
                Err(Declined::Refused) => Self::fail("declined"),
            }
        }
    }

    /// Repairs a charge whose card had expired: it asks the bank for the card that replaced it.
    #[derive(RescueStep)]
    struct RenewCard {
        #[input]
        card: String,
        #[reason]
        reason: Reason,
    }

    #[rescue_step]
    impl RenewCard {
        fn run(
            self,
            #[service] payments: &dyn Payments,
            #[contributor] mut contributor: Contributor<'_>,
        ) -> Result<RescueOutcome, Error> {
            if self.reason.code() != "card-expired" {
                return Self::fail("not-repairable");
            }
            let Some(card) = payments.renewed_card(&self.card) else {
                return Self::fail("not-renewed");
            };
            contributor.contribute("card", card);
            Self::succeed()
        }
    }
    // ANCHOR_END: steps

    // ANCHOR: audit
    /// Writes every successful charge to the shop's audit trail.
    #[derive(Default)]
    struct Audit;

    #[step_policy(name = "audit")]
    impl Audit {
        #[on_step_success]
        fn charged(
            &self,
            #[from_workflow] amount: i64,
            #[from_step] receipt: String,
        ) -> Result<Option<OnSuccess>, Error> {
            audit::record(&format!("charged {amount}, receipt {receipt}"));
            Self::carry_on()
        }
    }
    // ANCHOR_END: audit

    // ANCHOR: log
    /// Writes to the shop's log when a step fails, and when a rescue step repairs it.
    #[derive(Default)]
    struct Log;

    impl Reporter for Log {
        fn report(&mut self, event: &Event) -> Result<(), Error> {
            match &event.body {
                EventBody::StepFailed { step, reason, .. } => log::info(&format!(
                    "payment {}: {} failed, {}",
                    event.journey_id,
                    step.step,
                    reason.code()
                )),
                EventBody::StepRescued { step, rescue, .. } => log::info(&format!(
                    "payment {}: {} repaired by {rescue}",
                    event.journey_id, step.step
                )),
                _ => {}
            }
            Ok(())
        }
    }
    // ANCHOR_END: log

    // ANCHOR: workflow
    /// Charges an order, renewing a card that has expired, and audits the charge.
    #[workflow(name = "payment", reporters = [Log])]
    #[with_step(Charge, name = "charge", rescue = RenewCard, policies = [Audit])]
    pub struct Payment;

    #[workflow]
    impl Payment {
        /// Names each journey after its order.
        #[id_generator]
        fn order_number(&self, data: &DataBag) -> Result<String, Error> {
            Ok(data.required("order")?)
        }

        /// The bank, as it is configured today.
        #[provider(key = "payments")]
        fn bank(&self) -> Result<Provided<dyn Payments>, Error> {
            Ok(provide!(Bank {
                this_year: 2026,
                limit: 1000,
            }))
        }
    }

    /// What a payment starts with.
    #[derive(InitialData)]
    pub struct Due {
        pub order: String,
        pub card: String,
        pub amount: i64,
    }

    /// What a payment gives back.
    #[derive(Output)]
    pub struct Paid {
        pub receipt: String,
    }
    // ANCHOR_END: workflow
}

/// Shipping an order.
mod shipping {
    use crate::log;
    use crate::outside::Carrier;

    /// Ships through the main carrier, or through the backup carrier when the main one
    /// cannot take the parcel.
    pub fn ship(main: &Carrier, backup: &Carrier, order: &str, weight: i64) -> Option<String> {
        match main.ship(order, weight) {
            Ok(tracking) => Some(tracking),
            Err(error) => {
                log::warn(&format!(
                    "{} cannot take {order}: {error}; trying {}",
                    main.name, backup.name
                ));
                backup.ship(order, weight).ok()
            }
        }
    }
}

/// The checkout: what happens to an order once the customer has paid.
mod checkout {
    use std::fmt;

    use itinera::executor::local_executor_for;
    use itinera::journey::{Failure, JourneyStatus};

    use crate::outside::{Carrier, Declined, Messages, Warehouse};
    use crate::payment::{Due, Paid, Payment};
    use crate::{log, shipping};

    /// An order, as the shop's website hands it over.
    pub struct Order {
        pub order: &'static str,
        pub customer: &'static str,
        pub card: &'static str,
        pub price: i64,
        pub discount: Option<i64>,
        pub weight: i64,
    }

    /// Why a checkout did not complete.
    #[derive(Debug)]
    pub enum CheckoutError {
        OutOfStock,
        PaymentDeclined(Declined),
        NotShipped,
        PaymentNotRun(String),
    }

    impl fmt::Display for CheckoutError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::OutOfStock => write!(f, "out of stock"),
                Self::PaymentDeclined(Declined::ExpiredCard) => write!(f, "card expired"),
                Self::PaymentDeclined(Declined::Refused) => write!(f, "payment declined"),
                Self::NotShipped => write!(f, "no carrier could take it"),
                Self::PaymentNotRun(why) => write!(f, "the payment did not run: {why}"),
            }
        }
    }

    impl From<Declined> for CheckoutError {
        fn from(declined: Declined) -> Self {
            Self::PaymentDeclined(declined)
        }
    }

    /// The shop, with everything it talks to.
    pub struct Shop {
        pub warehouse: Warehouse,
        pub main: Carrier,
        pub backup: Carrier,
        pub messages: Messages,
    }

    impl Shop {
        /// Reserves, charges and ships an order, and tells the customer how it went.
        pub fn checkout(&self, order: &Order) -> Result<String, CheckoutError> {
            log::info(&format!("checkout {}", order.order));
            let shipped = self.reserve_charge_and_ship(order);
            match &shipped {
                Ok(tracking) => self.messages.tell(
                    order.customer,
                    &format!("order {} is on its way: {tracking}", order.order),
                ),
                Err(error) => {
                    log::warn(&format!("checkout {} failed: {error}", order.order));
                    self.messages.tell(
                        order.customer,
                        &format!("order {} could not be completed", order.order),
                    );
                }
            }
            shipped
        }

        fn reserve_charge_and_ship(&self, order: &Order) -> Result<String, CheckoutError> {
            self.warehouse
                .reserve(order.order)
                .map_err(|_| CheckoutError::OutOfStock)?;
            let amount = order.price - order.discount.unwrap_or(0);
            self.pay(order.order, order.card, amount)?;
            shipping::ship(&self.main, &self.backup, order.order, order.weight)
                .ok_or(CheckoutError::NotShipped)
        }

        // ANCHOR: pay
        /// Charges an order's card through the payment workflow, and gives back the receipt.
        fn pay(&self, order: &str, card: &str, amount: i64) -> Result<String, CheckoutError> {
            let instance = Payment::builder()
                .initial_data(Due {
                    order: order.to_owned(),
                    card: card.to_owned(),
                    amount,
                })
                .create()
                .map_err(not_run)?;
            let result = local_executor_for!(Payment)
                .run(instance)
                .map_err(not_run)?;
            match &result.status {
                JourneyStatus::Succeeded { .. } => {
                    let paid: Paid = result.output().map_err(not_run)?;
                    Ok(paid.receipt)
                }
                JourneyStatus::Failed {
                    failure: Failure::Failure(reason),
                    ..
                } if reason.code() == "card-expired" => Err(Declined::ExpiredCard.into()),
                JourneyStatus::Failed { .. } => Err(Declined::Refused.into()),
                _ => Err(CheckoutError::PaymentNotRun(
                    result.status.kind().to_string(),
                )),
            }
        }
        // ANCHOR_END: pay
    }

    /// Why the payment workflow could not say how the payment went.
    fn not_run(error: impl fmt::Display) -> CheckoutError {
        CheckoutError::PaymentNotRun(error.to_string())
    }
}

fn main() {
    let shop = Shop {
        warehouse: Warehouse {
            out_of_stock: vec![],
        },
        main: Carrier {
            name: "Rapid",
            tracking_prefix: "T",
            max_weight: 20,
        },
        backup: Carrier {
            name: "Steady",
            tracking_prefix: "B",
            max_weight: 50,
        },
        messages: Messages,
    };
    let orders = [
        Order {
            order: "A-1",
            customer: "Ann",
            card: "ann-card-2028",
            price: 40,
            discount: Some(5),
            weight: 2,
        },
        Order {
            order: "A-2",
            customer: "Ben",
            card: "ben-card-2024",
            price: 25,
            discount: None,
            weight: 1,
        },
        Order {
            order: "A-3",
            customer: "Cat",
            card: "cat-card-2027",
            price: 30,
            discount: None,
            weight: 30,
        },
        Order {
            order: "A-4",
            customer: "Dan",
            card: "dan-card-2025",
            price: 1200,
            discount: None,
            weight: 3,
        },
    ];
    for order in &orders {
        match shop.checkout(order) {
            Ok(tracking) => println!("{}: shipped, {tracking}", order.order),
            Err(error) => println!("{}: failed, {error}", order.order),
        }
        println!();
    }
}
