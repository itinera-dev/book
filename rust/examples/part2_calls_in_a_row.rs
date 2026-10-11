//! Without Itinera: the shop charges an order, then ships it, by calling both itself.

use std::fmt;

/// Why a charge could not be made.
#[derive(Debug)]
enum ChargeError {
    NothingToCharge,
}

impl fmt::Display for ChargeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NothingToCharge => f.write_str("nothing to charge"),
        }
    }
}

impl std::error::Error for ChargeError {}

/// What the shop knows once an order is charged and shipped.
struct Shipped {
    charged: i64,
    tracking: String,
}

// ANCHOR: calls
/// Charges an order's price.
fn charge(price: i64) -> Result<i64, ChargeError> {
    if price <= 0 {
        return Err(ChargeError::NothingToCharge);
    }
    println!("the bank charges {price}");
    Ok(price)
}

/// Ships an order through the carrier, and returns its tracking number.
fn ship(order: &str) -> String {
    println!("the carrier collects order {order}");
    format!("T-{order}")
}

/// Charges an order, then ships it.
fn checkout(order: &str, price: i64) -> Result<Shipped, ChargeError> {
    let charged = charge(price)?;
    let tracking = ship(order);
    Ok(Shipped { charged, tracking })
}
// ANCHOR_END: calls

// ANCHOR: run
fn main() {
    for (order, price) in [("A-1", 40), ("A-2", 0)] {
        match checkout(order, price) {
            Ok(shipped) => println!(
                "{order}: charged {}, tracking {}",
                shipped.charged, shipped.tracking
            ),
            Err(error) => println!("{order}: not shipped: {error}"),
        }
    }
}
// ANCHOR_END: run
