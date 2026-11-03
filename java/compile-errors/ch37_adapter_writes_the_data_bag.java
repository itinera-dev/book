// An input adapter that asks for the data bag, and tries to write the weight into it.

package ch37_adapter_writes_the_data_bag;

import org.itineradev.itinera.executor.LocalExecutor;
import org.itineradev.itinera.executor.Refusal;
import org.itineradev.itinera.journey.DataBagAccess;
import org.itineradev.itinera.policy.FromWorkflow;
import org.itineradev.itinera.step.Input;
import org.itineradev.itinera.step.ItineraStep;
import org.itineradev.itinera.step.Outcome;
import org.itineradev.itinera.step.Step;
import org.itineradev.itinera.workflow.InputAdapter;
import org.itineradev.itinera.workflow.ItineraWorkflow;
import org.itineradev.itinera.workflow.Key;
import org.itineradev.itinera.workflow.Supplied;
import org.itineradev.itinera.workflow.WithStep;
import org.itineradev.itinera.workflow.Workflow;

/** Ships an order's parcel. */
@Step
final class Ship implements ItineraStep {
  private final long weight;

  Ship(@Input long weight) {
    this.weight = weight;
  }

  public Outcome run() {
    System.out.println("shipping " + weight + " kg");
    return succeed();
  }
}

/** How the warehouse describes an order's parcel: its weight in kg. */
record Parcel(long weight) {}

/** The shop's workflow. */
@Workflow(name = "orders")
@WithStep(step = Ship.class, name = "ship")
final class Orders implements ItineraWorkflow {
  /** Gives the shipping the parcel's weight, and tries to keep it in the data bag too. */
  @InputAdapter(steps = {"ship"})
  public Supplied parcels(@Key String key, @FromWorkflow Parcel parcel, DataBagAccess data) {
    data.write("weight", parcel.weight());
    return key.equals("weight") ? supply(parcel.weight()) : fromDataBag();
  }
}

/** What the shop knows about an order when its journey starts. */
record NewOrder(Parcel parcel) {}

final class Main {
  private Main() {}

  public static void main(String[] args) throws Refusal {
    var instance = new Orders().instance().initialData(new NewOrder(new Parcel(2))).create();
    System.out.println("the journey " + new LocalExecutor().run(instance).status());
  }
}
