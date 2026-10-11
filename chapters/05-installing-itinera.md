# Installing Itinera

## What you want to do

The shop is a program you already have. You want to add Itinera to it, with the options the shop needs, so that the next chapter can turn the charge into a step.

## The code

Itinera is one package. You add it to your project as you add any other library, and choose its options there.

```note
name: ch05_installing
```

## How it works

1. **It is a library, and nothing else.** Itinera runs inside your program. There is no server to start, no database to create and no background process to keep running. The executor that runs a journey, one end-to-end run of a workflow for one order, lives in your process and keeps nothing once the journey is over.
2. **One package brings everything.** The package you add brings the parts it is built from. You never add them yourself, and you never need to know they are there.
3. **Some parts are options.** The short form for declaring steps, policies and workflows, which the book uses throughout, and the asynchronous executor can be options of the package, depending on the language; the note above says which are on by default and how to turn them on. Chapter 28 shows what the short form stands for.
4. **Each part has an execution mode.** A step, a policy or a reporter is synchronous or asynchronous: that is its execution mode. Each executor accepts the modes it can run, and refuses anything else before the journey starts. You need the asynchronous executor only when the shop's steps call asynchronous code, such as a payment client that you wait for without blocking. The book shows each example's asynchronous form where it differs.
5. **The version follows the specification.** Each release of the package says which version and tier of the specification it implements. This edition of the book covers the release that implements tier 1, a single workflow, and appendix 3 says exactly what it supports.

## Remember

1. Add the one package and nothing else; it brings its own parts.
2. You need the asynchronous executor only if the shop has asynchronous steps, policies or reporters; synchronous ones never need it.
3. Nothing outside your program is needed to run a journey, and nothing of a journey remains in the executor once it ends.
