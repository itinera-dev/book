# The book's Rust examples

The examples, notes and compile errors of the book's Rust edition, laid out as [FORMAT.md](../FORMAT.md) describes. They depend on `itinera` at the commit or version pinned in [Cargo.toml](Cargo.toml).

## Running them

From this directory:

- `cargo build --examples` builds every example;
- `cargo run --example ch06_one_step` runs one, and what it prints must match `expected/ch06_one_step.txt`.

Examples under `pending/` use services or rescue steps, which `itinera` does not have yet. They are not built; each moves to `examples/` when its feature arrives.
