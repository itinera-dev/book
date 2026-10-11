# How the book is laid out

Where each part of the book lives in this repository, and how the shared text names the code each language supplies. [BOOK.md](BOOK.md) says what the book contains; this file says how it is stored.

## The shared text

1. Each chapter is one file under [chapters/](chapters/), named after its number and title, for example `chapters/06-a-workflow-of-one-step.md`. [chapters/README.md](chapters/README.md) lists the parts and chapters in reading order.
2. Each "Without Itinera" section is a file of its own, named after the part it closes, for example `chapters/part-2-without-itinera.md`.
3. Each appendix is a file named `chapters/appendix-N-short-name.md`.
4. A chapter opens with its title as the only level 1 heading, and has the four sections of BOOK.md as level 2 headings, in order: "What you want to do", "The code", "How it works" and "Remember". It may use level 3 headings inside them.
5. A "Without Itinera" section opens with the level 1 heading "Without Itinera", and an appendix with its own title; both choose their own level 2 and level 3 headings.
6. Every name a chapter gives to an example, a note or a compile error starts with the chapter's number, such as `ch06_one_step`, with the part's for a "Without Itinera" section, such as `part2_calls_in_a_row`, or with the appendix's, such as `appendix1_cheat_sheet`, so names never clash between chapters.
7. Any chapter, "Without Itinera" section or appendix may show a region of another chapter's example, by naming it, so the same code is never written twice.
8. The shared text never names a language. Code, output and anything that differs between languages are named by directives, below, and each language supplies them.

## Directives

A directive is a fenced block whose info string is the directive's kind, holding `key: value` lines. Each language's edition of the book replaces it with what that language supplies. On GitHub a directive shows as a small block naming what will be there.

### Example

````markdown
```example
name: ch06_one_step
region: step
```
````

The region of the example `ch06_one_step` between the comments `ANCHOR: step` and `ANCHOR_END: step`, with every other anchor line removed. Without `region`, the whole example, anchor lines removed. An example is a complete program; the region is only what the chapter shows of it.

### Output

````markdown
```output
name: ch06_one_step
```
````

What the example `ch06_one_step` prints, as recorded next to it. An example that is still pending (see "Each language", below) has no recorded output, so no `output` directive names it: the chapter says in its prose what the program does when it runs, and gains the directive when the example stops being pending.

### Note

````markdown
```note
name: ch05_installing
```
````

The language's own note of that name, inserted as prose. A note says what differs in that language, such as how its errors work. A chapter that needs no note for a language is shown without it, so a language may leave out a note it has nothing to say in.

### Compile error

````markdown
```compile-error
name: ch27_role_not_provided
```
````

A program the language refuses to compile, followed by the compiler's message, as recorded next to it. A language that cannot turn this mistake into a compile error supplies a note of the same name instead, saying how the mistake shows instead.

### Asynchronous

A directive may add `asynchronous: name`, naming the example's asynchronous form. Where a language's asynchronous form differs, the edition shows it after the synchronous one; where it does not, or the language has only one execution mode, it is left out.

## Each language

Each language has a directory of its own, named after it, for example [rust/](rust/). Its README says how to build and run its examples. Inside it:

1. `examples/NAME.EXT`: each example, a complete program, compiled and run in CI;
2. `expected/NAME.txt`: what each example prints, compared with what it prints in CI;
3. `compile-errors/NAME.EXT` and `compile-errors/NAME.txt`: each program that must not compile, and the compiler's message. The message keeps the lines that say what is wrong and point at the code, and leaves out the build tool's progress lines. Paths in it are written as a reader of this repository sees them: the program as `compile-errors/NAME.EXT`, and a file of the implementation as a path inside its repository, such as `itinera-rs/crates/...`, never as a path on the machine that recorded it;
4. `notes/NAME.md`: each note;
5. `pending/NAME.EXT`: examples that use a feature the language does not have yet. They are not compiled and have no recorded output. When the feature arrives, each moves to `examples/` and gains its recorded output, and the chapters showing it gain their `output` directives. Pending examples exist only while a language prepares a release: a released edition has none, and a feature its release lacks is shown as BOOK.md says, with a short note.

Every example prints the same thing on every run: a journey ID or a timestamp that changes from run to run is never printed.

## Web pages

The web pages of the book are built by the site in [site/](site/), which reads the chapters and each language's directory at build time and keeps no copy of them. For each edition, an `example` becomes the named region of the program, highlighted and captioned with the program's name, followed by its asynchronous form only where that region differs and the language has one; a program still in `pending/` is marked as not compiled yet, and fails the build of a released edition. An `output` becomes the recorded output, captioned as what the program prints, followed in the same way by what the asynchronous form prints where that differs; a `note` becomes the language's note, in a box headed for that language, or nothing when the language has none; and a `compile-error` becomes the program, captioned as one that does not compile, followed by the compiler's message, or the note that replaces it. A directive that names a file which does not exist fails the build, naming the chapter and the file. Links between chapters become links between the edition's pages, and links to other files of this repository point to them on GitHub.
