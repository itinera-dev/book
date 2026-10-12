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
4. it sends an order to review instead of shipping it when its charge is above the review limit the order carries;
5. it tells the customer how the order went.

Each chapter's example is a complete program that compiles, runs and shows what it prints. Every example runs on its own, so a reader can start at any chapter.

### The shape of a chapter

Every chapter, and the quick start, has the same four sections, in this order, so a screen reader user can jump straight to the part they need and an agent knows where to look:

1. **What you want to do,** in one or two sentences, in the shop's terms.
2. **The code,** with what it prints underneath.
3. **How it works,** in prose.
4. **Remember:** a short list of rules a reader or an agent must not break, such as "a step's contributions are committed to the data bag only when its attempt succeeds".

### Each idea in its own chapter

A reader meets each idea in the chapter that teaches it, and Itinera is flexible enough for every chapter to show only what the reader has learnt so far.

1. The welcome pages and chapters 1 to 3 show ideas so that their argument can be followed. From chapter 4 on, each idea is taught in its own chapter, even one those pages showed: a chapter's code and prose use only what it, or an earlier chapter from chapter 4 on, teaches. A later idea may be named in one sentence with a link to the chapter that teaches it, never with its syntax or an explanation. The quick start and chapter 3 show a whole declaration on purpose: each later idea in it is named with a link, and explained no further than the page's own point needs.
2. An idea is taught by the chapter BOOK.md gives it below. Anything else the book explains rather than assumes, such as an attribute of the language's package or a channel of its standard library, is taught by the first chapter, in reading order, that shows it and explains it.
3. Code a chapter does not show may use anything, as long as the prose never relies on it. Faked outside systems, such as the bank or a carrier, print what they do, so the output tells what happened.
4. The paths a chapter talks about come from the data its orders start with: each path is run by an order whose data takes it there, and what it prints is shown. An example takes no path that none of its orders runs. No hidden counter or flag decides a path, and the prose never asks the reader to change a value to see another path. When a page's point is a failure that passes, such as a retry in the usual code or a circuit breaker, the state that decides it is said plainly in the prose.
5. Once taught, an idea may be used again, with a link back to its chapter, and is never taught a second time. Where the language has a reporter that prints every event, chapter 4 explains it, and any later chapter whose point is the events may use it.
6. When a page changes, every other page that refers to it, quotes its code or relies on what it shows, before it or after it, is read again, and still holds.

### Synchronous and asynchronous together

Itinera's synchronous and asynchronous APIs differ little, so the book has no separate chapter on asynchronous code. Each chapter shows the synchronous form first; where the asynchronous form differs, the same example follows in its asynchronous form, with one sentence saying what changed. A language whose executors accept only one execution mode shows only that one.

### Without Itinera: the section that closes each part

Every part except the last ends with a section headed **Without Itinera**. It shows the same shop written the way such code is usually written, without Itinera, and then the Itinera version beside it, so the reader sees what changed and why it matters. Each comparison:

1. shows the usual code honestly, as a competent developer would write it, not as a straw man;
2. names what that code mixes together (a business rule, a flow decision, logging, a retry strategy);
3. shows where each of those lives with Itinera, and what the reader gains: one place to read the flow, business code that can be tested alone, observation that is the same everywhere.

The section is not called "outcomes", because outcome is a word of the specification (success, failure or skipped).

The usual code is real code in the book's language too, compiled and run like every other example, so a comparison can never claim the usual code does something it does not.

## Contents

### Welcome

Three short pages come before Part 1, unnumbered, so a reader meets Itinera, the book and a whole workflow before the argument starts.

1. **Welcome to Itinera.** Hello, and what this book is for. What we are trying to change: flow decisions buried in business code, which good habits such as clean modules and dependency injection do not remove, and systems that grow until nobody can say what one request does, which agents make worse; keeping the process apart from the logic was good practice before agents, and is a matter of safety with them. The page takes this from the site's "Why Itinera" and links to it. What Itinera is: steps, workflows, policies and the executor, each in a sentence, and one meaning across languages, defined by the specification. Where the name comes from: *itinera* is the Latin plural of *iter*, a journey, and every execution of a workflow for one order is a journey. How the book is laid out, part by part.
2. **How to read this book.** The conventions every page shares: the book for each language and the language selector of the web pages; the shop and its four customers; the shape of a chapter; code that is always a region of a complete program, and the marks a caption may carry (the asynchronous form, not compiled yet, does not compile); words with one meaning, the glossary and the cheat sheet; diagrams always followed by text; the "Without Itinera" sections; where to go further.
3. **Quick start.** A whole workflow on one page, declared the shortest way the language offers: two steps, a policy that picks the backup carrier before shipping is tried again, a reporter printing every event, and one journey. It does not explain every line; it shows where each decision lives, and points to the chapters that do.

