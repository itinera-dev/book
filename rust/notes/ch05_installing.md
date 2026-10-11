In Rust, Itinera is the crate `itinera`. Add it under `[dependencies]` in your `Cargo.toml` as `itinera = "0.1.0-rc.1"`, or run `cargo add itinera@0.1.0-rc.1`. It needs Rust 1.85 or later. Until `0.1.0-rc.1` is published, add it from its repository instead, as `itinera = { git = "https://github.com/itinera-dev/itinera-rs" }`; the book's own `Cargo.toml` does the same, pinned to one commit with `rev`.

Its features are:

- `macros`, on by default: the attributes and derives the book uses to declare steps, policies, workflows and journey data, such as `#[derive(Step)]` and `#[workflow]`. They expand to builder calls that you can write by hand, as chapter 28 shows.
- `async`, off by default: the asynchronous executor, and asynchronous steps, hooks and reporters. Turn it on with `itinera = { version = "0.1.0-rc.1", features = ["async"] }`, or `cargo add itinera@0.1.0-rc.1 --features async`. The asynchronous executor needs no particular runtime: it never spawns tasks, sleeps or sets timers, so you await it on whatever runtime the shop already uses. The book's examples use `futures::executor::block_on`.
- `unstable`, off by default: API the specification has not settled yet, which may change or go. It is empty for now.

`itinera` brings in `itinera-core`, which holds the implementation, and, with `macros`, `itinera-macros`. Do not add either yourself, and always write paths through `itinera`, such as `use itinera::step::Step;`.
