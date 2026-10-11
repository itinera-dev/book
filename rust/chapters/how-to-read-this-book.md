# How to read this book

A few things work the same way on every page of the book. Here they are, once, so you never have to wonder about them later.

## A book for Rust

This is the Rust book of Itinera. Every program in it is Rust, every output is what that program printed, and every detail is about `itinera`, the Rust crate. Each language that has an Itinera gets a book of its own, following the same plan: the same shop, the same chapters and the same words, each written for its language.

On the web pages, the **programming language** selector at the top of every page takes you to the same page in another language's book, so you can compare two languages chapter by chapter. Rust's is the first book, so for now there is nothing else to pick.

## Meet the shop

Every example belongs to the same small online shop, and to the same four customers. In [chapter 1](01-but-why-itinera.md) you meet them like this:

1. **Ann**, whose order goes through without trouble.
2. **Ben**, whose card has expired, so the shop has to get his new one.
3. **Cat**, whose parcel the main carrier cannot take, so it goes to the backup carrier.
4. **Dan**, whose card has expired too, and whose bank refuses the new one.

Their orders are numbered A-1, A-2, A-3 and A-4, in that order. Other chapters use only some of them, or give them other problems, such as a declined card or an address the carrier does not know, whatever the chapter is about. The names and the order numbers stay the same, so you never have to learn a new example.

## The shape of a chapter

Every chapter has the same four sections, so you can jump straight to the one you need:

1. **What you want to do:** the problem, in the shop's terms.
2. **The code:** the program, with what it prints underneath.
3. **How it works:** what happened, and why.
4. **Remember:** the few rules you must not break. If you only have a minute, read this one.

## Code you can trust

Every piece of code in the book is part of a complete program in the book's repository, built and run every time the book changes unless its caption says otherwise, and every output you see is what that program really printed. Nothing is pasted in by hand, so nothing in the book can drift away from what Itinera does. To run a program yourself, run `cargo run --example NAME` in the book's `rust` directory, with the name its caption gives.

Each block of code has a caption naming the program it comes from, with a link to the whole program. A few captions carry a mark:

1. **The asynchronous form.** A step, a hook or a reporter can be synchronous or asynchronous, and Rust offers both: an `async fn` where the synchronous form has a `fn`, run by `AsyncLocalExecutor` instead of `LocalExecutor`. The two forms differ very little, so each chapter shows the synchronous form first, and where the asynchronous form of the same code differs, it follows right after, with a caption that says so.
2. **Not compiled yet.** A program that uses a feature `itinera` does not have yet, such as services, is shown, but it waits in the book's `pending` directory and isn't compiled until that feature arrives. Its caption says so.
3. **Does not compile.** Some mistakes can't even be written: Rust's types and Itinera's macros refuse them before your program ever runs. When the book shows one, the caption says the program does not compile, and what the compiler says follows it, word for word.

## Words with one meaning

Itinera has a small vocabulary: step, workflow, policy, hook, executor, journey, data bag and a few more. Each word has exactly one meaning, defined by the specification, and the book never uses two of them as if they were the same thing. Where a chapter introduces a word, it is in **bold**, with its meaning in plain words. When you forget one, the [glossary](appendix-2-glossary.md) has them all, and the [cheat sheet](appendix-1-cheat-sheet.md) has every declaration on one page.

## Diagrams, read aloud

A diagram is always followed by text that says everything it shows, usually a numbered list. If you read the book with a screen reader, or simply prefer words, you miss nothing by skipping the picture.

## Without Itinera

Every part but the last closes with a section called "Without Itinera". It shows the same shop written the usual way, as a competent developer would write it, puts the Itinera version beside it, and says what changed. The usual code is real code too, compiled and run like every other example, so it never does worse than it really would.

## Going further

The book teaches; it doesn't list every detail. When you want the exact rule, the [specification](https://github.com/itinera-dev/spec) defines what every Itinera must do, and the API documentation of `itinera` documents every item the crate offers, with an example for each. The crate isn't on crates.io yet, so its documentation lives with its source, in [itinera-rs](https://github.com/itinera-dev/itinera-rs); once `itinera` is a dependency of your project, `cargo doc --open` builds it for the version you use. The book links to both wherever they help.

Now, let's see a whole workflow in one go.
