//! Without Itinera: shipping that falls back to a second carrier, tested by mocking both carriers
//! and counting their calls.

use std::cell::Cell;

/// A carrier the shop ships through.
trait Carrier {
    fn ship(&self, order: &str) -> Result<String, String>;
}

// ANCHOR: ship
/// Ships through the main carrier, and through the backup carrier when the main one fails.
fn ship(order: &str, main: &dyn Carrier, backup: &dyn Carrier) -> Result<String, String> {
    match main.ship(order) {
        Ok(tracking) => Ok(tracking),
        Err(error) => {
            println!("main carrier failed for {order}: {error}, trying the backup");
            backup.ship(order)
        }
    }
}
// ANCHOR_END: ship

// ANCHOR: mock
/// A carrier that works or fails as the test says, and counts how often it is called.
struct MockCarrier {
    name: &'static str,
    works: bool,
    calls: Cell<u32>,
}

impl MockCarrier {
    fn new(name: &'static str, works: bool) -> Self {
        Self {
            name,
            works,
            calls: Cell::new(0),
        }
    }
}

impl Carrier for MockCarrier {
    fn ship(&self, order: &str) -> Result<String, String> {
        self.calls.set(self.calls.get() + 1);
        if self.works {
            return Ok(format!("{}-{order}", self.name));
        }
        Err("unavailable".to_owned())
    }
}
// ANCHOR_END: mock

/// Prints what was checked, or fails the program when it does not hold.
fn check(what: &str, holds: bool) -> Result<(), String> {
    if !holds {
        return Err(format!("failed: {what}"));
    }
    println!("ok: {what}");
    Ok(())
}

// ANCHOR: tests
fn main() -> Result<(), String> {
    let main = MockCarrier::new("main", true);
    let backup = MockCarrier::new("backup", true);
    let tracking = ship("A-1", &main, &backup)?;
    check(
        "a working main carrier ships the order",
        tracking == "main-A-1",
    )?;
    check("the backup is never called", backup.calls.get() == 0)?;

    let main = MockCarrier::new("main", false);
    let backup = MockCarrier::new("backup", true);
    let tracking = ship("A-2", &main, &backup)?;
    check(
        "a failing main carrier falls back",
        tracking == "backup-A-2",
    )?;
    check("the main carrier is called once", main.calls.get() == 1)?;
    check("the backup is called once", backup.calls.get() == 1)?;
    Ok(())
}
// ANCHOR_END: tests
