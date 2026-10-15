# AGENTS.md: working in itinera-dev/book

The operating manual for anyone, human or agent, working in this repository. It is self-contained: reading only this file is enough to work here safely.

## What this repository is

The Itinera book, written once for each language. A reader picks a language and reads a whole book written for it, with every example compiled and run.

- [BOOK.md](BOOK.md) says what the book contains and how it is written, for every language. It is to the books what the specification is to the implementations.
- Each language has a directory of its own, for example `rust/`, holding that language's book, its examples as complete programs, and their recorded output. Languages are directories, never branches.
- [site/](site/) builds the web pages from each language's directory; every edition is rendered by the site on `main`: the latest release's from `main`, `next`'s from the `next` branch, and each release's frozen edition from the book's files at its tag, such as `rust-0.1.0-beta.1`.

## Where behaviour comes from

- What Itinera does is defined only by the specification in [itinera-dev/spec](https://github.com/itinera-dev/spec). The book teaches it and never adds to it.
- If a chapter needs behaviour the specification does not define, or the specification seems to contradict what an implementation does, stop and tell the maintainer. Never decide it in the book.
- How a language exposes Itinera is decided in that language's repository, such as [itinera-dev/itinera-rs](https://github.com/itinera-dev/itinera-rs). The book follows its released API.

## Vocabulary

Use the words defined in chapter 1 of the specification, [01-concepts.md](https://github.com/itinera-dev/spec/blob/main/spec/01-concepts.md), exactly as defined there; they are not synonyms of each other. The book explains each in plain language the first time it uses it, without changing its meaning.

## Hard rules

1. **BOOK.md comes first.** A chapter that departs from BOOK.md changes BOOK.md first, in a pull request of its own.
2. **The book is for people building software for others,** and for the agents working with them. It is not a technical manual: chapters that teach a task are named after what the reader wants to do, follow the shop of BOOK.md, and link to the specification and the API reference instead of repeating them.
3. **Every piece of code shown is checked in CI.** Code in the book is a marked region of a complete program in this repository, and the output shown is the program's recorded output, checked in CI. A mistake the language turns into a compile error is a compile-fail case instead, and the message shown is its recorded compiler message, checked in CI the same way. A program is never pasted into the prose; only a short fragment that is not a program, such as one attribute or a type's name, may be written inline.
4. **Each language has its own book, and all of them follow BOOK.md.** A language's book names its language and is written for it; the story, the parts, the shape of a chapter and the voice come from BOOK.md, and a change to those changes BOOK.md first.
5. **Screen-reader friendly writing.** Headings, lists, prose and simple tables. Diagrams are Mermaid diagrams in fenced `mermaid` blocks; each declares `accTitle` and `accDescr` and is accompanied by text, usually a numbered list, that says everything it shows. No ASCII-art diagrams, no arrows drawn with characters, no box-drawing trees.
6. **No code in workflow YAML.** Automation is Python, standard library only, in [itinera-dev/actions](https://github.com/itinera-dev/actions), with tests; a workflow only declares triggers and permissions and calls an action at a released version. The full rule is the "Automation" section of the [contributing guide](https://github.com/itinera-dev/.github/blob/main/CONTRIBUTING.md#automation).
7. **Test support stays in tests.** Code the book shows outside a test never uses what exists only for tests, such as a package option turned on for tests only, so a reader who copies it gets code that compiles in their own setup. A program whose tests a chapter shows is a tested program, as [FORMAT.md](FORMAT.md) says: its tests sit beside its code, and CI builds it as it would ship, without test support, before running its tests.

## Commits and pull requests

- Work on a branch; never push to `main` directly, except for a release, as the next item says.
- Only a release's changes go straight to `main`, with no pull request, and its merge back into `next` goes straight to `next`. They are:
  1. merging `next` into `main` at the new version, as item 5 of "How the examples are checked" in BOOK.md says;
  2. updating, on `main`, the pages that name the version;
  3. merging `main` back into `next`, which keeps naming the commit of the language's repository that it already names, as item 6 says.

  They are pushed once three checks pass: the language's examples, such as `cargo test --locked` in `rust/`; the site, with `npm run verify` in `site/`; and a separate reviewer agent. The maintainer then tags the last of these commits on `main`.
- Commit messages and pull request descriptions say what changed and why, in plain prose.
- Every change an agent makes is reviewed by a separate reviewer agent before its pull request is opened, or, for a release, before it is pushed. The reviewer checks it against this manual, BOOK.md and FORMAT.md, starting with "Each idea in its own chapter" in BOOK.md.

## Working with the maintainer

- Start every reply with a heading.
- When a design decision is open, present one decision at a time, with a recommendation, and wait for the answer.
