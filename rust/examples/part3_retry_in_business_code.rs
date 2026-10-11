//! Without Itinera: the charge calls the bank, decides on its own to retry a timeout, and then
//! ships the order itself.

use std::fmt;

/// What can go wrong when the bank is asked to charge.
#[derive(Debug)]
enum BankError {
    Timeout,
    Refused,
}

impl fmt::Display for BankError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => f.write_str("the bank timed out"),
            Self::Refused => f.write_str("the bank refused the charge"),
        }
    }
}

impl std::error::Error for BankError {}

/// The shop's bank, faked: it times out the first time it is called, and refuses to charge
/// nothing.
struct Bank {
    calls: u32,
}

impl Bank {
    fn charge(&mut self, price: i64) -> Result<(), BankError> {
        self.calls += 1;
        if self.calls == 1 {
            return Err(BankError::Timeout);
        }
        if price <= 0 {
            return Err(BankError::Refused);
        }
        println!("the bank charges {price}");
        Ok(())
    }
}

/// Ships an order through the carrier, and returns its tracking number.
fn ship(order: &str) -> String {
    println!("the carrier collects order {order}");
    format!("T-{order}")
}

// ANCHOR: charge
/// How many times the charge is tried before giving up.
const CHARGE_ATTEMPTS: u32 = 3;

/// Charges an order, trying again when the bank times out, then ships it.
fn charge(bank: &mut Bank, order: &str, price: i64) -> Result<String, BankError> {
    let mut attempt = 1;
    loop {
        match bank.charge(price) {
            Ok(()) => break,
            Err(BankError::Timeout) if attempt < CHARGE_ATTEMPTS => {
                println!("attempt {attempt} timed out, trying again");
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(ship(order))
}
// ANCHOR_END: charge

fn main() {
    let mut bank = Bank { calls: 0 };
    for (order, price) in [("A-1", 40), ("A-2", 0)] {
        match charge(&mut bank, order, price) {
            Ok(tracking) => println!("{order}: tracking {tracking}"),
            Err(error) => println!("{order}: not shipped: {error}"),
        }
    }
}
