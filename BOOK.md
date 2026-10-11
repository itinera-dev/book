# The Itinera book

What the Itinera book contains, and how it is written. It is one book, written for several languages: a reader picks a language and reads the whole book in it. This document is to the book what the specification is to the implementations: it is written once, without naming a language, and the book's edition in each language follows it. Each chapter and part below later becomes its own issue, so the work can be split across sessions and languages.

## Who the book is for

The book is for people building software for others with Itinera, and for the AI agents doing that work with them or for them. It is not a technical manual. The specification defines the behaviour and the API reference of each language documents every item; the book links to them for readers who want to go further, and never repeats them.

A reader finishes the book able to:

1. tell whether Itinera fits a problem, and why;
2. turn a piece of business code into steps, a workflow and policies;
3. run it, observe it and test it;
4. explain to someone else why the flow lives where it does.

## How the book is written

### Voice

1. Plain language, in the second person, aimed at a developer who knows their language but has never heard of Itinera.
2. The vocabulary of the specification's chapter 1 is used exactly, since its words are not synonyms of each other. Each word is explained in plain language the first time the book uses it, and the glossary collects them.
3. The book never describes behaviour the specification does not define. Where a language makes a rule impossible to express, the book shows it as a compile error, with the real message.
4. Everything is written for a screen reader: headings, lists, prose and simple tables. Diagrams are Mermaid, with `accTitle` and `accDescr`, always followed by text that says everything they show.

### One story: the shop

Every example belongs to one small online shop taking orders, which grows a little in each chapter:

1. it charges an order, less any discount;
2. it reserves the stock and ships the order;
3. it audits every charge;
4. it tells the customer how the order went.

Each chapter's example is a complete program that compiles, runs and shows what it prints. Every example runs on its own, so a reader can start at any chapter.

### The shape of a chapter

Every chapter has the same four sections, in this order, so a screen reader user can jump straight to the part they need and an agent knows where to look:

1. **What you want to do,** in one or two sentences, in the shop's terms.
2. **The code,** with what it prints underneath.
3. **How it works,** in prose.
4. **Remember:** a short list of rules a reader or an agent must not break, such as "a step's contributions are committed to the data bag only when its attempt succeeds".

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

### Part 1: Introduction

The introduction makes one argument: the hardest part of understanding a system that mixes business rules with flow control is knowing what is going on, and with Itinera you find out by reading one workflow. Every chapter of the part serves that argument, and no later chapter is needed to follow it.

1. **The hardest part is knowing what is going on.** The shop written the usual way: a checkout that charges, retries, falls back, ships, logs and notifies, with those decisions spread across functions and modules. The reader is asked simple questions, such as "what happens to an order when the card is declined twice?", and the chapter follows the code to answer them, showing how much has to be read, and how easily a decision is missed.
2. **Steps, policies and workflows.** The three ideas that separate the two kinds of decision. A step holds one business rule, reports what happened and knows nothing of what comes next. A policy holds the flow decisions, through hooks the executor calls after each outcome. A workflow lists the steps in the order they run, with their retry budgets and the policies attached to them. Each idea is shown with the smallest piece of the shop that makes it clear.
3. **Read the workflow, know what happens.** The shop's workflow declaration, on its own. The reader answers the questions of chapter 1 from it alone, in a few lines: the order of the steps, how many times the charge is tried, which policy decides what after a failure, what happens at the end. The chapter closes with a diagram of a journey through the shop, and the text that says everything it shows.
4. **What happens when it runs.** The remaining words, each in a paragraph: workflow instance, journey, executor, data bag, result and event, and the two execution modes, synchronous and asynchronous, introduced once here. Where Itinera fits, and what it is not: not a job queue, not BPMN, no branching inside a workflow.

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
12. **Giving a step the tools it needs.** Services: a payment client or a stock store, provided to a step from outside it, so the step uses it without knowing where it came from. Provisional until the services proposal is accepted.

Without Itinera:

1. A function that calls the next component itself, catches its timeout and decides to retry. Each is a flow decision living in business code.
2. Data threaded through parameters or shared state, half-written when something fails. With Itinera a failed attempt's contributions never reach the data bag.

### Part 4: Controlling the flow

13. **Trying again.** Retry budgets, and which failures are retried.
14. **Repairing before trying again.** Rescue steps: when the charge fails because the card has expired, a rescue step asks the customer's bank for the new card, and the charge runs once more. What happens when the rescue, or the second try, fails too. Provisional until the rescue steps proposal is accepted.
15. **Reacting to what a step did.** Step policies and their hooks, for example auditing every charge.
16. **Ending a journey early.** Finishing the journey once the charge step has contributed that the order is digital, so nothing is left to ship; giving up at once, instead of spending the retry budget, when the payment provider reports an outage.
17. **Doing something when the journey ends.** Workflow policies, for example telling the customer.
18. **Letting policies act on the outside world.** Roles: the shop provides "tell the customer", and the policy uses it without knowing how it is done.
19. **Deciding where a step's data comes from.** Input adapters, for example charging the order total less the discount already in the data bag, without the step knowing where the amount came from.
20. **Naming your journeys.** Journey IDs, and generating them from the order.

Without Itinera:

