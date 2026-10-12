//! The shop's checkout written the usual way, without Itinera: reserving, charging, renewing an
//! expired card, auditing, shipping with a fallback carrier and telling the customer, with the
//! decisions and the logging spread across modules.

use checkout::{Order, Shop};
use outside::{Bank, Carrier, Messages, Warehouse};

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

/// Taking the payment for an order.
mod payments {
    use crate::outside::{Bank, Declined};
    use crate::{audit, log};

    // ANCHOR: payment
    /// Charges the card. An expired card is renewed with the bank and charged once more.
    /// Every successful charge is audited.
    pub fn take_payment(bank: &Bank, card: &str, amount: i64) -> Result<String, Declined> {
        let receipt = match bank.charge(card, amount) {
            Err(Declined::ExpiredCard) => {
                log::info(&format!("card {card} expired, renewing it"));
                let renewed = bank.renewed_card(card).ok_or(Declined::ExpiredCard)?;
                bank.charge(&renewed, amount)?
            }
            charged => charged?,
        };
        audit::record(&format!("charged {amount}, receipt {receipt}"));
        Ok(receipt)
    }
    // ANCHOR_END: payment
}

/// Shipping an order.
mod shipping {
    use crate::log;
    use crate::outside::Carrier;

    // ANCHOR: shipping
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
    // ANCHOR_END: shipping
}

/// The checkout: what happens to an order once the customer has paid.
mod checkout {
    use std::fmt;

    use crate::outside::{Bank, Carrier, Declined, Messages, Warehouse};
    use crate::{log, payments, shipping};

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
    }

    impl fmt::Display for CheckoutError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::OutOfStock => write!(f, "out of stock"),
                Self::PaymentDeclined(Declined::ExpiredCard) => write!(f, "card expired"),
                Self::PaymentDeclined(Declined::Refused) => write!(f, "payment declined"),
                Self::NotShipped => write!(f, "no carrier could take it"),
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
        pub bank: Bank,
        pub warehouse: Warehouse,
        pub main: Carrier,
        pub backup: Carrier,
        pub messages: Messages,
    }

    impl Shop {
        // ANCHOR: checkout
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
            payments::take_payment(&self.bank, order.card, amount)?;
            shipping::ship(&self.main, &self.backup, order.order, order.weight)
                .ok_or(CheckoutError::NotShipped)
        }
        // ANCHOR_END: checkout
    }
}

// ANCHOR: main
fn main() {
    let shop = Shop {
        bank: Bank {
            this_year: 2026,
            limit: 1000,
        },
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
// ANCHOR_END: main
