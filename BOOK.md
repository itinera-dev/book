# The Itinera book

What the Itinera book contains, and how it is written. There is one book for each language: a reader picks a language and reads a whole book written for it. This document is to the books what the specification is to the implementations: it is written once, without naming a language, and every language's book follows it. The books share their story, the shape of their chapters and their voice; each is written for its own language, naming it, showing its syntax in the prose, and covering what its readers need in as much detail as they need. Each chapter, once this document lists it, becomes its own issue, so the work can be split across sessions and languages.

## Who the book is for

The book is for people building software for others with Itinera, and for the AI agents doing that work with them or for them. It is not a technical manual. The specification defines the behaviour and the API reference of each language documents every item; the book links to them for readers who want to go further, and never repeats them.

A reader finishes the book able to:

1. tell whether Itinera fits a problem, and why;
2. turn a piece of business code into steps, a workflow and policies;
3. run it, observe it and test it;
4. explain to someone else why the flow lives where it does.

An agent working with the book gains what a person does, and one thing more: in an Itinera codebase, changing a business rule needs only its step, and changing the flow needs only one workflow, so the context an agent needs is given to it rather than left for it to rebuild.

The book follows the message and the design of the Itinera site, [itinera-dev.github.io](https://itinera-dev.github.io/), which owns them: its slogan, "Separate process from logic. Make agentic code safer.", its argument on the "Why Itinera" page, its colours and its logo. The book teaches what the site claims, with code that compiles and runs, and links to the site rather than retelling it.

## How the book is written

### Voice

1. Plain language, warm and personal, aimed at a developer who knows their language but has never heard of Itinera. The book talks to the reader as "you", and the people who make Itinera speak as "we", the way a colleague would explain it over coffee: it says hello, says why something matters before how it works, and admits when the usual way is perfectly reasonable. Being friendly never costs precision: the rules stay exact, and the vocabulary stays the specification's.
2. The vocabulary of the specification's chapter 1 is used exactly, since its words are not synonyms of each other. Each word is explained in plain language the first time the book uses it, and the glossary collects them.
3. The book never describes behaviour the specification does not define. Where a language makes a rule impossible to express, the book shows it as a compile error, with the real message.
4. Everything is written for a screen reader: headings, lists, prose and simple tables. Diagrams are Mermaid, with `accTitle` and `accDescr`, always followed by text that says everything they show.

### One story: the shop

Every example belongs to one small online shop taking orders, which grows a little in each chapter:

1. it charges an order, less any discount;
2. it reserves the stock and ships the order through a carrier, falling back to another carrier when the first one fails;
3. it audits every successful charge;
4. it tells the customer how the order went.

Its customers keep their names and order numbers from chapter to chapter, and a chapter adds a customer whenever it needs an order that no earlier one gives, so the book never promises how many there are.

Each chapter's example is a complete program that compiles, runs and shows what it prints. Every example runs on its own, so a reader can start at any chapter.

### The shape of a chapter

From chapter 3 on, every chapter teaches one topic; chapters 1 and 2 make Part 1's argument. Every chapter has the same four sections, in this order, so a screen reader user can jump straight to the part they need and an agent knows where to look:

1. **What you want to do,** in one or two sentences, in the shop's terms.
2. **The code,** with what it prints underneath.
3. **How it works,** in prose or a short numbered list. A rule that a reader or an agent must not break is written in bold where the chapter says it, such as "**a step's contributions are committed to the data bag only when its attempt succeeds**".
4. **Outcomes:** one sentence saying what the chapter has shown, starting "We have", then a table of one to three rows, as "The Outcomes table" below says.

A chapter aims for 400 to 800 words of prose, not counting its code, its output and its Outcomes table; chapter 1 and chapter 2 may reach 1,300. A chapter that needs more is teaching two topics, and is split.

### The Outcomes table

The table has two columns, headed "The usual way" and "With Itinera". Each row is one contrast, with one sentence in each cell. For a chapter on a step's inputs, it reads:

We have shown how a step asks for the data it needs.

| The usual way | With Itinera |
|---|---|
| A function gets its data through parameters, shared state or objects it reaches for, and finding where a value came from means following every caller. | Every step asks for its data the same way, by name, so everything it reads is listed where the step is declared. |

1. The left cell says what competent code usually does, fairly, as a habit rather than a mistake: "usually" or "often", never "always". It names no library or framework, and admits it when the usual way is perfectly reasonable.
2. The right cell claims only what the chapter's own code shows; in chapter 1, which has no Itinera yet, what chapter 2 shows.
3. No row repeats a row of an earlier chapter.
4. The section's heading is "Outcomes", but its text uses "outcome" only in the specification's sense (success, failure or skip): its sentence starts "We have", never "the outcome is".

Chapter 1 is the book's one long comparison, written in code. Every other chapter compares only in its Outcomes table, and no chapter writes the shop the usual way a second time.

### How the chapters follow one another

The chapters are not planned ahead. Each one starts from something the reader needs and cannot yet do with what the book has taught, and teaches only that, so that what is worth teaching is found as the book is written, and no idea is introduced twice.

1. The book starts with steps: a step, a step asking for data, a step saying how it went, a step passing data on, a step reporting what it did. A step runs on its own, called directly, before there is any workflow.
2. Then the reader asks how steps make a journey, and the book teaches workflows, with the steps already known.
3. Each later need, such as a step that needs more data, a service to reach the outside world, or a decision after a step's outcome, is taught on the steps and workflows the reader already knows.
4. Each chapter separates one more concern. It says what problem the separation solves, and, when a new idea makes it possible, what problem the earlier way still had.

### Each idea in its own chapter

A reader meets each idea in the chapter that teaches it, and Itinera is flexible enough for every chapter to show only what the reader has learnt so far.

1. The welcome pages and chapters 1 and 2 show ideas so that their argument can be followed. From chapter 3 on, each idea is taught in its own chapter, even one those pages showed: every page, including the appendices, uses in its code and prose only what it, or an earlier chapter from chapter 3 on, teaches. From chapter 3 on, a page never names a later chapter or an idea the book has not taught yet, not even in passing.
2. An idea is taught by the first chapter, from chapter 3 on, that shows it and explains it. That includes anything the book explains rather than assumes, such as an attribute of the language's package or a channel of its standard library.
3. Code a chapter does not show may use anything, as long as the prose never relies on it. Faked outside systems, such as the bank or a carrier, print what they do, so the output tells what happened.
4. The paths a chapter talks about come from the data its orders start with: each path is run by an order whose data takes it there, and what it prints is shown. Every path the prose or a diagram talks about is run by an order; the prose or a diagram never talks about a branch that no order takes. No hidden counter or flag decides a path, and the prose never asks the reader to change a value to see another path. When a page's point is a failure that passes, such as a retry in the usual code, the state that decides it is said plainly in the prose. A chapter with no program, or whose program runs nothing because the book has not yet taught how to run what it declares, says so, and describes only what it shows.
5. Once taught, an idea may be used again, with a link back to its chapter, and is never taught a second time.
6. When a page changes, every other page that refers to it, quotes its code or relies on what it shows, before it or after it, is read again, and still holds.

### Synchronous and asynchronous together

Itinera's synchronous and asynchronous APIs differ little, so the book teaches the two execution modes in one chapter and has no part on asynchronous code. From that chapter on, each chapter shows the synchronous form first; where the asynchronous form differs, the same example follows in its asynchronous form, with one sentence saying what changed. A language whose executors accept only one execution mode shows only that one.

## Contents

### Welcome

Two short pages come before Part 1, unnumbered, so a reader meets Itinera and the book before the argument starts.

1. **Welcome to Itinera.** Hello, and what this book is for. What Itinera is, in a few sentences: steps, workflows, policies and the executor, and one meaning across languages, defined by the specification. Where the name comes from: *itinera* is the Latin plural of *iter*, a journey, and every execution of a workflow for one order is a journey. How the book goes: from steps, to workflows, to everything a journey needs around them. The argument for Itinera is chapter 1's, and the page links to the site's "Why Itinera" rather than retelling it.
2. **How to read this book.** The conventions every page shares: the book for each language and the language selector of the web pages; the shop and its customers; the shape of a chapter and its Outcomes table; code that is always a region of a complete program, and the marks a caption may carry (the asynchronous form, not compiled yet, does not compile); words with one meaning, the glossary and the cheat sheet; diagrams always followed by text; where to go further.

### Part 1: Why Itinera

The part makes one argument: the hardest part of understanding a system that mixes business rules with flow control is knowing what is going on, and with Itinera you find out by reading one workflow.

1. **But why Itinera?** The shop written the usual way, before any Itinera: a checkout that reserves, charges, renews an expired card, falls back to another carrier, audits, logs and tells the customer, with those decisions spread across functions and modules. The reader is asked simple questions, such as "what happens to an order when the card is declined twice?", and the chapter follows the code to answer them, showing how much has to be read, and how easily a decision is missed. It is the book's one long comparison.
2. **The big picture.** The same checkout, with the same orders and one more for each path they do not take, written with Itinera, as a quick start: the program's workflow first, since reading it alone tells what happens to every order, then the steps, the policies and what the workflow supplies them, with comments that say what each piece is for and answer chapter 1's questions where the code answers them, then what it prints, and a diagram of a journey through the shop with the text that says everything it shows. The prose frames the program and explains none of it: each piece has a chapter of its own later, found as the book needs it.

### Part 2: Getting started

3. **Installing Itinera.** The package, the language version it needs, and the one thing to know about its options: the defaults are enough to start, and a chapter that needs another option says how to turn it on. It is a library, with nothing else to run.
4. **A first step.** The charge written as a step: a struct holding the price it asks for as an input, and a `run` that does the work and says how it went, succeeded or failed with a reason, and nothing else. The program declares the step and runs nothing, and the chapter ends on the question that raises: nothing builds the step with an order's price or runs it, so how does the shop charge an order?
5. **A first journey.** The charge of chapter 4 run for two orders, one it charges and one with nothing to charge: a workflow listing the step under a name, each order's initial data, an instance of the workflow for each order, an executor that runs it as a journey, and the result saying how the journey ended, succeeded or failed, a failure being a business outcome rather than an error of the program. The chapter ends on what the result does not say: what the charge did.
6. **Passing data on.** The charge gives back the receipt the bank gave it, and a second step, the shipping, puts it in the parcel: a contribution, the data bag, a later step reading a contribution as an ordinary input, the steps running in the order the workflow lists them, a failed attempt's contributions never committed, and the data bag read back as the journey's output. The chapter ends on what the steps still do on their own: each reaches the bank or the carrier by itself, so it can only ever use that one.
7. **Giving a step the tools it needs.** The bank and the carrier become services: traits the steps ask for by a key and use without knowing which implementation they got, which the workflow's providers make once for each journey, the first time a step asks. The chapter ends on what the shop still cannot see: everything it learns comes from what its own code prints.
8. **Seeing what happened.** The shop's log written as a reporter that the workflow lists: the events every journey reports, saying who decided what and when, including which provider served which step and which implementation it made, in the same order and shape whoever wrote the steps, which code written the usual way rarely has, the fields every event carries, its body as an enum, the executor's events naming data and never carrying its value, and delivery as things happen. The chapter ends on the one line still printed by a step: what only the shipping knows.
9. **Saying what only a step knows.** The shipping reports that the parcel left through its step reporter, as an event of its own in the journey's stream, stamped with the step and its attempt, so the record of who did what holds what only the step knows, and delivered before the call returns; a step never emits what the executor already records. The chapter ends on another audience for the same events: the customer service desk, which wants only the failures.
10. **More than one reporter.** The desk as a second reporter, reading the same record of who decided what as the log, for a purpose of its own: the dispatcher fanning every event out to each reporter in the order the workflow lists them, the journey waiting for every reporter, and adding a reporter changing nothing else. The chapter ends on what the desk cannot say: which order failed.
11. **Naming a journey.** An ID generator names each journey after its order, so every event says which order each decision was about: the random default, the generator as the workflow's decision, reading the initial data, and the workflow's name every event also carries, which says what the workflow does while the ID tells its journeys apart. The chapter ends on why Gil's order failed: the shipping gets one attempt, with one carrier.
12. **Trying again.** The bank and the carrier now return an error when their own systems fail, and the workflow gives the charge and the shipping a retry budget of one, turning off retries for the charge's errors, since the bank may already have taken the money: an abnormal termination as an error the step did not handle, which never aborts the journey, the retry budget alone deciding whether a step that failed is tried again, failures and abnormal terminations spending the same budget, a step whose abnormal terminations are never retried, a fresh step built for each attempt, a failed attempt's contributions thrown away, the step given up once its budget is spent, and a panic outside the model. The chapter ends on why Gil's second attempt fails too: it gets the same carrier.
13. **A carrier for each attempt.** A provider made for every attempt, which gives the shipping the main carrier on its first attempt and the backup carrier, which takes heavier parcels, on the next, so Gil's order ships on its second attempt: the provider reading the attempt number, and a service made anew for each attempt instead of once for each journey. The chapter ends on the order still failing: Jo's expired card.
14. **Repairing, then trying once more.** A rescue step for the charge, which asks the bank for the card that replaced an expired one and contributes it, so the charge runs once more and Jo's order is charged: the rescue step reading why the step failed, deciding whether it can repair it, its contribution replacing the old value in the data bag, the step tried again after a repair, and the charge's retry budget dropped, since a step with a rescue step has none. The chapter ends on another audience for what the charge did: the accountants, who want every charge written down.
15. **Reacting to what a step did.** An audit step policy attached to the charge, whose hook on the step's success reads the step's name, the order's price and the receipt the charge gave back, and contributes the audit line: a step policy and its hook, what a hook can ask for, carrying on as its decision, and its contribution kept with the step's. The chapter ends on Kim's gift card, which has nothing to ship but still gets a parcel booked.
16. **Skipping a step.** The shipping skips a gift card, with a reason: a skip as the third outcome, the step's own decision for a reason of its business, its contributions thrown away, the journey moving on, and a skip never retried. The chapter ends on what no customer reads: whether their journey succeeded is only printed by `main`.
17. **Doing something when the journey ends.** A notify workflow policy writes each customer's message once the journey has ended, whichever step ended it, and a new customer's order fails when the bank has no replacement for a long-expired card, so the rescue step fails: a workflow policy and its hooks on the journey's success and on its failure, exactly one of them running for every journey that succeeds or fails, optional requests and the absences they record, and a hook that decides nothing. The chapter ends on the message existing only in the data bag.
18. **Letting policies act on the outside world.** The notify policy sends the message through a role the workflow offers, implemented with the channel the workflow is configured with: a role as operations and never data, the workflow's fields as its configuration, and only policies asking for roles. The chapter ends on a discount the shop never applies.
19. **An optional input.** A pricing step of its own takes an order's discount when there is one and gives back the amount, which the charge charges without knowing prices or discounts: an optional input, initial data leaving out what an order does not have, the absence recorded, the step deciding what the absence means, and each rule of the business a step of its own. The chapter ends on the warehouse describing each parcel as one value, its weight and its size, which the shipping cannot read.
20. **Deciding where a step's data comes from.** The workflow's input adapter takes the weight out of the warehouse's parcel and gives it to the shipping, which still asks only for a weight: an input adapter satisfying what a step needs in the form it needs, so the step depends on nothing else, belonging to the workflow, asked once for each input as the step is built, supplying a value or leaving the input to the data bag, its uses (taking a field out of a value, converting a type, fetching data only when a step needs it) and never a rule of the business, reading and never writing, and recording what it supplied without its value. The chapter ends on how `main` runs the orders: a new executor for each, waiting for each journey to end.

### Part 4: Running it for real

21. **Running many journeys.** One executor runs every order of the day, one journey after another: an executor keeping nothing between journeys, the compiler refusing to run an instance twice, and an executor running one journey at a time. The chapter ends on what `main` learns from Lee's result: only that the journey failed.
22. **Reading the result.** `main` reads how each journey ended, and a new customer's order arrives without a card: the status as an enum, a failure's cause and reason, the result naming no step, an abort when something that must not happen did, such as required data missing, and an aborted journey with no output, running no workflow hook and never resumed. The chapter ends on the shop's real bank and carriers answering over the network, while the journey holds the program's thread.

### The chapters to come

They are written one at a time, as "How the chapters follow one another" says, and each is added above, with its entry, once it is written. Until a topic has its chapter, it stays on this list of what Tier 1 still has to teach, in no particular order:

1. the package's options other than the defaults;
2. steps: an input or a service asked for under another key;
3. what can refuse a journey before it starts, and the two execution modes;
4. a reporter that fails, a reporter added for one journey only, a ready-made reporter that prints every event, and global metadata;
5. the other hooks of step policies, what else a hook may request, and ending a journey early;
6. a provider for some steps only, data a provider supplies with a service, a service a provider gets already boxed, and what may be attached to a rescue step's name;
7. the listing, violations, refusals, and the mistakes the language refuses to compile;
8. declaring by hand, without the language's shorthand;
9. testing a step, a policy's hook or an input adapter alone, testing the flow with scripted steps, and testing a whole journey with fake services and approved paths;
10. deciding where a decision belongs, where Itinera fits and where it does not, adopting Itinera in existing code, using a step in another workflow, and working with agents.

### Appendices

1. **Cheat sheet.** Every declaration the language offers, one line each, in the order the book teaches them, with a link to the chapter that teaches it.
2. **Glossary.** Every word of the specification's vocabulary, in plain language, linking to the specification's definition.
3. **What this version supports.** The specification version, tier and capabilities of the release the book covers, its conformance report, and what later tiers will add.

### Later tiers

Each tier adds a part, in the same style, once its proposals are accepted and the language implements them: moving a journey to another workflow (tier 2), running a workflow from inside another (tier 3), and durable runs.

## What each language provides

For every chapter, each language's book provides:

1. the chapter's prose, following this document and written for that language, such as how recoverable and unrecoverable failures work in it. A new language's book usually starts as a copy of an existing one and is then rewritten for its language;
2. the chapter's example as a complete program, with marked regions the book quotes, and its recorded output;
3. for the chapter on mistakes caught before a journey, compile-fail cases with their recorded compiler messages, where the language turns mistakes into compile errors.

A language that lacks a feature a chapter needs shows that chapter as a short section saying the feature is not in this language yet, with a link to its implementation issue.

## How the examples are checked

1. Each language's directory in this repository depends on a pinned version of that language's Itinera package, and this repository's CI compiles every example, runs it and compares its output with the recorded one, in one job per language.
2. Each language's own repository pins a commit of this repository and runs its edition of the book on every pull request against its own code, as a required check. A change that breaks the book is caught where it is made.
3. A change that breaks the book on purpose comes as a pair of pull requests, one here and one in the language's repository. The book's pull request updates the examples and pins the commit of the language's pull request; the language's pull request then pins the commit of the book's pull request, and both go green together. Until the next release, the book stays pinned to that commit rather than to a released version.
4. After a language's release, its edition moves to the released version and is tagged, for example `rust-0.1.0-beta.1`. The site quotes only from such tags, so each published edition matches a published package.

## Where editions are published

1. Each language's editions are published at three kinds of address:
   1. the latest release at `/book/LANGUAGE/`, such as `/book/rust/`;
   2. every release, frozen, at `/book/LANGUAGE/VERSION/`, such as `/book/rust/0.1.0-beta.1/`, so a link to it never changes;
   3. the main branch, ahead of any release, at `/book/LANGUAGE/next/`.

   Until a language's first release, its main branch is published at `/book/LANGUAGE/`, so links to the book work from the start.
2. From a language's first release, a published page keeps its address in every later edition: a new chapter goes where it renumbers no published page, and a page that is moved or renamed leaves a redirect at its old address. Before the first release, pages may be renumbered and renamed freely.

## Open points

1. **Where the prose lives:** with the examples, in each language's directory of this repository, so one pull request changes both.