1. **Retry strategies that multiply.** One call retried in a loop, another through a decorator, a third through a library, each with its own limits and none visible from outside. With Itinera every retry budget is in the workflow's declaration, every other decision to give up follows from the step's outcome, and any earlier stop is decided by a policy, all of it visible in the events.
2. **Flow decisions in business code.** A charge function that knows shipping comes next and does not call it for digital goods. With Itinera the charge only charges, what follows is in the workflow, and whether a digital order ships is decided where it belongs: the ship step reports that it skipped, or a policy finishes the journey from what the charge contributed.
3. **A circuit breaker.** The usual breaker wrapped around a client, deciding inside the call whether the call happens. With Itinera, a policy that sees every failure and a role whose operations record failures and say whether the breaker is open. To be checked against tier 1 before it is written: a policy can stop the retries while the breaker is open, but refusing the first attempt without the step deciding it may need a feature not yet specified.
4. **Another provider on the next attempt.** The usual fallback: try the main payment provider, catch the failure, call the backup, all inside the charge. With Itinera the charge step only charges, through the payment service it is given, and which provider that service is changes with the attempt, decided outside the step. How the provider is chosen per attempt follows what the services proposal decides.
5. **Repair code inside the business rule.** The usual charge that catches an expired card, fetches the new one and tries again, all in one function. With Itinera the repair is a rescue step of its own, and the charge stays a charge.

### Part 5: Running it for real

21. **Running many journeys.** Workflow instances and the executor, giving each journey its initial data, and what it means when `run` refuses an instance before any journey starts.
22. **Reading the result.** Succeeded, failed or aborted: what each means for your users, and what each carries.
23. **Seeing what happened.** Events and reporters: logging every journey, writing your own reporter, and what happens when a reporter fails.
24. **Testing your workflows.** Testing a step alone, a policy alone, and a whole journey by its result and its events.

Without Itinera:

1. **Logging that depends on who wrote the code.** One module logs before each call, another only on errors, a third swallows the error and logs nothing; reconstructing what happened to one order means reading all three. With Itinera every journey reports the same stream of events, in the same order, and a reporter decides where it goes.
2. **Testing a flow by mocking it.** Testing the retry of a charge by mocking a client and counting calls. With Itinera the step is tested alone, the policy alone, and the journey by its events.

### Part 6: Designing with Itinera

25. **Step, rescue step or policy?** How to decide where a decision or a repair belongs, with examples from the shop and a few counter-examples.
26. **From tangled code to a workflow.** The checkout of chapter 1, rewritten step by step, gathering what each "Without Itinera" section showed.
27. **Mistakes caught before a journey.** Violations found at admission, refusals, and the rules this language makes impossible to express, each with the real message and how to fix it.
28. **Declaring by hand.** What the language's shorthand expands to, and when the long form is needed.

Part 6 has no "Without Itinera" section: chapter 26 is that comparison.

### Appendices

1. **Cheat sheet.** Every declaration the language offers, one line each, with a link to the chapter that teaches it.
2. **Glossary.** Every word of the specification's vocabulary, in plain language, linking to the specification's definition.
3. **What this version supports.** The specification version, tier and capabilities of the release the book covers, its conformance report, and what later tiers will add.

### Later tiers

Each tier adds a part, in the same style, once its proposals are accepted and the language implements them: moving a journey to another workflow (tier 2), running a workflow from inside another (tier 3), and durable runs.

## What each language provides

For every chapter, each language's edition provides:

1. the chapter's prose, following this document, with the language's own notes where it differs, such as how recoverable and unrecoverable failures work in that language;
2. the chapter's example as a complete program, with marked regions the book quotes, and its recorded output;
3. for the "Without Itinera" sections, the usual code as a complete program too;
4. for chapter 27, compile-fail cases with their recorded compiler messages, where the language turns mistakes into compile errors.

A language that lacks a feature a chapter needs shows that chapter as a short note saying the feature is not in this language yet, with a link to its implementation issue.

## How the examples are checked

1. Each language's directory in this repository depends on a pinned version of that language's Itinera package, and this repository's CI compiles every example, runs it and compares its output with the recorded one, in one job per language.
2. Each language's own repository pins a commit of this repository and runs its edition of the book on every pull request against its own code, as a required check. A change that breaks the book is caught where it is made.
3. A change that breaks the book on purpose comes as a pair of pull requests, one here and one in the language's repository. The book's pull request updates the examples and pins the commit of the language's pull request; the language's pull request then pins the commit of the book's pull request, and both go green together. Until the next release, the book stays pinned to that commit rather than to a released version.
4. After a language's release, its edition moves to the released version and is tagged, for example `rust-0.1.0-rc.1`. The site quotes only from such tags, so each published edition matches a published package.

## Open points

1. **Services and rescue steps** are being added to tier 1 by their own proposals. The chapters and comparisons that use them follow what their proposals decide, and are written once those proposals are accepted.
2. **The circuit breaker** of part 4 is checked against tier 1, services included, before it is written, including how a role can hold its state when roles carry operations and never data.
3. **Where the prose lives:** with the examples in this repository, or with the site. Keeping it here, next to the code it quotes, lets one pull request change both.
