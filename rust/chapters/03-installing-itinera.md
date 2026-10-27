# Installing Itinera

## What you want to do

The shop is a Rust program you already have. You want to add Itinera to it, so that you can start writing the shop with it.

## The code

Itinera for Rust is one crate, `itinera`, and you add it as you add any other dependency. It needs Rust 1.85 or later.

You add it by its version, from crates.io:

- in your `Cargo.toml`, under `[dependencies]`, as `itinera = "0.1.0-beta.5"`;
- or from the command line, with `cargo add itinera@0.1.0-beta.5`.

That is all you need to start. The crate has a few Cargo features that turn on more. Its defaults are what the shop needs for now, and when the book needs another feature, it says how to turn it on.

There is no program in this chapter, so nothing is printed.

## How it works

1. **It is a library, and nothing else.** Itinera runs inside your program. There is no server to start, no database to create and no background process to keep running.
2. **One crate brings everything.** `itinera` brings in two crates of its own: `itinera-core`, which holds the implementation, and `itinera-macros`, which holds the attributes and derives you declare things with. Don't add either yourself: **every path you write goes through `itinera`**, and you never need to know the other two are there.
3. **Each release says which specification it implements.** Itinera is defined once, by a [specification](https://github.com/itinera-dev/spec) that names no programming language, and each release of `itinera` says which version of it the release implements. This book is written for `0.1.0-beta.5`. It is a beta: until `0.1.0`, features may be added and the API may still change.
4. **You can pin it exactly.** Since a beta may still change its API, `itinera = "=0.1.0-beta.5"` asks for exactly this release, where the `=` keeps Cargo from choosing a later one. The book's own examples are pinned too, each built against exactly the code it was written for.

## Outcomes

We have shown that Itinera is one dependency of your program, with nothing else to install or run.

| The usual way | With Itinera |
|---|---|
| A workflow engine usually runs as a service of its own, with its own storage and workers to deploy, which lets work outlive a crash of the program that started it. | Itinera is a crate inside your program, and one dependency line is the whole installation. |
