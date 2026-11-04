// The shop's checkout written the usual way, without Itinera: reserving, charging, renewing an
// expired card, auditing, shipping with a fallback carrier and telling the customer, with the
// decisions and the logging spread across modules.

package ch01_checkout;

import java.util.List;
import java.util.Optional;

// The outside world the shop talks to, faked so that every run is the same.

/** Why the bank refused a charge. */
enum Why {
  /** The card has expired; the bank may know the card that replaced it. */
  EXPIRED_CARD("card expired"),
  /** The bank refused the charge. */
  REFUSED("payment declined");

  final String message;

  Why(String message) {
    this.message = message;
  }
}

/** The bank refused a charge. */
final class Declined extends Exception {
  private static final long serialVersionUID = 1L;

  final Why why;

  Declined(Why why) {
    super(why.message);
    this.why = why;
  }
}

/**
 * The customers' bank. A card ends with the year it expires, such as {@code ann-card-2028}; the
 * bank refuses a card that expired before this year, and any charge above its limit.
 */
final class Bank {
  private final int thisYear;
  private final long limit;

  Bank(int thisYear, long limit) {
    this.thisYear = thisYear;
    this.limit = limit;
  }

  /** Charges an amount to a card, and returns the receipt. */
  String charge(String card, long amount) throws Declined {
    if (Integer.parseInt(card.substring(card.lastIndexOf('-') + 1)) < thisYear) {
      throw new Declined(Why.EXPIRED_CARD);
    }
    if (amount > limit) throw new Declined(Why.REFUSED);
    return "R-" + card + "-" + amount;
  }

  /** The card that replaced an expired one: the same card, expiring three years from now. */
  String renewedCard(String card) {
    return card.substring(0, card.lastIndexOf('-')) + "-" + (thisYear + 3);
  }
}

/** The warehouse, which holds the stock. */
final class Warehouse {
  private final List<String> outOfStock;

  Warehouse(List<String> outOfStock) {
    this.outOfStock = outOfStock;
  }

  /** Sets the order's goods aside. */
  void reserve(String order) {
    if (outOfStock.contains(order)) throw new IllegalStateException("nothing left for " + order);
  }
}

/** A carrier, which takes parcels up to a weight. */
final class Carrier {
  final String name;
  private final String trackingPrefix;
  private final long maxWeight;

  Carrier(String name, String trackingPrefix, long maxWeight) {
    this.name = name;
    this.trackingPrefix = trackingPrefix;
    this.maxWeight = maxWeight;
  }

  /** Takes the order's parcel, and returns its tracking number. */
  String ship(String order, long weight) {
    if (weight > maxWeight) {
      throw new IllegalStateException(weight + " kg is over its " + maxWeight + " kg limit");
    }
    return trackingPrefix + "-" + order;
  }
}

/** Sends messages to customers. */
final class Messages {
  /** Tells a customer something. */
  void tell(String customer, String message) {
    System.out.println("to " + customer + ": " + message);
  }
}

// The shop's logging, used here and there.

final class Log {
  private Log() {}

  static void info(String message) {
    System.out.println("INFO " + message);
  }

  static void warn(String message) {
    System.out.println("WARN " + message);
  }
}

// The shop's audit trail.

final class Audit {
  private Audit() {}

  static void record(String entry) {
    System.out.println("AUDIT " + entry);
  }
}

// Taking the payment for an order.

final class Payments {
  private Payments() {}

  // ANCHOR: payment
  /**
   * Charges the card. An expired card is renewed with the bank and charged once more. Every
   * successful charge is audited.
   */
  static String takePayment(Bank bank, String card, long amount) throws Declined {
    String receipt;
    try {
      receipt = bank.charge(card, amount);
    } catch (Declined declined) {
      if (declined.why != Why.EXPIRED_CARD) throw declined;
      Log.info("card " + card + " expired, renewing it");
      receipt = bank.charge(bank.renewedCard(card), amount);
    }
    Audit.record("charged " + amount + ", receipt " + receipt);
    return receipt;
  }
  // ANCHOR_END: payment
}

// Shipping an order.

final class Shipping {
  private Shipping() {}

  // ANCHOR: shipping
  /**
   * Ships through the main carrier, or through the backup carrier when the main one cannot take the
   * parcel. Returns the tracking number, or nothing when neither can take it.
   */
  static Optional<String> ship(Carrier main, Carrier backup, String order, long weight) {
    try {
      return Optional.of(main.ship(order, weight));
    } catch (IllegalStateException refused) {
      Log.warn(
          main.name
              + " cannot take "
              + order
              + ": "
              + refused.getMessage()
              + "; trying "
              + backup.name);
      try {
        return Optional.of(backup.ship(order, weight));
      } catch (IllegalStateException alsoRefused) {
        return Optional.empty();
      }
    }
  }
  // ANCHOR_END: shipping
}

// The checkout: what happens to an order once the customer has paid.

/** An order, as the shop's website hands it over. */
record Order(
    String order, String customer, String card, long price, Optional<Long> discount, long weight) {}

/** The shop, with everything it talks to. */
final class Shop {
  private final Bank bank;
  private final Warehouse warehouse;
  private final Carrier main;
  private final Carrier backup;
  private final Messages messages;

  Shop(Bank bank, Warehouse warehouse, Carrier main, Carrier backup, Messages messages) {
    this.bank = bank;
    this.warehouse = warehouse;
    this.main = main;
    this.backup = backup;
    this.messages = messages;
  }

  // ANCHOR: checkout
  /** Reserves, charges and ships an order, and tells the customer how it went. */
  String checkout(Order order) throws Declined {
    Log.info("checkout " + order.order());
    try {
      String tracking = reserveChargeAndShip(order);
      messages.tell(order.customer(), "order " + order.order() + " is on its way: " + tracking);
      return tracking;
    } catch (Exception error) {
      Log.warn("checkout " + order.order() + " failed: " + error.getMessage());
      messages.tell(order.customer(), "order " + order.order() + " could not be completed");
      throw error;
    }
  }

  private String reserveChargeAndShip(Order order) throws Declined {
    warehouse.reserve(order.order());
    long amount = order.price() - order.discount().orElse(0L);
    Payments.takePayment(bank, order.card(), amount);
    return Shipping.ship(main, backup, order.order(), order.weight())
        .orElseThrow(() -> new IllegalStateException("no carrier could take it"));
  }
  // ANCHOR_END: checkout
}

final class Main {
  private Main() {}

  // ANCHOR: main
  public static void main(String[] args) {
    var shop =
        new Shop(
            new Bank(2026, 1000),
            new Warehouse(List.of()),
            new Carrier("Rapid", "T", 20),
            new Carrier("Steady", "B", 50),
            new Messages());
    var orders =
        List.of(
            new Order("A-1", "Ann", "ann-card-2028", 40, Optional.of(5L), 2),
            new Order("A-2", "Ben", "ben-card-2024", 25, Optional.empty(), 1),
            new Order("A-3", "Cat", "cat-card-2027", 30, Optional.empty(), 30),
            new Order("A-4", "Dan", "dan-card-2025", 1200, Optional.empty(), 3));
    for (var order : orders) {
      try {
        System.out.println(order.order() + ": shipped, " + shop.checkout(order));
      } catch (Exception error) {
        System.out.println(order.order() + ": failed, " + error.getMessage());
      }
      System.out.println();
    }
  }
  // ANCHOR_END: main
}
