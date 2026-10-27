# The Itinera book

A book about [Itinera](https://github.com/itinera-dev/spec), a workflow framework that keeps business rules separate from flow control, written once for each language. You pick the language you build with, and read a whole book written for it, with examples that compile and run.

## Status

Published at <https://itinera-dev.org/book/>, built by [the site](site/README.md). The first book is for Rust, in [rust/](rust/), with [itinera-rs](https://github.com/itinera-dev/itinera-rs): the book for its latest release is at <https://itinera-dev.org/book/rust/>, the book ahead of the next release at <https://itinera-dev.org/book/rust/next/>, and each release's book, frozen, at `/book/rust/VERSION/` once the release is tagged, as [BOOK.md](BOOK.md#where-editions-are-published) says. The TypeScript book, in [typescript/](typescript/), follows [itinera-ts](https://github.com/itinera-dev/itinera-ts) in the same way, at <https://itinera-dev.org/book/typescript/>, <https://itinera-dev.org/book/typescript/next/> and `/book/typescript/VERSION/`.

## Who it is for

People building software for others with Itinera, and the AI agents doing that work with them. It explains why Itinera keeps flow decisions out of business code, and shows how to write steps, workflows and policies, run them, observe them and test them. It is not a reference manual: the [specification](https://github.com/itinera-dev/spec) defines what Itinera does, and each language's API reference documents every item.

## What is here

- [BOOK.md](BOOK.md): what the book contains and how it is written, the plan every language's book follows.
- [rust/](rust/): the Rust book, its chapters in [rust/chapters/](rust/chapters/README.md) and its examples beside them.
- [typescript/](typescript/): the TypeScript book, its chapters in [typescript/chapters/](typescript/chapters/README.md) and its examples beside them.
- [FORMAT.md](FORMAT.md): how a book is stored, and how its chapters show compiled code.
- [AGENTS.md](AGENTS.md): the working manual for anyone, human or agent, working in this repository.

Built with AI under the terms of [A manifesto for software engineering with AI](https://marlon-sousa.com/blog/manifesto/); see [how Itinera is built](https://github.com/itinera-dev/.github/blob/main/CONTRIBUTING.md#how-itinera-is-built).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
