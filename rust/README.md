# The Rust book

The Rust book: its [chapters](chapters/README.md), and the examples and compile errors they show, laid out as [FORMAT.md](../FORMAT.md) describes. They depend on `itinera` at the commit or version pinned in [Cargo.toml](Cargo.toml).

## Running the examples

From this directory:

- `cargo build --examples` builds every example;
- `cargo run --example ch06_one_step` runs one, and what it prints must match `expected/ch06_one_step.txt`;
- `cargo test` runs every example and checks that it prints its recorded output, as CI does. An event's timestamp and a generated journey ID change from one run to the next, so they are compared as placeholders, and the recorded output keeps the values of the run that recorded it.
