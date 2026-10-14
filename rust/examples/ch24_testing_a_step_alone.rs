//! Testing a step alone: the shop's charge, rescue step and shipping, each run by a test outside
//! any journey, with services faked for the test.

use std::fmt;

use itinera::error::Error;
use itinera::journey::{Contributions, Contributor, Read};
use itinera::step::{
    Outcome, Reason, RescueOutcome, RescueStep, Step, StepEventReporter, StepReporter, rescue_step,
    step,
};

/// A payment client: it charges an order's amount to a card, and gives back a receipt, or nothing
/// when the card has expired, or the error of a network that failed; and it says which card
/// replaced one, if any did.
trait Payments: Send + Sync {
    fn charge(&self, order: &str, card: &str, amount: i64) -> Result<Option<String>, NetworkDown>;
    fn replacement(&self, card: &str) -> Option<String>;
}

/// A carrier: it books a parcel of this weight in kg and gives back its tracking number, or the
/// error of a booking system that failed, then says whether it collects the parcel.
trait Carrier: Send + Sync {
    fn book(&self, order: &str, weight: i64) -> Result<String, BookingFailed>;
    fn collects(&self, weight: i64) -> bool;
}

/// The error of the bank's network when it fails.
#[derive(Debug)]
struct NetworkDown;

impl fmt::Display for NetworkDown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the payment network is down")
    }
}

impl std::error::Error for NetworkDown {}

/// The error of the carrier's booking system when it fails.
#[derive(Debug)]
struct BookingFailed;

impl fmt::Display for BookingFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the booking system failed")
    }
}

impl std::error::Error for BookingFailed {}

/// Charges an order's amount to its card through the payment client, and gives back its receipt,
/// and the gift card's code when the order is a gift card.
#[derive(Step)]
struct Charge {
    #[input]
    order: String,
    #[input]
    amount: i64,
    #[input]
    card: String,
    #[input]
    gift_card: bool,
}

#[step]
impl Charge {
    fn run(
        self,
        #[service] payments: &dyn Payments,
        #[contributor] mut contributor: Contributor<'_>,
    ) -> Result<Outcome, Error> {
        if self.amount <= 0 {
            return Self::fail("nothing-to-charge");
        }
        let Some(receipt) = payments.charge(&self.order, &self.card, self.amount)? else {
            return Self::fail("card-expired");
        };
        contributor.contribute("receipt", receipt);
        if self.gift_card {
            contributor.contribute("code", format!("G-{}", self.order));
        }
        Self::succeed()
    }
}

/// Repairs a charge that failed because its card had expired: it asks the bank for the card that
/// replaced it, and gives it back as the order's card.
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
        let Some(card) = payments.replacement(&self.card) else {
            return Self::fail("not-replaced");
        };
        contributor.contribute("card", card);
        Self::succeed()
    }
}

/// Ships an order through the carrier, with its receipt in the parcel, gives back its tracking
/// number, and says when the parcel has left. A parcel the carrier won't collect may be taken on
/// another attempt. A gift card has nothing to ship, and the shop ships no parcel over 80 kg.
#[derive(Step)]
struct Ship {
    #[input]
    order: String,
    #[input]
    weight: i64,
    #[input]
    receipt: String,
    #[input]
    gift_card: bool,
}

#[step]
impl Ship {
    fn run(
        self,
        #[service] carrier: &dyn Carrier,
        #[contributor] mut contributor: Contributor<'_>,
        #[reporter] mut reporter: StepReporter<'_>,
    ) -> Result<Outcome, Error> {
        if self.gift_card {
            return Self::skip_because("nothing-to-ship");
        }
        if self.weight > 80 {
            return Self::fail("over-parcel-limit");
        }
        let tracking = carrier.book(&self.order, self.weight)?;
        contributor.contribute("tracking", tracking);
        if !carrier.collects(self.weight) {
            return Self::fail("too-heavy");
        }
        reporter.info(format!("the parcel leaves with receipt {}", self.receipt))?;
        Self::succeed()
    }
}

// ANCHOR: fakes
/// A bank for tests: it answers every charge as the test chose, and has replaced every card
/// with one named after its holder that expires in 2029.
enum TestBank {
    Charges,
    Declines,
    IsDown,
}

impl Payments for TestBank {
    fn charge(&self, order: &str, _: &str, _: i64) -> Result<Option<String>, NetworkDown> {
        match self {
            TestBank::Charges => Ok(Some(format!("R-{order}"))),
            TestBank::Declines => Ok(None),
            TestBank::IsDown => Err(NetworkDown),
        }
    }

