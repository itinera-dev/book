# Installing Itinera

## What you want to do

The shop is a program you already have. You want to add Itinera to it, with the options the shop needs, so that the next chapter can turn the charge into a step.

## The code

Itinera is one package. You add it to your project as you add any other library, and choose its options there.

Itinera is the crate `itinera`. Add it under `[dependencies]` in your `Cargo.toml` as `itinera = "0.1.0-rc.1"`, or run `cargo add itinera@0.1.0-rc.1`. It needs Rust 1.85 or later. Until `0.1.0-rc.1` is published, add it from its repository instead, as `itinera = { git = "https://github.com/itinera-dev/itinera-rs" }`; the book's own `Cargo.toml` does the same, pinned to one commit with `rev`.

Its features are:

- `macros`, on by default: the attributes and derives the book uses to declare steps, policies, workflows and journey data, such as `#[derive(Step)]` and `#[workflow]`. They expand to builder calls that you can write by hand, as chapter 28 shows.
- `async`, off by default: the asynchronous executor, and asynchronous steps, hooks and reporters. Turn it on with `itinera = { version = "0.1.0-rc.1", features = ["async"] }`, or `cargo add itinera@0.1.0-rc.1 --features async`. The asynchronous executor needs no particular runtime: it never spawns tasks, sleeps or sets timers, so you await it on whatever runtime the shop already uses. The book's examples use `futures::executor::block_on`.
- `unstable`, off by default: API the specification has not settled yet, which may change or go. It is empty for now.

`itinera` brings in `itinera-core`, which holds the implementation, and, with `macros`, `itinera-macros`. Do not add either yourself, and always write paths through `itinera`, such as `use itinera::step::Step;`.

## How it works

1. **It is a library, and nothing else.** Itinera runs inside your program. There is no server to start, no database to create and no background process to keep running. The executor that runs a journey, one end-to-end execution of a workflow for one order, lives in your process and keeps nothing once the journey is over.
2. **One package brings everything.** The package you add brings the parts it is built from. You never add them yourself, and you never need to know they are there.
3. **Some parts are options.** The short form for declaring steps, policies and workflows, which the book uses throughout, and the asynchronous executor can be options of the package, depending on the language; which are on by default, and how to turn them on, is said above. [Chapter 28](28-declaring-by-hand.md) shows what the short form stands for.
4. **Each part has an execution mode.** A step, a hook or a reporter is synchronous or asynchronous: that is its execution mode. Each executor accepts the modes it can run, and refuses anything else before the journey starts. You need the asynchronous executor only when the shop's steps call asynchronous code, such as a payment client that you wait for without blocking. The book shows each example's asynchronous form where it differs.
5. **The version follows the specification.** Each release of the package says which version and tier of the specification it implements. This edition of the book covers the release that implements tier 1, a single workflow, and [appendix 3](appendix-3-what-this-version-supports.md) says exactly what it supports.

## Remember

1. Add the one package and nothing else; it brings its own parts.
2. You need the asynchronous executor only if the shop has asynchronous steps, hooks or reporters; synchronous ones never need it.
3. Nothing outside your program is needed to run a journey, and nothing of a journey remains in the executor once it ends.
