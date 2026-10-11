//! The shop of part 4 written without Itinera: retries, a fallback carrier, a circuit breaker and
//! a card repair, each written where the code needed it.

use std::cell::Cell;

/// What can go wrong when the shop talks to the outside world.
#[derive(Debug)]
enum Trouble {
    StockBusy,
    BankTimeout,
    ExpiredCard,
    CarrierDown,
    BreakerOpen,
}

impl Trouble {
    /// Whether trying again might help.
    fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::StockBusy | Self::BankTimeout | Self::CarrierDown
        )
    }
}

/// An order, as the shop receives it.
struct Order {
    id: &'static str,
    customer: &'static str,
    price: i64,
    card: &'static str,
    digital: bool,
}

/// The warehouse. It is busy the first time it is asked.
struct Warehouse {
    asked: Cell<u32>,
}

impl Warehouse {
    fn reserve(&self, _order: &Order) -> Result<(), Trouble> {
        let asked = self.asked.get();
        self.asked.set(asked + 1);
        if asked == 0 {
            return Err(Trouble::StockBusy);
        }
        Ok(())
    }
}

/// The customers' bank. It knows which cards have expired, and which card replaced each. It
/// times out the first time it is called.
struct Bank {
    called: Cell<bool>,
}

impl Bank {
    fn charge(&self, card: &str, amount: i64) -> Result<String, Trouble> {
        if !self.called.replace(true) {
            return Err(Trouble::BankTimeout);
        }
        if card == "card-2019" {
            return Err(Trouble::ExpiredCard);
        }
        Ok(format!("R-{amount}"))
    }

    fn renewed_card(&self, customer: &str) -> Option<&'static str> {
        (customer == "Ada").then_some("card-2031")
    }
}

/// A carrier the shop ships through.
struct Carrier {
    name: &'static str,
    up: bool,
}

impl Carrier {
    fn send(&self, order: &str) -> Result<String, Trouble> {
        if !self.up {
            return Err(Trouble::CarrierDown);
        }
        Ok(format!("{}-{order}", self.name))
    }
}

// ANCHOR: breaker
/// Stops calling a service after `threshold` failures in a row.
struct Breaker {
    failures: Cell<u32>,
    threshold: u32,
}

impl Breaker {
    fn call<T>(&self, call: impl FnOnce() -> Result<T, Trouble>) -> Result<T, Trouble> {
        if self.failures.get() >= self.threshold {
            return Err(Trouble::BreakerOpen);
        }
        let result = call();
        match result {
            Ok(_) => self.failures.set(0),
            Err(_) => self.failures.set(self.failures.get() + 1),
        }
        result
    }
}
// ANCHOR_END: breaker

// ANCHOR: decorator
/// Calls `call` up to `times` times while it fails in a way that might go away.
fn retrying<T>(times: u32, mut call: impl FnMut() -> Result<T, Trouble>) -> Result<T, Trouble> {
    let mut tried = 1;
    loop {
        match call() {
            Err(trouble) if trouble.is_transient() && tried < times => tried += 1,
            result => return result,
        }
    }
}
// ANCHOR_END: decorator

// ANCHOR: library
/// A retry policy, of the kind a retry library offers.
struct RetryPolicy {
    max_attempts: u32,
}

impl RetryPolicy {
    fn new() -> Self {
        Self { max_attempts: 3 }
    }

    fn max_attempts(self, max_attempts: u32) -> Self {
        Self { max_attempts }
    }

    fn run<T>(&self, call: impl FnMut() -> Result<T, Trouble>) -> Result<T, Trouble> {
        retrying(self.max_attempts, call)
    }
}
// ANCHOR_END: library

/// The shop, with everything it talks to.
struct Shop {
    warehouse: Warehouse,
    bank: Bank,
    main: Carrier,
    backup: Carrier,
    breaker: Breaker,
}

impl Shop {
    // ANCHOR: checkout
    /// Takes an order: reserves the stock, then charges it, which ships it.
    fn checkout(&self, order: &Order) -> Result<String, Trouble> {
        let mut tries = 0;
        loop {
            tries += 1;
            match self.warehouse.reserve(order) {
                Ok(()) => break,
                Err(Trouble::StockBusy) if tries < 3 => println!("  stock busy, trying again"),
                Err(trouble) => return Err(trouble),
            }
        }
        self.charge(order)
    }
    // ANCHOR_END: checkout

    // ANCHOR: charge
    /// Charges the order, renewing an expired card, then ships it unless it is digital.
    fn charge(&self, order: &Order) -> Result<String, Trouble> {
        let charged = retrying(2, || self.bank.charge(order.card, order.price));
        let receipt = match charged {
            Err(Trouble::ExpiredCard) => {
                let card = self
                    .bank
                    .renewed_card(order.customer)
                    .ok_or(Trouble::ExpiredCard)?;
                println!("  card expired, charging the renewed card");
                self.bank.charge(card, order.price)?
            }
            other => other?,
        };
        println!("  charged, receipt {receipt}");
        if order.digital {
            return Ok(format!("download D-{}", order.id));
        }
        self.ship(order)
    }
    // ANCHOR_END: charge

    // ANCHOR: ship
    /// Ships the order through the main carrier, or the backup one when the main one fails.
    fn ship(&self, order: &Order) -> Result<String, Trouble> {
        RetryPolicy::new().max_attempts(2).run(|| {
            match self.breaker.call(|| self.main.send(order.id)) {
                Ok(tracking) => Ok(tracking),
                Err(trouble) => {
                    println!("  main carrier: {trouble:?}, using the backup");
                    self.backup.send(order.id)
                }
            }
        })
    }
    // ANCHOR_END: ship
}

fn main() {
    let shop = Shop {
        warehouse: Warehouse {
            asked: Cell::new(0),
        },
        bank: Bank {
            called: Cell::new(false),
        },
        main: Carrier {
            name: "main",
            up: false,
        },
        backup: Carrier {
            name: "backup",
            up: true,
        },
        breaker: Breaker {
            failures: Cell::new(0),
            threshold: 2,
        },
    };
    let orders = [
        ("A-1", "Ada", 40, "card-2019", false),
        ("A-2", "Ben", 15, "card-2030", true),
        ("A-3", "Cleo", 25, "card-2028", false),
        ("A-4", "Dan", 30, "card-2029", false),
    ];
    for (id, customer, price, card, digital) in orders {
        println!("order {id}");
        let order = Order {
            id,
            customer,
            price,
            card,
            digital,
        };
        match shop.checkout(&order) {
            Ok(done) => println!("  done: {done}"),
            Err(trouble) => println!("  failed: {trouble:?}"),
        }
    }
}
