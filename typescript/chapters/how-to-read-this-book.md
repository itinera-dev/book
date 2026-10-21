# How to read this book

A few things work the same way on every page. Here they are, once.

## A book for TypeScript

This is the TypeScript book of Itinera. Every program in it is TypeScript, every output is what that program printed, and every detail is about `itinera`, the npm package. Each language that has an Itinera gets a book of its own, with the same shop, the same chapters and the same words. On the web pages, the **programming language** selector at the top of every page takes you to the same page in another language's book.

## Meet the shop

Every example belongs to the same small online shop. In the first chapter you meet its first four customers:

1. **Ann**, whose order goes through without trouble.
2. **Ben**, whose card has expired, so the shop has to get his new one.
3. **Cat**, whose parcel is too heavy for the main carrier, so it goes to the backup carrier.
4. **Dan**, whose card has expired too, and whose bank refuses the new one, because his order is above its limit.

Their orders are numbered A-1 to A-4. A chapter brings in a new customer when it needs an order none of them gives, and a customer keeps the same name and order number wherever they appear.

## The shape of a chapter

Every chapter teaches one thing, in four sections:

1. **What you want to do:** the problem, in the shop's terms.
2. **The code:** the program, with what it prints underneath.
3. **How it works:** what happened, and why. A rule you must not break is in **bold**.
4. **Outcomes:** what the chapter has shown, in one sentence, and a small table that puts the usual way of writing it beside the way with Itinera. The usual way is often perfectly reasonable; the table only shows what changes. The heading names what the chapter gives you; a step's outcome, which the book teaches with the first step, is another thing.

## Code you can trust

Every piece of code in the book is part of a complete program, compiled and run every time the book changes, and every output is what that program really printed. Each block of code has a caption naming its program, with a link to the whole of it; to run it yourself, follow the [README of the book's `typescript` directory](../README.md), then run `node build/examples/NAME.js` there.

Every program is compiled with `tsc`, with `strict`, `experimentalDecorators` and `emitDecoratorMetadata` turned on, and runs as an ES module on Node. A step's `run`, a hook and a reporter's methods may be ordinary or `async`, and the executor waits for whichever it gets, so there is only one way to run a journey and only one form of each program to show.

Some mistakes can't even be written: TypeScript's types refuse them. Such a program's caption says it **does not compile**, and what the compiler says follows, word for word.

## Words with one meaning

Itinera has a small vocabulary: step, workflow, policy, hook, executor, journey, data bag and a few more. Each word has exactly one meaning, defined by the specification, and the book never uses two of them as if they were the same. Where a chapter introduces a word, it is in **bold**, as is a rule you must not break. The glossary at the end of the book has them all, and the cheat sheet beside it has every declaration on one page.

## Diagrams, read aloud

A diagram is always followed by text that says everything it shows. If you read the book with a screen reader, or simply prefer words, you miss nothing by skipping the picture.

## Going further

The book teaches; it doesn't list every detail. The [specification](https://github.com/itinera-dev/spec) defines what every Itinera must do, and `itinera` documents every item it offers where the item is declared, each with an example, so your editor shows it as you write. Its source is in [itinera-ts](https://github.com/itinera-dev/itinera-ts).
