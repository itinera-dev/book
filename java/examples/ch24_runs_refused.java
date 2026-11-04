// Runs an executor refuses: an instance handed to it a second time, and an instance handed to it
// while it runs another journey.

package ch24_runs_refused;

import java.util.concurrent.CountDownLatch;
import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.JourneyResult;
import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

/**
 * The shop's bank, faked: it prints what it charges. Once it is told to hold, it keeps the next
 * charge waiting until it is told to answer.
 */
final class Bank {
  private static volatile boolean holding = false;
  private static final CountDownLatch charging = new CountDownLatch(1);
  private static final CountDownLatch answered = new CountDownLatch(1);

  private Bank() {}

  static void charges(long amount) throws InterruptedException {
    if (holding) {
      charging.countDown();
      answered.await();
    }
    System.out.println("the bank charges " + amount);
  }

  /** Keeps the next charge waiting until {@link #answer()}. */
  static void hold() {
    holding = true;
  }

  /** Returns once the charge it holds has started. */
  static void awaitHeldCharge() throws InterruptedException {
    charging.await();
  }

  /** Lets the charge it holds go on. */
  static void answer() {
    holding = false;
    answered.countDown();
  }
}

/** Charges an order's price. */
@Step
final class Charge implements ItineraStep {
  private final long price;

  Charge(@Input long price) {
    this.price = price;
  }

  public Outcome run() throws InterruptedException {
    if (price <= 0) return fail("nothing-to-charge");
    Bank.charges(price);
    return succeed();
  }
}

/** The shop's workflow of chapter 5: it only charges. */
@Workflow(name = "orders")
@WithStep(step = Charge.class, name = "charge")
final class Orders implements ItineraWorkflow {}

// ANCHOR: run
/** Runs an instance, and gives back its journey's result, unless the run is refused. */
interface Run {
  JourneyResult run() throws Refusal;
}

final class Main {
  private Main() {}

  /** Says how the journey {@code run} runs ended, or why the run was refused. */
  static String said(Run run) {
    try {
      return run.run().isSucceeded() ? "the journey succeeded" : "the journey failed";
    } catch (Refusal refusal) {
      return "refused (" + refusal.getRefused() + "): " + refusal.getMessage();
    }
  }

  public static void main(String[] args) throws InterruptedException {
    var executor = new LocalExecutor();

    var ann = new Orders().instance().data("price", 40L).create();
    System.out.println("Ann's order: " + said(() -> executor.run(ann)));
    System.out.println("Ann's order again: " + said(() -> executor.run(ann)));

    Bank.hold();
    var ben = new Orders().instance().data("price", 25L).create();
    var bens =
        new Thread(() -> System.out.println("Ben's order: " + said(() -> executor.run(ben))));
    bens.start();
    Bank.awaitHeldCharge();
    var cat = new Orders().instance().data("price", 30L).create();
    System.out.println("Cat's order, while Ben's runs: " + said(() -> executor.run(cat)));
    Bank.answer();
    bens.join();
  }
}
// ANCHOR_END: run
