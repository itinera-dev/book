# How to read this book

A few things work the same way on every page of the book. Here they are, once, so you never have to wonder about them later.

## Pick your language

This is one book, written once and published in an edition for each programming language that has an Itinera. The text is the same in every edition; the code, what it prints and the notes about the language are not.

On the book's web pages, the **programming language** selector at the top of every page lists the editions. The one you are reading is marked. Pick another, and you land on the same page in that language, so you can compare two languages chapter by chapter. If you are reading the book's files instead, each language has a directory of its own, holding its programs, what they print and its notes.

## Meet the shop

Every example belongs to the same small online shop, and to the same four customers. In chapter 1 you meet them like this:

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

Every piece of code in the book is part of a complete program in the book's repository, checked every time the book changes, and every output you see is what that program really printed. Nothing is pasted in by hand, so nothing in the book can drift away from what Itinera does.

Each block of code has a caption naming the program it comes from, with a link to the whole program. A few captions carry a mark:

1. **The asynchronous form.** A step, a hook or a reporter can be synchronous or asynchronous, and the two forms differ very little. Each chapter shows the synchronous form first. Where the asynchronous form of the same code differs, it follows right after, and its caption says so. A language that offers only one of the two shows only that one.
2. **Not compiled yet.** A program that uses a feature not released yet is shown, but it isn't compiled until that feature arrives, and its caption says so.
3. **Does not compile.** Some mistakes can't even be written: the language refuses them before your program ever runs. When the book shows one, the caption says the program does not compile, and what the compiler says follows it, word for word.

Notes about your language appear in a box headed with its name. They say what is particular to that language, such as how it spells a declaration, and the text around them never does.

## Words with one meaning

Itinera has a small vocabulary: step, workflow, policy, hook, executor, journey, data bag and a few more. Each word has exactly one meaning, defined by the specification, and the book never uses two of them as if they were the same thing. Where a chapter introduces a word, it is in **bold**, with its meaning in plain words. When you forget one, the [glossary](appendix-2-glossary.md) has them all, and the [cheat sheet](appendix-1-cheat-sheet.md) has every declaration on one page.

## Diagrams, read aloud

A diagram is always followed by text that says everything it shows, usually a numbered list. If you read the book with a screen reader, or simply prefer words, you miss nothing by skipping the picture.

## Without Itinera

Every part but the last closes with a section called "Without Itinera". It shows the same shop written the usual way, as a competent developer would write it, puts the Itinera version beside it, and says what changed. The usual code is real code too, compiled and run like every other example, so it never does worse than it really would.

## Going further

The book teaches; it doesn't list every detail. When you want the exact rule, the [specification](https://github.com/itinera-dev/spec) defines what every Itinera must do, and your language's API reference documents every item it offers. The book links to both wherever they help.

Now, let's see a whole workflow in one go.
