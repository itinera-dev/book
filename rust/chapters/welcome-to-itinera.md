# Welcome to Itinera

Hi, and welcome. We're glad you're here.

This book is about Itinera, a workflow framework that keeps the rules of your business apart from the decisions about what happens around them, and about `itinera`, the crate that brings it to Rust. We wrote it for people who build software for others, and for the AI agents who build it with them. You don't need to know anything about workflows to read it. You only need to know Rust: structs, traits, `Result` and the `?` operator, and a little about attributes and derives.

## What Itinera is

Think of the last piece of business code you read that charged a card or called a carrier. Somewhere in it was a decision that had nothing to do with the business: try again if the bank times out, fall back to the other carrier, log this, tell the customer that. Itinera gives each kind of decision a place of its own:

1. **A step** holds one business rule, such as "charge the order its price, less any discount". It never knows what comes before it or after it.
2. **A workflow** lists the steps in the order they run, how many times each may be tried, and what is attached to them. Read it, and you know what will run.
3. **A policy** holds the flow decisions: its hooks run after a step's outcome, or when the journey ends, and decide what happens next.
4. **The executor** runs the workflow, one journey for each order, and reports everything it does the same way for every workflow.

In Rust, steps, workflows and policies are ordinary code of yours, declared with a few attributes, and the executor comes with the crate.

Itinera is also more than this crate. Its behaviour is defined once, in a [specification](https://github.com/itinera-dev/spec) that names no programming language, and every language's implementation is checked against the same conformance suite. A workflow means the same thing in every language, and every language's book tells the same story: this one tells it in Rust.

The site explains why we think this matters, especially now that agents write so much of our code, on [Why Itinera](https://itinera-dev.org/why/). The book's first chapter shows it in code.

## Where the name comes from

*Itinera* is Latin. It is the plural of *iter*, which means a journey, a route, the way from one place to another. It is where the word "itinerary" comes from.

That is exactly what Itinera runs. One execution of a workflow for one order is a **journey**: it starts with what you know about the order, passes through the steps in order, and ends with what happened. One workflow, many journeys: *itinera*.

## How this book is laid out

The book follows one small online shop from start to finish. It charges orders, ships them, falls back to another carrier when the first one fails, audits every successful charge and tells customers how their order went. The shop grows a little in every chapter, and every chapter teaches one thing.

1. **Part 1, Why Itinera,** shows a checkout written the usual way, then the same checkout with Itinera, in one picture.
2. **The chapters after it** start small, with a single step, and grow only when you need more: putting steps together into a journey, giving them what they need from the outside world, reacting to how they went, watching, testing and designing it all.
3. **The appendices** hold a cheat sheet, a glossary and what this version supports.

You can read the book from the start, or jump to the chapter you need: every example is a complete Rust program that runs on its own.

Let's begin.
