# The Itinera book

What the Itinera book contains, and how it is written. There is one book for each language: a reader picks a language and reads a whole book written for it. This document is to the books what the specification is to the implementations: it is written once, without naming a language, and every language's book follows it. The books share their story, their parts, the shape of their chapters and their voice; each is written for its own language, naming it, showing its syntax in the prose, and covering what its readers need in as much detail as they need. Each chapter and part below later becomes its own issue, so the work can be split across sessions and languages.

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

Every chapter teaches one topic, the one its entry under "Contents" names, and has the same four sections, in this order, so a screen reader user can jump straight to the part they need and an agent knows where to look:

1. **What you want to do,** in one or two sentences, in the shop's terms.
2. **The code,** with what it prints underneath.
3. **How it works,** in prose or a short numbered list. A rule that a reader or an agent must not break is written in bold where the chapter says it, such as "**a step's contributions are committed to the data bag only when its attempt succeeds**".
4. **Outcomes:** one sentence saying what the chapter has shown, starting "We have", then a table of one to three rows, as "The Outcomes table" below says.

A chapter aims for 400 to 800 words of prose, not counting its code, its output and its Outcomes table; chapter 1 and chapter 2 may reach 1,300. A chapter that needs more is teaching two topics, and is split.

### The Outcomes table

The table has two columns, headed "The usual way" and "With Itinera". Each row is one contrast, with one sentence in each cell. For chapter 7, it reads:

We have shown how a step asks for the data it needs.

| The usual way | With Itinera |
|---|---|
| A function gets its data through parameters, shared state or objects it reaches for, and finding where a value came from means following every caller. | Every step asks for its data the same way, by name, so everything it reads is listed where the step is declared. |

1. The left cell says what competent code usually does, fairly, as a habit rather than a mistake: "usually" or "often", never "always". It names no library or framework, and admits it when the usual way is perfectly reasonable.
2. The right cell claims only what the chapter's own code shows; in chapter 1, which has no Itinera yet, what chapter 2 shows.
3. No row repeats a row of an earlier chapter.
4. The section's heading is "Outcomes", but its text uses "outcome" only in the specification's sense (success, failure or skip): its sentence starts "We have", never "the outcome is".

Chapter 1 is the book's one long comparison, written in code. Every other chapter compares only in its Outcomes table, except chapter 2, which answers chapter 1's questions from the workflow, and chapter 28, which moves chapter 1's checkout into Itinera one step at a time. No chapter writes the shop the usual way a second time.

### Each idea in its own chapter

A reader meets each idea in the chapter that teaches it, and Itinera is flexible enough for every chapter to show only what the reader has learnt so far.

1. The welcome pages and chapters 1 and 2 show ideas so that their argument can be followed. From chapter 3 on, each idea is taught in its own chapter, even one those pages showed: every page, including the appendices, uses in its code and prose only what it, or an earlier chapter from chapter 3 on, teaches. A later idea may be named in one sentence with a link to the chapter that teaches it, never with its syntax or an explanation. Chapter 2 shows a whole declaration on purpose: each idea in it is named with a link, and explained no further than the chapter's own point needs.
2. An idea is taught by the first chapter, from chapter 3 on, whose entry below names it. Anything else the book explains rather than assumes, such as an attribute of the language's package or a channel of its standard library, is taught by the first chapter, from chapter 3 on, that shows it and explains it.
3. Code a chapter does not show may use anything, as long as the prose never relies on it. Faked outside systems, such as the bank or a carrier, print what they do, so the output tells what happened.
4. The paths a chapter talks about come from the data its orders start with: each path is run by an order whose data takes it there, and what it prints is shown. Every path the prose or a diagram talks about is run by an order; the prose or a diagram never talks about a branch that no order takes. No hidden counter or flag decides a path, and the prose never asks the reader to change a value to see another path. When a page's point is a failure that passes, such as a retry in the usual code, the state that decides it is said plainly in the prose.
5. Once taught, an idea may be used again, with a link back to its chapter, and is never taught a second time. Where the language has a reporter that prints every event, chapter 6 explains it, and any later chapter whose point is the events may use it.
6. When a page changes, every other page that refers to it, quotes its code or relies on what it shows, before it or after it, is read again, and still holds.

