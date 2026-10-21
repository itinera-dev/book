// An input adapter that tries to write into the data bag, which it may only
// read.

import { LocalExecutor } from "itinera/executor";
import { type DataBag, dataBag } from "itinera/journey";
import { input, type Outcome, Step, step } from "itinera/step";
import { integer } from "itinera/value";
import {
  inputAdapter,
  key,
  type Supplied,
  withStep,
  Workflow,
  workflow,
} from "itinera/workflow";

/** Ships an order, which has to weigh something. */
@step()
class Ship extends Step {
  constructor(@input("weight", integer) private readonly weight: number) {
    super();
  }

  override run(): Outcome {
    if (this.weight <= 0) return this.fail("weighs-nothing");
    return this.succeed();
  }
}

/** The shop's workflow, whose adapter writes the weight it supplies. */
@workflow({ name: "orders" })
@withStep(Ship, { name: "ship" })
class Orders extends Workflow {
  @inputAdapter({ steps: ["ship"] })
  parcels(@key() key: string, @dataBag() data: DataBag): Supplied {
    if (key !== "weight") return this.fromDataBag();
    data.write("weight", 1);
    return this.supply(1);
  }
}

const result = await new LocalExecutor().run(Orders.build().instance());
console.log(`the journey ${result.status.kind}`);
