// An input adapter that asks for the data bag, and tries to write the weight into it.

package ch37_adapter_writes_the_data_bag

import org.itineradev.itinera.executor.LocalExecutor
import org.itineradev.itinera.journey.DataBagAccess
import org.itineradev.itinera.policy.FromWorkflow
import org.itineradev.itinera.step.Input
import org.itineradev.itinera.step.ItineraStep
import org.itineradev.itinera.step.Outcome
import org.itineradev.itinera.step.Step
import org.itineradev.itinera.workflow.InputAdapter
import org.itineradev.itinera.workflow.ItineraWorkflow
import org.itineradev.itinera.workflow.Key
import org.itineradev.itinera.workflow.Supplied
import org.itineradev.itinera.workflow.WithStep
import org.itineradev.itinera.workflow.Workflow

/** Ships an order's parcel. */
@Step
class Ship(@Input private val weight: Long) : ItineraStep {
    fun run(): Outcome {
        println("shipping $weight kg")
        return succeed()
    }
}

/** How the warehouse describes an order's parcel: its weight in kg. */
data class Parcel(val weight: Long)

/** The shop's workflow. */
@Workflow(name = "orders")
@WithStep(Ship::class, name = "ship")
class Orders : ItineraWorkflow {
    /** Gives the shipping the parcel's weight, and tries to keep it in the data bag too. */
    @InputAdapter(steps = ["ship"])
    fun parcels(@Key key: String, @FromWorkflow parcel: Parcel, data: DataBagAccess): Supplied {
        data.write("weight", parcel.weight)
        return if (key == "weight") supply(parcel.weight) else fromDataBag()
    }
}

/** What the shop knows about an order when its journey starts. */
data class NewOrder(val parcel: Parcel)

fun main() {
    val instance = Orders().instance().initialData(NewOrder(parcel = Parcel(weight = 2))).create()
    println("the journey ${LocalExecutor().run(instance).status}")
}
