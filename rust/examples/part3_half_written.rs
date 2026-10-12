//! Without Itinera: the order is one record that each function fills in, and a failure leaves it
//! half written.

use std::fmt;

/// Why an order could not be shipped.
#[derive(Debug)]
enum ShipError {
    NoAddress,
}

impl fmt::Display for ShipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAddress => f.write_str("the carrier has no address"),
        }
    }
}

impl std::error::Error for ShipError {}

/// The carrier, faked: it delivers only to customers whose address it knows.
fn carrier_delivers_to(customer: &str) -> bool {
    customer == "Ann"
}

// ANCHOR: record
/// Everything the shop knows about an order, filled in as the checkout goes.
#[derive(Debug)]
struct Order {
    order: String,
    customer: String,
    price: i64,
    charged: Option<i64>,
    tracking: Option<String>,
}

/// Charges the order, and records what was charged.
fn charge(order: &mut Order) {
    order.charged = Some(order.price);
}

/// Books the carrier, records the tracking number, then hands the parcel over.
fn ship(order: &mut Order) -> Result<(), ShipError> {
    order.tracking = Some(format!("T-{}", order.order));
    if !carrier_delivers_to(&order.customer) {
        return Err(ShipError::NoAddress);
    }
    Ok(())
}
// ANCHOR_END: record

// ANCHOR: run
fn main() {
    let mut order = Order {
        order: "A-2".to_owned(),
        customer: "Ben".to_owned(),
        price: 40,
        charged: None,
        tracking: None,
    };
    charge(&mut order);
    if let Err(error) = ship(&mut order) {
        println!("not shipped: {error}");
    }
    println!("{order:?}");
}
// ANCHOR_END: run