    fn replacement(&self, card: &str) -> Option<String> {
        let (holder, _) = card.rsplit_once('-')?;
        Some(format!("{holder}-2029"))
    }
}

/// A carrier for tests: it books and collects every parcel, or its booking system is down.
enum TestCarrier {
    Works,
    IsDown,
}

impl Carrier for TestCarrier {
    fn book(&self, order: &str, _: i64) -> Result<String, BookingFailed> {
        match self {
            TestCarrier::Works => Ok(format!("T-{order}")),
            TestCarrier::IsDown => Err(BookingFailed),
        }
    }

    fn collects(&self, _: i64) -> bool {
        true
    }
}
// ANCHOR_END: fakes

/// Prints what was checked, or fails the program when it does not hold.
fn check(what: &str, holds: bool) -> Result<(), Error> {
    if !holds {
        return Err(Error::msg(format!("failed: {what}")));
    }
    println!("ok: {what}");
    Ok(())
}

// ANCHOR: charge
/// Ann's charge of 40 to her card.
fn anns_charge() -> Charge {
    Charge {
        order: "A-1".to_owned(),
        amount: 40,
        card: "ann-2028".to_owned(),
        gift_card: false,
    }
}

fn the_charge() -> Result<(), Error> {
    let mut contributions = Contributions::default();
    let outcome = anns_charge().run(&TestBank::Charges, contributions.contributor())?;
    check("a charge the bank takes succeeds", outcome.is_success())?;
    check(
        "it gives back the bank's receipt",
        contributions.read::<String>("receipt") == Read::Present("R-A-1".to_owned()),
    )?;

    let mut contributions = Contributions::default();
    let outcome = anns_charge().run(&TestBank::Declines, contributions.contributor())?;
    check(
        "a charge the bank declines fails with card-expired",
        outcome.reason().map(Reason::code) == Some("card-expired"),
    )?;
    check(
        "it gives back no receipt",
        contributions.read::<String>("receipt") == Read::Absent,
    )?;

    let mut contributions = Contributions::default();
    let ended = anns_charge().run(&TestBank::IsDown, contributions.contributor());
    check(
        "a charge lets the error of a bank that is down escape",
        ended.is_err(),
    )?;
    Ok(())
}
// ANCHOR_END: charge

// ANCHOR: rescue
fn the_rescue_step() -> Result<(), Error> {
    let mut contributions = Contributions::default();
    let renew = RenewCard {
        card: "jo-2024".to_owned(),
        reason: Reason::from("card-expired"),
    };
    let outcome = renew.run(&TestBank::Charges, contributions.contributor())?;
    check("renewing an expired card succeeds", outcome.is_success())?;
    check(
        "it gives back the card that replaced it",
        contributions.read::<String>("card") == Read::Present("jo-2029".to_owned()),
    )?;

    let mut contributions = Contributions::default();
    let renew = RenewCard {
        card: "max-2027".to_owned(),
        reason: Reason::from("nothing-to-charge"),
    };
    let outcome = renew.run(&TestBank::Charges, contributions.contributor())?;
    check(
        "a failure that isn't an expired card is not repairable",
        outcome.reason().map(Reason::code) == Some("not-repairable"),
    )?;
    Ok(())
}
// ANCHOR_END: rescue

// ANCHOR: ship
/// An order's shipping, for a parcel of this weight.
fn shipping(weight: i64) -> Ship {
    Ship {
        order: "A-1".to_owned(),
        weight,
        receipt: "R-A-1".to_owned(),
        gift_card: false,
    }
}

fn the_shipping() -> Result<(), Error> {
    let mut contributions = Contributions::default();
    let mut events = StepEventReporter::default();
    let outcome = shipping(2).run(
        &TestCarrier::Works,
        contributions.contributor(),
        events.reporter(),
    )?;
    check("a parcel of 2 kg ships", outcome.is_success())?;
    check(
        "the shipping says the parcel left with its receipt",
        events.emitted().first().map(|event| event.message())
            == Some("the parcel leaves with receipt R-A-1"),
    )?;

    let mut contributions = Contributions::default();
    let mut events = StepEventReporter::default();
    let outcome = shipping(90).run(
        &TestCarrier::IsDown,
        contributions.contributor(),
        events.reporter(),
    )?;
    check(
        "a parcel of 90 kg fails with over-parcel-limit, before the carrier is called",
        outcome.reason().map(Reason::code) == Some("over-parcel-limit"),
    )?;
    Ok(())
}
// ANCHOR_END: ship

fn main() -> Result<(), Error> {
    the_charge()?;
    the_rescue_step()?;
    the_shipping()?;
    Ok(())
}
