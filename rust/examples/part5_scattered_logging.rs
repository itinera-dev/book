//! Without Itinera: a checkout whose three parts each log in their own way, written by three
//! people.

use std::fmt;

/// What the shop knows about an order.
struct Order {
    order: String,
    price: i64,
    customer: String,
}

/// Why a part of the checkout could not do its work.
#[derive(Debug)]
struct Refused(String);

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Refused {}

// ANCHOR: inventory
/// Written by the warehouse team: logs before every call.
mod inventory {
    use super::{Order, Refused};

    pub fn reserve(order: &Order) -> Result<(), Refused> {
        println!("[inventory] reserving stock for {}", order.order);
        Ok(())
    }
}
// ANCHOR_END: inventory

// ANCHOR: payments
/// Written by the payments team: logs only what goes wrong.
mod payments {
    use super::{Order, Refused};

    pub fn charge(order: &Order) -> Result<i64, Refused> {
        if order.price <= 0 {
            println!("WARN payments: nothing to charge on {}", order.order);
            return Err(Refused("nothing to charge".to_owned()));
        }
        Ok(order.price)
    }
}
// ANCHOR_END: payments

// ANCHOR: shipping
/// Written by whoever needed shipping first: a carrier that refuses is not worth failing the
/// order for, so its error is dropped, and nothing is logged.
mod shipping {
    use super::Order;

    pub fn ship(order: &Order) -> Option<String> {
        carrier(order).ok()
    }

    /// The carrier, faked: it does not deliver to Bo's address.
    fn carrier(order: &Order) -> Result<String, String> {
        if order.customer == "Bo" {
            return Err("no delivery to this address".to_owned());
        }
        Ok(format!("T-{}", order.order))
    }
}
// ANCHOR_END: shipping

// ANCHOR: checkout
fn checkout(order: &Order) -> Result<(), Refused> {
    inventory::reserve(order)?;
    let charged = payments::charge(order)?;
    let tracking = shipping::ship(order);
    println!(
        "order {} done: charged {charged}, tracking {}",
        order.order,
        tracking.as_deref().unwrap_or("none")
    );
    Ok(())
}
// ANCHOR_END: checkout

fn main() {
    let orders = [
        Order {
            order: "A-1".to_owned(),
            price: 40,
            customer: "Ana".to_owned(),
        },
        Order {
            order: "A-2".to_owned(),
            price: 25,
            customer: "Bo".to_owned(),
        },
        Order {
            order: "A-3".to_owned(),
            price: 0,
            customer: "Cy".to_owned(),
        },
    ];
    for order in &orders {
        if let Err(refused) = checkout(order) {
            println!("order {} not completed: {refused}", order.order);
        }
    }
}
