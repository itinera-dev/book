// A first step: the shop's charge, declared as a step. Nothing builds or runs it yet, so the
// program prints nothing.

package ch04_first_step;

import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;

/** The shop's bank, faked: it prints what it charges. */
final class Bank {
  private Bank() {}

  static void charges(long amount) {
    System.out.println("the bank charges " + amount);
  }
}

// ANCHOR: step
/** Charges an order's price. */
@Step
final class Charge implements ItineraStep {
  private final long price;

  Charge(@Input long price) {
    this.price = price;
  }

  public Outcome run() {
    if (price <= 0) return fail("nothing-to-charge");
    Bank.charges(price);
    return succeed();
  }
}

// ANCHOR_END: step

final class Main {
  private Main() {}

  public static void main(String[] args) {}
}