### Part 1: Introduction

The introduction makes one argument: the hardest part of understanding a system that mixes business rules with flow control is knowing what is going on, and with Itinera you find out by reading one workflow. Every chapter of the part serves that argument, and no later chapter is needed to follow it.

1. **But why Itinera?** The shop written the usual way, before any Itinera: a checkout that charges, retries, falls back, ships, logs and notifies, with those decisions spread across functions and modules. The reader is asked simple questions, such as "what happens to an order when the card is declined twice?", and the chapter follows the code to answer them, showing how much has to be read, and how easily a decision is missed.
2. **Steps, policies and workflows.** The three ideas that separate the two kinds of decision. A step holds one business rule, reports what happened and knows nothing of what comes next. A policy holds the flow decisions, through hooks the executor calls after each outcome. A workflow lists the steps in the order they run, with their retry budgets and the policies attached to them. Each idea is shown with the smallest piece of the shop that makes it clear; the policy is the review, whose hook compares what the charge contributed with the review limit in the order's initial data.
3. **Read the workflow, know what happens.** The shop's workflow declaration, for what chapter 1's checkout does, on its own. The reader answers the questions of chapter 1 from it alone, in a few lines: the order of the steps, how many times the charge is tried, which policy decides what after a failure, what happens at the end. Then the program prints the workflow's listing, which the language makes from the declaration without running anything: the steps in order, the step policies attached to each, the services each step requests with the provider that serves it, and the charge's rescue step. A workflow explains itself, to a person or an agent, before any journey. The chapter closes with a diagram of a journey through the shop, and the text that says everything it shows.
4. **What happens when it runs.** The remaining words, each in a paragraph: workflow instance, journey, executor, data bag, result and event, and the two execution modes, synchronous and asynchronous, introduced once here. Where the language has a reporter that prints every event, the chapter's program uses it, and explains what it prints. Where Itinera fits, and what it is not: not a job queue, not BPMN, no branching inside a workflow.

Without Itinera: the comparison of this part is chapter 1 against chapter 3, so it closes with a short summary of the two side by side rather than new code.

### Part 2: Getting started

5. **Installing Itinera.** The package, its options and the language version it needs.
6. **A workflow of one step.** Charge an order, run the journey and read the result.
7. **Adding a step.** Ship after charging. The order the steps are written in is the order they run.

Without Itinera: the same two calls made directly, one after the other, and what the caller must now know about both.

### Part 3: Writing steps

8. **Giving a step what it needs.** Inputs, required or optional, and the journey's initial data.
9. **Passing data on.** Contributions and the data bag, and why only a successful attempt's contributions are committed.
10. **Saying how it went.** Succeed; fail with a reason; fail in a way that might work next time; skip. When to use each.
11. **When something goes wrong inside a step.** Throwing or returning an error (an abnormal termination) versus failing on purpose, and why an unrecoverable failure is outside the model.
12. **Giving a step the tools it needs.** Services: a payment client, a stock store or a carrier, which a step asks for by a key and uses without knowing which implementation it got. The workflow supplies each through a provider, which decides how long an instance lasts: one shared by the whole journey, such as the stock store used by both reserving and shipping, or a fresh one for every attempt. A provider can also serve only some steps. The step is lent the service for its attempt only.

Without Itinera:

1. A function that calls the next component itself, catches its timeout and decides to retry. Each is a flow decision living in business code.
2. Data threaded through parameters or shared state, half-written when something fails. With Itinera a failed attempt's contributions never reach the data bag.

### Part 4: Controlling the flow

13. **Trying again.** Retry budgets, and which failures are retried. Shipping is tried twice, and the carrier's provider gives the second attempt the backup carrier, so the ship step never knows there are two.
14. **Repairing, then trying once more.** Rescue steps: when the charge fails because the card has expired, its rescue step reads why the charge failed, asks the customer's bank for the new card and contributes it, and the charge runs once more. A rescued step is never retried, since the rescue takes the place of retrying. If the rescue fails, the journey fails and the charge never runs again; if the attempt after the rescue fails, the journey fails too. The events show the step being rescued and how the rescue went. Declaring a retry budget above 0 on a step that has a rescue step is refused before any journey, and a rescue step cannot skip: doing so aborts the journey. A rescue step inherits nothing from the step it rescues: an input adapter or service provider it needs is attached to its own name, such as `charge_rescue`, and the listing shows it in a rescue section of its step's entry.
15. **Reacting to what a step did.** Step policies and their hooks, for example auditing every successful charge.
16. **Ending a journey early.** Finishing the journey once the charge step has contributed that the order is digital, so nothing is left to ship; giving up at once, instead of rescuing the charge, when the payment network is down and the charge ends in an abnormal termination: the `on step abnormal termination` hook of a step policy fails the journey.
17. **Doing something when the journey ends.** Workflow policies, for example telling the customer.
18. **Letting policies act on the outside world.** Roles: the shop provides "tell the customer", and the policy uses it without knowing how it is done.
19. **Deciding where a step's data comes from.** Input adapters, for example charging the order total less the discount already in the data bag, without the step knowing where the amount came from.
20. **Naming your journeys.** Journey IDs, and generating them from the order.