### Synchronous and asynchronous together

Itinera's synchronous and asynchronous APIs differ little, so the book has no separate chapter on asynchronous code. Each chapter shows the synchronous form first; where the asynchronous form differs, the same example follows in its asynchronous form, with one sentence saying what changed. A language whose executors accept only one execution mode shows only that one.

## Contents

### Welcome

Two short pages come before Part 1, unnumbered, so a reader meets Itinera and the book before the argument starts.

1. **Welcome to Itinera.** Hello, and what this book is for. What Itinera is, in a few sentences: steps, workflows, policies and the executor, and one meaning across languages, defined by the specification. Where the name comes from: *itinera* is the Latin plural of *iter*, a journey, and every execution of a workflow for one order is a journey. How the book is laid out, part by part. The argument for Itinera is chapter 1's, and the page links to the site's "Why Itinera" rather than retelling it.
2. **How to read this book.** The conventions every page shares: the book for each language and the language selector of the web pages; the shop and its customers; the shape of a chapter and its Outcomes table; code that is always a region of a complete program, and the marks a caption may carry (the asynchronous form, not compiled yet, does not compile); words with one meaning, the glossary and the cheat sheet; diagrams always followed by text; where to go further.

### Part 1: Why Itinera

The part makes one argument: the hardest part of understanding a system that mixes business rules with flow control is knowing what is going on, and with Itinera you find out by reading one workflow.

1. **But why Itinera?** The shop written the usual way, before any Itinera: a checkout that reserves, charges, renews an expired card, falls back to another carrier, audits, logs and tells the customer, with those decisions spread across functions and modules. The reader is asked simple questions, such as "what happens to an order when the card is declined twice?", and the chapter follows the code to answer them, showing how much has to be read, and how easily a decision is missed. It is the book's one long comparison.
2. **The big picture.** The same checkout, with the same orders and one more for each path they do not take, written with Itinera. The chapter shows the whole workflow declaration and the listing the language makes from it without running anything, runs the orders, and answers chapter 1's questions from the declaration alone, a short paragraph each. Each idea in it, such as steps, policies, retry budgets, rescue steps, services, the listing and the journey, is named in one sentence with a link to the chapter that teaches it, and explained only as far as answering a question needs. It closes with a diagram of a journey through the shop, with the text that says everything it shows, and a paragraph on what Itinera is not: not a job queue, not BPMN, no branching inside a workflow.

### Part 2: Getting started

3. **Installing Itinera.** The package, its options and the language version it needs.
4. **A workflow of one step.** Charge an order, run the journey and read the result: a step, a workflow, a workflow instance and its initial data, the executor, the journey and its result, and the two execution modes, synchronous and asynchronous.
5. **Adding a step.** Ship after charging. The order the steps are written in is the order they run, as the workflow's listing shows without running anything.
6. **Watching a journey.** Events: what each journey reports as it runs, and in what order. Where the language has a reporter that prints every event, the chapter attaches it and explains what it prints.

### Part 3: Writing steps

7. **Giving a step what it needs.** Inputs, required or optional, and the journey's initial data.
8. **Passing data on.** Contributions and the data bag, and why only a successful attempt's contributions are committed.
9. **Saying how it went.** Succeed; fail with a reason; fail in a way that might work next time; skip. When to use each.
10. **When something goes wrong inside a step.** Throwing or returning an error (an abnormal termination) versus failing on purpose, and why an unrecoverable failure is outside the model.
11. **Giving a step the tools it needs.** Services: a payment client, a stock store or a carrier, which a step asks for by a key and uses without knowing which implementation it got. The workflow supplies each through a provider, which decides how long an instance lasts: one shared by the whole journey, such as the stock store used by both reserving and shipping, or a fresh one for every attempt. A provider can also serve only some steps. The step is lent the service for its attempt only.

### Part 4: Controlling the flow

