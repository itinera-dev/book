# How the book is laid out

Where each part of the book lives in this repository, and how a chapter shows code that is compiled and run. [BOOK.md](BOOK.md) says what the book contains; this file says how it is stored.

## Each language's book

Each language has a book of its own, in its own directory, such as [rust/](rust/). Every book follows BOOK.md: the same shop, the same parts, the same shape of a chapter and the same voice. Beyond that, each is written for its language: it names the language, shows its syntax in the prose, and covers what its readers need, in as much detail as they need. A new language's book usually starts as a copy of an existing one, and is then rewritten for its language.

1. Each chapter is one file under the language's `chapters/`, named after its number and title, for example `rust/chapters/02-the-big-picture.md`. The `chapters/README.md` next to it lists the parts and chapters in reading order.
2. Each appendix is a file named `appendix-N-short-name.md`.
3. Each welcome page before Part 1 is a file named after its title, without a number, for example `how-to-read-this-book.md`. The contents list them unnumbered, under the heading "Welcome".
4. A chapter opens with its title as the only level 1 heading, and has the four sections of BOOK.md as level 2 headings, in order: "What you want to do", "The code", "How it works" and "Outcomes". It may use level 3 headings inside the first three, and a level 3 heading never stands alone: it has at least one other under the same level 2 heading. "Outcomes" holds only its sentence and its table.
5. An appendix or a welcome page opens with its own title, and chooses its own level 2 and level 3 headings.
6. Every name a chapter gives to an example, a tested program or a compile error starts with the chapter's number, such as `ch02_big_picture`, or with the appendix's, such as `appendix1_cheat_sheet`, so names never clash between chapters.
7. Any chapter or appendix may show a region of another chapter's example, by naming it, so the same code is never written twice.
8. A chapter never holds code that is meant to compile. Code that compiles, what it prints and what the compiler refuses are named by directives, below, so every listing in the book is checked. A short fragment that is not a program, such as one attribute or a type's name, may be written inline in the prose.

## Directives

A directive is a fenced block whose info string is the directive's kind, holding `key: value` lines. It names a file of the language's directory, and the web pages replace it with that file's content. On GitHub a directive shows as a small block naming what will be there.

### Example

````markdown
```example
name: ch02_big_picture
region: steps
```
````

The region of the example `ch02_big_picture` between the comments `ANCHOR: steps` and `ANCHOR_END: steps`, with every other anchor line removed. Without `region`, the whole example, anchor lines removed. An example is a complete program; the region is only what the chapter shows of it.

### Output

````markdown
```output
name: ch02_big_picture
```
````

What the example `ch02_big_picture` prints, as recorded next to it. An example that is still pending (see "Each language", below) has no recorded output, so no `output` directive names it: the chapter says in its prose what the program does when it runs, and gains the directive when the example stops being pending.

### Test run

````markdown
```test-run
name: chNN_testing_a_step_alone
```
````

What running the tests of the tested program `chNN_testing_a_step_alone` prints, as recorded next to it: the report of the language's test runner, naming each test and whether it passed. An `example` directive shows a region of a tested program the same way as a region of an example.

### Compile error

````markdown
```compile-error
name: chNN_role_not_provided
```
````

A program the language refuses to compile, followed by the compiler's message, as recorded next to it.

### Asynchronous

A directive may add `asynchronous: name`, naming the example's asynchronous form. Where the asynchronous form differs, the web pages show it after the synchronous one; where it does not, it is left out. A language with only one execution mode never uses the key.

## Each language

Each language has a directory of its own, named after it, for example [rust/](rust/). Its README says how to build and run its examples. Inside it:

1. `chapters/`: the language's book, as above;

2. `examples/NAME.EXT`: each example, a complete program, compiled and run in CI. Its comments describe the code as it is, and it holds no line, import, helper or region marker that nothing uses;
3. `expected/NAME.txt`: what each example prints, compared with what it prints in CI;
4. `compile-errors/NAME.EXT` and `compile-errors/NAME.txt`: each program that must not compile, and the compiler's message. The message is every error the compiler gives, as it renders it, without warnings, the compiler's closing notes or the build tool's progress lines. Paths in it are written as a reader of this repository sees them: the program as `compile-errors/NAME.EXT`, and a file of the implementation as a path inside its repository, such as `itinera-rs/crates/...`, never as a path on the machine that recorded it;
5. `tested/NAME.EXT` and `tested/NAME.txt`: each program whose tests a chapter shows, and the report of running them. The program's tests sit beside its code, as a reader's own tests would, and only they use the language's test support, such as a package option turned on for tests only. CI builds the program as it would ship, without anything that is only for tests, so a program that uses test support outside its tests fails the build, then runs its tests and compares their report with `tested/NAME.txt`. Like an example, it holds no line that nothing uses;
6. `pending/NAME.EXT`: examples that use a feature the language does not have yet. They are not compiled and have no recorded output. When the feature arrives, each moves to `examples/` and gains its recorded output, and the chapters showing it gain their `output` directives. Pending examples exist only while a language prepares a release: a released edition has none, and a feature its release lacks is shown as BOOK.md says.

An example prints the same thing on every run, except for an event's timestamp and a generated journey ID. The recorded output keeps the values of the run that recorded it, so the book shows a real run, and the check compares those two as placeholders. Nothing else that changes from run to run is printed. The same holds for a test run's report, whose time taken is the one placeholder; its tests run one at a time, in the order of their names.

## Web pages

The web pages of the book are built by the site in [site/](site/), which reads each language's directory at build time and keeps no copy of it. For each language, an `example` becomes the named region of the program, highlighted and captioned with the program's name, followed by its asynchronous form only where that region differs and the language has one; a program still in `pending/` is marked as not compiled yet, and fails the build of a released edition. An `output` becomes the recorded output, captioned as what the program prints, followed in the same way by what the asynchronous form prints where that differs; a `test-run` becomes the recorded report, captioned as what running the program's tests prints; and a `compile-error` becomes the program, captioned as one that does not compile, followed by the compiler's message. A directive that names a file which does not exist fails the build, naming the chapter and the file. Links between chapters become links between the edition's pages, and links to other files of this repository point to them on GitHub.