Without Itinera:

1. **Retry strategies that multiply.** One call retried in a loop, another through a decorator, a third through a library, each with its own limits and none visible from outside. With Itinera every retry budget is in the workflow's declaration, every other decision to give up follows from the step's outcome, and any earlier stop is decided by a policy, all of it visible in the events.
2. **Flow decisions in business code.** A charge function that knows shipping comes next and does not call it for digital goods. With Itinera the charge only charges, what follows is in the workflow, and whether a digital order ships is decided where it belongs: the ship step reports that it skipped, or a policy finishes the journey from what the charge contributed.
3. **A circuit breaker.** The usual breaker wrapped around a client, deciding inside the call whether the call happens. With Itinera, a policy sees every failure and records it through a role, and the carrier's provider, which belongs to the workflow, hands out the backup carrier while the breaker is open. The step only ships.
4. **Another carrier on the next attempt.** The usual fallback: try the main carrier, catch the failure, call the backup, all inside the shipping code. With Itinera the ship step only ships, through the carrier service it is given. The provider makes a fresh service for every attempt and picks the carrier from the attempt number, or from a choice that the `on step retry` hook of a step policy contributed, which the provider reads as data from the workflow.
5. **Repair code inside the business rule.** The usual charge that catches an expired card, fetches the new one and tries again, all in one function. With Itinera the repair is a rescue step of its own, the charge stays a charge, and the events say when a rescue happened and how it went.

### Part 5: Running it for real

21. **Running many journeys.** Workflow instances and the executor, giving each journey its initial data, and what it means when `run` refuses an instance before any journey starts.
22. **Reading the result.** Succeeded, failed or aborted: what each means for your users, and what each carries.
23. **Seeing what happened.** Events and reporters: logging every journey, writing your own reporter, and what happens when a reporter fails.
24. **Testing your workflows.** Testing a step alone, a policy alone, and a whole journey by its result and its events.

Without Itinera:

1. **Logging that depends on who wrote the code.** One module logs before each call, another only on errors, a third swallows the error and logs nothing; reconstructing what happened to one order means reading all three. With Itinera every journey reports the same stream of events, in the same order, and a reporter decides where it goes.
2. **Testing a flow by mocking it.** Testing the fallback of shipping by mocking two carriers and counting calls. With Itinera the step is tested alone, the policy alone, and the journey by its events.

### Part 6: Designing with Itinera

25. **Step, rescue step or policy?** How to decide where a decision or a repair belongs, with examples from the shop and a few counter-examples.
26. **From tangled code to a workflow.** Adopting Itinera in a system you already have: one route at a time, starting with the checkout of chapter 1. Its business rules become steps, the services it already calls are wrapped and lent to the steps through providers, and its flow decisions move into the workflow and its policies, one move at a time, each leaving a program that runs. It gathers what each "Without Itinera" section showed.
27. **Mistakes caught before a journey.** Violations found at admission, refusals, and the rules this language makes impossible to express, each with the real message and how to fix it.
28. **Declaring by hand.** What the language's shorthand expands to, and when the long form is needed.
29. **Using a step in another workflow.** The shop starts letting customers collect their orders at the counter. A second workflow lists the same reserve and charge steps and the same audit policy, unchanged, with its own flow (no rescue step, the order put aside instead of shipped), its own services (the back room's stock and the counter's card terminal) and its own notify policy. Both listings, and two orders run through each, show that a step is written once and the workflow decides everything around it.
30. **Working with agents.** What an agent needs to change a step (its inputs, its contributions, its outcomes and the services it requests) or the flow (one workflow and its policies), and why neither change needs the other. The one rule a reviewer, human or agent, checks on every change: a step decides nothing about the flow. Tests that pin the paths of a few orders, the outcomes and decisions their journeys go through, approved by a person, so a flow change that alters them cannot pass unnoticed. A paragraph for a team's `AGENTS.md` that says so, ready to copy. The chapter stays practical and claims nothing that has not been measured.

Part 6 has no "Without Itinera" section: chapter 26 is that comparison.

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
3. for the "Without Itinera" sections, the usual code as a complete program too;
4. for chapter 27, compile-fail cases with their recorded compiler messages, where the language turns mistakes into compile errors.

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
2. A published page keeps its address in every later edition. A new chapter goes where it renumbers no published page, and a page that is moved or renamed leaves a redirect at its old address.

## Open points

1. **Where the prose lives:** with the examples, in each language's directory of this repository, so one pull request changes both.
