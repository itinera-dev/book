# Declaring by hand

## What you want to do

So far you have declared the shop in the short form, which writes most of the declaration for you. You want to see what that short form stands for, write the same workflow in the long form, and know when the long form is the one you need.

## The code

Here is the shop in the short form you have used so far: a charge and a ship step, the `audit` and `notify` policies, the `Messages` role, and the workflow that puts them together.

```example
name: ch28_short_form
region: step
```

```example
name: ch28_short_form
region: policies
```

```example
name: ch28_short_form
region: workflow
```

```example
name: ch28_short_form
region: data
```

It lists the workflow's steps, then runs one order:

```example
name: ch28_short_form
region: run
```

It prints:

```output
name: ch28_short_form
```

Now the same shop, by hand. A step is two things: the step itself, which runs once, and what builds it for each attempt. The builder says what the step needs, and builds the step from what the workflow resolved. A step that needs nothing can be a plain function.

```example
name: ch28_by_hand
region: step
asynchronous: ch28_by_hand_async
```

A policy defines each of its hooks, says what each hook requests, and reads what it requested when it is called. The role is an interface the workflow implements, and a hook asks for it by its type.

```example
name: ch28_by_hand
region: policies
asynchronous: ch28_by_hand_async
```

The workflow is declared part by part: each step under its name, with its retry budget and its policies, then the workflow's own policies. Building it checks the declaration, and gives you either the workflow or every violation found.

```example
name: ch28_by_hand
region: workflow
asynchronous: ch28_by_hand_async
```

Reading the output by hand means reading each key from the data bag.

```example
name: ch28_by_hand
region: data
```

Running it is the same, except that the initial data is given key by key.

```example
name: ch28_by_hand
region: run
asynchronous: ch28_by_hand_async
```

It prints exactly what the short form printed:

```output
name: ch28_by_hand
```

The asynchronous form changes the step, the hooks and the executor to their asynchronous kinds, and declares the workflow asynchronous; everything else is the same.

```note
name: ch28_by_hand
```

## How it works

1. **The short form writes descriptors.** Whatever form you write in, a workflow is a workflow descriptor: its name, a step descriptor for each step in order, its workflow policies, its input adapters and the roles it provides. The short form writes the descriptors from your types and their markings; the long form writes them yourself. The executor does not know or care which form made them, and the two programs above behave the same and print the same.
2. **A step descriptor** holds the step's name, how to build the step, its retry budget, whether an abnormal termination may be retried, and the step policies attached to it. How to build the step means saying what it needs, its inputs and its handles, and building it, for each attempt, from what was resolved. Nothing carries over from one attempt to the next, because each attempt gets a new step.
3. **A policy descriptor** holds the policy's name, the hooks it defines, what each hook requests, and how to build an instance of it. Workflow policies are built once per journey, step policies for every attempt.
4. **Building the workflow is admission.** In the long form you see it happen: building the declaration checks it, and returns every violation found, as [chapter 27](27-mistakes-caught-before-a-journey.md) shows. The short form runs the same checks, and in some languages runs them when the program is compiled.
5. **The listing reads the descriptor.** Both programs list the workflow's steps, in order, with their policies, without running anything. A workflow is fixed before any journey, whichever form declared it.
6. **When you need the long form:**
   - one kind of step, policy or workflow used with different types, which the short form may not express;
   - one policy attached to several steps and reading a different key on each, such as an audit that reads `charged` on the charge and `refunded` on a refund;
   - a declaration put together when the program starts, from the shop's configuration, for example adding a gift-wrapping step only in the shops that offer it. It is still fixed before any journey starts;
   - seeing every violation as data when the workflow is built, rather than as a compile error;
   - code that would rather not depend on the short form at all.

   Otherwise, the short form says the same thing in fewer lines, and is easier to read as the plan of the business.

## Remember

1. Both forms make the same workflow descriptor, and the executor runs either the same way.
2. A step is built afresh for every attempt, from what it declares it needs; keep nothing in what builds it.
3. Build the workflow once, before any journey, and handle its violations there; never change it while journeys run.
4. Use the long form only where the short form cannot say what you need.
