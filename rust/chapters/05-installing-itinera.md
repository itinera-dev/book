# Installing Itinera

## What you want to do

The shop is a Rust program you already have. You want to add Itinera to it, with the features the shop needs, so that the next chapter can turn the charge into a step.

## The code

Itinera for Rust is one crate, `itinera`, and you add it as you add any other dependency, in your `Cargo.toml`. It needs Rust 1.85 or later.

The crate is not on crates.io yet, so for now you add it from its repository, under `[dependencies]`:

- in `Cargo.toml`, as `itinera = { git = "https://github.com/itinera-dev/itinera-rs" }`;
- or from the command line, with `cargo add itinera --git https://github.com/itinera-dev/itinera-rs`.

The book's own `Cargo.toml` does the same, pinned to one commit with `rev`, so that every example is built against exactly the code it was written for. Once its first release, `0.1.0-beta.1`, is published, you add it by its version instead, as `itinera = "0.1.0-beta.1"`, or with `cargo add itinera@0.1.0-beta.1`.

The crate has four Cargo features:

| Feature | On by default | What it brings |
|---|---|---|
| `macros` | yes | the attributes and derives the book uses to declare steps, policies, workflows and journey data, such as `#[derive(Step)]` and `#[workflow]` |
| `async` | no | the asynchronous executor, `AsyncLocalExecutor`, and asynchronous steps, hooks and reporters |
| `testing` | no | `RecordingReporter`, a reporter that keeps every event for a test to read, which [chapter 24](24-testing-your-workflows.md) uses; turn it on for tests only, under `[dev-dependencies]` |
| `unstable` | no | API the specification has not settled yet, which may change or go; it is empty for now |

To turn on `async`, add it to the dependency's features, as `itinera = { git = "https://github.com/itinera-dev/itinera-rs", features = ["async"] }`, or pass `--features async` to `cargo add`. Once the crate is published, the same line reads `itinera = { version = "0.1.0-beta.1", features = ["async"] }`.

## How it works

1. **It is a library, and nothing else.** Itinera runs inside your program. There is no server to start, no database to create and no background process to keep running. The executor that runs a journey, one end-to-end execution of a workflow for one order, lives in your process and keeps nothing once the journey is over.
2. **One crate brings everything.** `itinera` brings in `itinera-core`, which holds the implementation, and, with the `macros` feature, `itinera-macros`. Don't add either yourself, and always write paths through `itinera`, such as `use itinera::step::Step;`. You never need to know the other two are there.
3. **The macros are the short form.** With `macros`, which is on unless you turn default features off, you declare a step with `#[derive(Step)]` and `#[step]`, a workflow with `#[workflow]`, and so on, and the compiler checks each declaration. The macros expand to builder calls that you can write by hand, and [chapter 28](28-declaring-by-hand.md) shows what they stand for.
4. **Each part has an execution mode.** A step, a hook or a reporter is synchronous or asynchronous: that is its execution mode. In Rust, an asynchronous one is an `async fn`. `LocalExecutor` runs synchronous workflows, and `AsyncLocalExecutor`, which needs the `async` feature, runs asynchronous ones; handing either the other kind does not compile. You need the asynchronous executor only when the shop's steps call asynchronous code, such as a payment client that you `.await` without blocking. The book shows each example's asynchronous form where it differs.
5. **The asynchronous executor brings no runtime.** It never spawns tasks, sleeps or sets timers, so you await it on whatever runtime the shop already uses, Tokio or any other. The book's examples use `futures::executor::block_on`, from the `futures` crate.
6. **The version follows the specification.** Each release of `itinera` says which version and tier of the specification it implements. This book is written for `0.1.0-beta.1`, the first release, which implements tier 1, a single workflow. It's a beta: it's ready to learn and try, not yet meant for production, and later betas may still add or change features. Until it's published, the book's examples build against the commit of itinera-rs pinned in the book's `Cargo.toml`, and [appendix 3](appendix-3-what-this-version-supports.md) says exactly what that supports.

## Remember

1. Add `itinera` and nothing else; it brings its own parts, and every path goes through it.
2. Turn on the `async` feature only if the shop has asynchronous steps, hooks or reporters; synchronous ones never need it.
3. Nothing outside your program is needed to run a journey, and nothing of a journey remains in the executor once it ends.
