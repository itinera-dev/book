// The shop's checkout written the usual way, without Itinera: reserving, charging, renewing an
// expired card, auditing, shipping with a fallback carrier and telling the customer, with the
// decisions and the logging spread across modules.

// The outside world the shop talks to, faked so that every run is the same.

/** Why the bank refused a charge: the card has expired, or the bank refused it. */
type Why = "expired card" | "refused";

/** The bank refused a charge. */
class Declined extends Error {
  constructor(readonly why: Why) {
    super(why === "expired card" ? "card expired" : "payment declined");
  }
}

/**
 * The customers' bank. A card ends with the year it expires, such as `ann-card-2028`; the bank
 * refuses a card that expired before this year, and any charge above its limit.
 */
class Bank {
  constructor(
    private readonly thisYear: number,
    private readonly limit: number,
  ) {}

  /** Charges an amount to a card, and returns the receipt. */
  charge(card: string, amount: number): string {
    if (expiry(card) < this.thisYear) throw new Declined("expired card");
    if (amount > this.limit) throw new Declined("refused");
    return `R-${card}-${String(amount)}`;
  }

  /** The card that replaced an expired one: the same card, expiring three years from now. */
  renewedCard(card: string): string {
    return `${holder(card)}-${String(this.thisYear + 3)}`;
  }
}

/** The year a card expires, from the end of its number. */
function expiry(card: string): number {
  return Number(card.slice(card.lastIndexOf("-") + 1));
}

/** A card's number without the year it expires. */
function holder(card: string): string {
  return card.slice(0, card.lastIndexOf("-"));
}

/** The warehouse, which holds the stock. */
class Warehouse {
  constructor(private readonly outOfStock: readonly string[]) {}

  /** Sets the order's goods aside. */
  reserve(order: string): void {
    if (this.outOfStock.includes(order)) {
      throw new Error(`nothing left for ${order}`);
    }
  }
}

/** A carrier, which takes parcels up to a weight. */
class Carrier {
  constructor(
    readonly name: string,
    private readonly trackingPrefix: string,
    private readonly maxWeight: number,
  ) {}

  /** Takes the order's parcel, and returns its tracking number. */
  ship(order: string, weight: number): string {
    if (weight > this.maxWeight) {
      throw new Error(
        `${String(weight)} kg is over its ${String(this.maxWeight)} kg limit`,
      );
    }
    return `${this.trackingPrefix}-${order}`;
  }
}

/** Sends messages to customers. */
class Messages {
  /** Tells a customer something. */
  tell(customer: string, message: string): void {
    console.log(`to ${customer}: ${message}`);
  }
}

// The shop's logging, used here and there.

const log = {
  info(message: string): void {
    console.log(`INFO ${message}`);
  },
  warn(message: string): void {
    console.log(`WARN ${message}`);
  },
};

/** What was thrown, as a line of the log. */
function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

// The shop's audit trail.

const audit = {
  record(entry: string): void {
    console.log(`AUDIT ${entry}`);
  },
};

// Taking the payment for an order.

// ANCHOR: payment
/**
 * Charges the card. An expired card is renewed with the bank and charged once more. Every
 * successful charge is audited.
 */
function takePayment(bank: Bank, card: string, amount: number): string {
  let receipt: string;
  try {
    receipt = bank.charge(card, amount);
  } catch (error) {
    if (!(error instanceof Declined) || error.why !== "expired card") {
      throw error;
    }
    log.info(`card ${card} expired, renewing it`);
    receipt = bank.charge(bank.renewedCard(card), amount);
  }
  audit.record(`charged ${String(amount)}, receipt ${receipt}`);
  return receipt;
}
// ANCHOR_END: payment

// Shipping an order.

// ANCHOR: shipping
/**
 * Ships through the main carrier, or through the backup carrier when the main one cannot take
 * the parcel. Returns the tracking number, or `undefined` when neither can take it.
 */
function ship(
  main: Carrier,
  backup: Carrier,
  order: string,
  weight: number,
): string | undefined {
  try {
    return main.ship(order, weight);
  } catch (error) {
    log.warn(
      `${main.name} cannot take ${order}: ${messageOf(error)}; trying ${backup.name}`,
    );
    try {
      return backup.ship(order, weight);
    } catch {
      return undefined;
    }
  }
}
// ANCHOR_END: shipping

// The checkout: what happens to an order once the customer has paid.

/** An order, as the shop's website hands it over. */
interface Order {
  readonly order: string;
  readonly customer: string;
  readonly card: string;
  readonly price: number;
  readonly discount?: number;
  readonly weight: number;
}

/** The shop, with everything it talks to. */
class Shop {
  constructor(
    private readonly bank: Bank,
    private readonly warehouse: Warehouse,
    private readonly main: Carrier,
    private readonly backup: Carrier,
    private readonly messages: Messages,
  ) {}

  // ANCHOR: checkout
  /** Reserves, charges and ships an order, and tells the customer how it went. */
  checkout(order: Order): string {
    log.info(`checkout ${order.order}`);
    try {
      const tracking = this.reserveChargeAndShip(order);
      this.messages.tell(
        order.customer,
        `order ${order.order} is on its way: ${tracking}`,
      );
      return tracking;
    } catch (error) {
      log.warn(`checkout ${order.order} failed: ${messageOf(error)}`);
      this.messages.tell(
        order.customer,
        `order ${order.order} could not be completed`,
      );
      throw error;
    }
  }

  private reserveChargeAndShip(order: Order): string {
    this.warehouse.reserve(order.order);
    const amount = order.price - (order.discount ?? 0);
    takePayment(this.bank, order.card, amount);
    const tracking = ship(this.main, this.backup, order.order, order.weight);
    if (tracking === undefined) throw new Error("no carrier could take it");
    return tracking;
  }
  // ANCHOR_END: checkout
}

// ANCHOR: main
const shop = new Shop(
  new Bank(2026, 1000),
  new Warehouse([]),
  new Carrier("Rapid", "T", 20),
  new Carrier("Steady", "B", 50),
  new Messages(),
);
const orders: Order[] = [
  {
    order: "A-1",
    customer: "Ann",
    card: "ann-card-2028",
    price: 40,
    discount: 5,
    weight: 2,
  },
  {
    order: "A-2",
    customer: "Ben",
    card: "ben-card-2024",
    price: 25,
    weight: 1,
  },
  {
    order: "A-3",
    customer: "Cat",
    card: "cat-card-2027",
    price: 30,
    weight: 30,
  },
  {
    order: "A-4",
    customer: "Dan",
    card: "dan-card-2025",
    price: 1200,
    weight: 3,
  },
];
for (const order of orders) {
  try {
    console.log(`${order.order}: shipped, ${shop.checkout(order)}`);
  } catch (error) {
    console.log(`${order.order}: failed, ${messageOf(error)}`);
  }
  console.log();
}
// ANCHOR_END: main
