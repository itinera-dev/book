# The Rust book

The Rust book: its [chapters](chapters/README.md), and the examples and compile errors they show, laid out as [FORMAT.md](../FORMAT.md) describes. They depend on `itinera` at the commit or version pinned in [Cargo.toml](Cargo.toml).

## Running the examples

From this directory:

- `cargo build --examples` builds every example;
- `cargo run --example ch05_first_journey` runs one, and what it prints must match `expected/ch05_first_journey.txt`;
- `cargo test` runs every example and checks that it prints its recorded output, as CI does. An event's timestamp and a generated journey ID change from one run to the next, so they are compared as placeholders, and the recorded output keeps the values of the run that recorded it.
- `cargo test` also builds every program in `compile-errors/` on its own, with the `compile-errors` feature, and checks that the compiler refuses it with the message recorded next to it. Each of those programs is listed in [Cargo.toml](Cargo.toml) as an example that only this feature builds. Compiler messages change between Rust releases, so [rust-toolchain.toml](rust-toolchain.toml) pins the release that recorded them; moving to a newer one means recording them again.
