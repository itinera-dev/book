// The shop's checkout written the usual way, without Itinera: reserving, charging, renewing an
// expired card, auditing, shipping with a fallback carrier and telling the customer, with the
// decisions and the logging spread across modules.

package ch01_checkout

// The outside world the shop talks to, faked so that every run is the same.

/** Why the bank refused a charge. */
enum class Why(val message: String) {
    /** The card has expired; the bank may know the card that replaced it. */
    EXPIRED_CARD("card expired"),
    /** The bank refused the charge. */
    REFUSED("payment declined"),
}

/** The bank refused a charge. */
class Declined(val why: Why) : Exception(why.message)

/**
 * The customers' bank. A card ends with the year it expires, such as `ann-card-2028`; the bank
 * refuses a card that expired before this year, and any charge above its limit.
 */
class Bank(private val thisYear: Int, private val limit: Long) {
    /** Charges an amount to a card, and returns the receipt. */
    fun charge(card: String, amount: Long): String {
        if (card.substringAfterLast('-').toInt() < thisYear) throw Declined(Why.EXPIRED_CARD)
        if (amount > limit) throw Declined(Why.REFUSED)
        return "R-$card-$amount"
    }

    /** The card that replaced an expired one: the same card, expiring three years from now. */
    fun renewedCard(card: String): String = "${card.substringBeforeLast('-')}-${thisYear + 3}"
}

/** The warehouse, which holds the stock. */
class Warehouse(private val outOfStock: List<String>) {
    /** Sets the order's goods aside. */
    fun reserve(order: String) {
        check(order !in outOfStock) { "nothing left for $order" }
    }
}

/** A carrier, which takes parcels up to a weight. */
class Carrier(val name: String, private val trackingPrefix: String, private val maxWeight: Long) {
    /** Takes the order's parcel, and returns its tracking number. */
    fun ship(order: String, weight: Long): String {
        check(weight <= maxWeight) { "$weight kg is over its $maxWeight kg limit" }
        return "$trackingPrefix-$order"
    }
}

/** Sends messages to customers. */
class Messages {
    /** Tells a customer something. */
    fun tell(customer: String, message: String) {
        println("to $customer: $message")
    }
}

// The shop's logging, used here and there.

object Log {
    fun info(message: String) {
        println("INFO $message")
    }

    fun warn(message: String) {
        println("WARN $message")
    }
}

// The shop's audit trail.

object Audit {
    fun record(entry: String) {
        println("AUDIT $entry")
    }
}

// Taking the payment for an order.

// ANCHOR: payment
/**
 * Charges the card. An expired card is renewed with the bank and charged once more. Every
 * successful charge is audited.
 */
fun takePayment(bank: Bank, card: String, amount: Long): String {
    val receipt =
        try {
            bank.charge(card, amount)
        } catch (declined: Declined) {
            if (declined.why != Why.EXPIRED_CARD) throw declined
            Log.info("card $card expired, renewing it")
            bank.charge(bank.renewedCard(card), amount)
        }
    Audit.record("charged $amount, receipt $receipt")
    return receipt
}

// ANCHOR_END: payment

// Shipping an order.

// ANCHOR: shipping
/**
 * Ships through the main carrier, or through the backup carrier when the main one cannot take the
 * parcel. Returns the tracking number, or `null` when neither can take it.
 */
fun ship(main: Carrier, backup: Carrier, order: String, weight: Long): String? =
    try {
        main.ship(order, weight)
    } catch (refused: IllegalStateException) {
        Log.warn("${main.name} cannot take $order: ${refused.message}; trying ${backup.name}")
        runCatching { backup.ship(order, weight) }.getOrNull()
    }

// ANCHOR_END: shipping

// The checkout: what happens to an order once the customer has paid.

/** An order, as the shop's website hands it over. */
data class Order(
    val order: String,
    val customer: String,
    val card: String,
    val price: Long,
    val discount: Long? = null,
    val weight: Long,
)

/** The shop, with everything it talks to. */
class Shop(
    private val bank: Bank,
    private val warehouse: Warehouse,
    private val main: Carrier,
    private val backup: Carrier,
    private val messages: Messages,
) {
    // ANCHOR: checkout
    /** Reserves, charges and ships an order, and tells the customer how it went. */
    fun checkout(order: Order): String {
        Log.info("checkout ${order.order}")
        try {
            val tracking = reserveChargeAndShip(order)
            messages.tell(order.customer, "order ${order.order} is on its way: $tracking")
            return tracking
        } catch (error: Exception) {
            Log.warn("checkout ${order.order} failed: ${error.message}")
            messages.tell(order.customer, "order ${order.order} could not be completed")
            throw error
        }
    }

    private fun reserveChargeAndShip(order: Order): String {
        warehouse.reserve(order.order)
        val amount = order.price - (order.discount ?: 0)
        takePayment(bank, order.card, amount)
        return ship(main, backup, order.order, order.weight) ?: error("no carrier could take it")
    }
    // ANCHOR_END: checkout
}

// ANCHOR: main
fun main() {
    val shop =
        Shop(
            bank = Bank(thisYear = 2026, limit = 1000),
            warehouse = Warehouse(outOfStock = emptyList()),
            main = Carrier(name = "Rapid", trackingPrefix = "T", maxWeight = 20),
            backup = Carrier(name = "Steady", trackingPrefix = "B", maxWeight = 50),
            messages = Messages(),
        )
    val orders =
        listOf(
            Order("A-1", "Ann", "ann-card-2028", price = 40, discount = 5, weight = 2),
            Order("A-2", "Ben", "ben-card-2024", price = 25, weight = 1),
            Order("A-3", "Cat", "cat-card-2027", price = 30, weight = 30),
            Order("A-4", "Dan", "dan-card-2025", price = 1200, weight = 3),
        )
    for (order in orders) {
        try {
            println("${order.order}: shipped, ${shop.checkout(order)}")
        } catch (error: Exception) {
            println("${order.order}: failed, ${error.message}")
        }
        println()
    }
}
// ANCHOR_END: main
