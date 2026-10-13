//! A first step: the shop's charge, declared as a step. Nothing builds or runs it yet, so Rust
//! would warn that it is never used; the program turns that warning off.

#![allow(dead_code)]

use itinera::error::Error;
use itinera::step::{Outcome, Step, step};

/// The shop's bank, faked: it prints what it charges.
fn bank_charges(amount: i64) {
    println!("the bank charges {amount}");
}

// ANCHOR: step
/// Charges an order's price.
#[derive(Step)]
struct Charge {
    #[input]
    price: i64,
}

#[step]
impl Charge {
    fn run(self) -> Result<Outcome, Error> {
        if self.price <= 0 {
            return Self::fail("nothing-to-charge");
        }
        bank_charges(self.price);
        Self::succeed()
    }
}
// ANCHOR_END: step

fn main() {}