12. **Trying again.** Retry budgets, and which failures are retried. Shipping is tried twice, and the carrier's provider gives the second attempt the backup carrier, so the ship step never knows there are two.
13. **Repairing, then trying once more.** Rescue steps: when the charge fails because the card has expired, its rescue step reads why the charge failed, asks the customer's bank for the new card and contributes it, and the charge runs once more. A rescued step is never retried, since the rescue takes the place of retrying. If the rescue fails, the journey fails and the charge never runs again; if the attempt after the rescue fails, the journey fails too. The events show the step being rescued and how the rescue went. Declaring a retry budget above 0 on a step that has a rescue step is refused before any journey, and a rescue step cannot skip: doing so aborts the journey. A rescue step inherits nothing from the step it rescues: an input adapter or service provider it needs is attached to its own name, such as `charge_rescue`, and the listing shows it in a rescue section of its step's entry.
14. **Reacting to what a step did.** Step policies and their hooks, for example auditing every successful charge.
15. **Ending a journey early.** Finishing the journey once the charge step has contributed that the order is digital, so nothing is left to ship; giving up at once, instead of rescuing the charge, when the payment network is down and the charge ends in an abnormal termination: the `on step abnormal termination` hook of a step policy fails the journey.
16. **Doing something when the journey ends.** Workflow policies, for example telling the customer.
17. **Letting policies act on the outside world.** Roles: the shop provides "tell the customer", and the policy uses it without knowing how it is done.
18. **Deciding where a step's data comes from.** Input adapters, for example charging the order total less the discount already in the data bag, without the step knowing where the amount came from.
19. **Naming your journeys.** Journey IDs, and generating them from the order.

### Part 5: Running it for real

20. **Running many journeys.** Workflow instances and the executor, giving each journey its initial data, and what it means when `run` refuses an instance before any journey starts.
21. **Reading the result.** Succeeded, failed or aborted: what each means for your users, and what each carries.
22. **Writing your own reporter.** A reporter that writes every journey to the shop's log, and what happens when a reporter fails.
23. **Labelling a journey's log.** Global metadata that labels every line of a journey's log, such as the customer and the tracking number: given when the instance is built, and set by a step or a policy as the journey runs.
24. **Testing a step.** Testing a step, a policy's hook or an input adapter alone, by calling it directly with the recorders the language offers, then checking what it reported and contributed.
25. **Testing the flow without the steps.** Running the real workflow with each step's attempts scripted, so a test checks the retries, rescues and policies' decisions without the business rules.
26. **Testing a whole journey.** Running the real steps against fake services, and pinning the paths of a few orders, approved by a person, so a flow change that alters them cannot pass unnoticed.

### Part 6: Designing with Itinera

27. **Step, rescue step or policy?** A short guide to deciding where a decision or a repair belongs: a table of the shop's decisions and where each lives, and a paragraph for each kind of place.
28. **From tangled code to a workflow.** Adopting Itinera in a system you already have: one route at a time, starting with the checkout of chapter 1, as a short list of moves, each leaving a program that runs. Its business rules become steps, the services it already calls are wrapped and lent to the steps through providers, and its flow decisions move into the workflow and its policies.
29. **Mistakes caught before a journey.** Violations found at admission, refusals, and the rules this language makes impossible to express, each with the real message and how to fix it.
30. **Declaring by hand.** What the language's shorthand expands to, and when the long form is needed.
31. **Using a step in another workflow.** The shop starts letting customers collect their orders at the counter. A second workflow lists the same reserve and charge steps and the same audit policy, unchanged, with its own flow (no rescue step, the order put aside instead of shipped), its own services (the back room's stock and the counter's card terminal) and its own notify policy. Both listings, and two orders run through each, show that a step is written once and the workflow decides everything around it.
32. **Working with agents.** What an agent needs to change a step (its inputs, its contributions, its outcomes and the services it requests) or the flow (one workflow and its policies), and why neither change needs the other. The one rule a reviewer, human or agent, checks on every change: a step decides nothing about the flow. The approved paths of chapter 26 as the guard on flow changes. A paragraph for a team's `AGENTS.md` that says so, ready to copy. The chapter stays practical and claims nothing that has not been measured.

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
3. for chapter 29, compile-fail cases with their recorded compiler messages, where the language turns mistakes into compile errors.

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
