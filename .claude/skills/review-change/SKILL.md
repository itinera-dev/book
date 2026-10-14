---
name: review-change
description: How Claude has a change to the Itinera book reviewed by a separate reviewer agent, and the checklist that reviewer works through, starting with ideas used before the chapter that teaches them. Use before a change is opened as a pull request or pushed, after every later push, when a reviewer's findings come back, and to audit the whole book.
---

# Reviewing a change to the book

[AGENTS.md](../../../AGENTS.md) and [BOOK.md](../../../BOOK.md) are the rulebook, and [FORMAT.md](../../../FORMAT.md) says how pages and examples are laid out. This skill adds only how Claude runs the review. It states no rule of its own: every item below points at the bullet or section that is its criterion.

## Starting the reviewer

Start a separate agent with the most capable model available. Its prompt says:

1. It is read-only. It edits, commits and pushes nothing, and it never switches branches in the shared checkout, where someone else's uncommitted work may be. It adds its own worktree with `git worktree add --detach <dir>/review <revision>`, where `<dir>` is the session's scratchpad directory, or `/tmp` when there is none. It reads every file from that worktree, or with `git show <revision>:<path>`, and runs every check there, after `npm ci` in its `site/`. It removes the worktree with `git worktree remove` when it is done.
2. Which commits to review, as revisions it can diff, what the change is meant to do, and the draft commit message or pull request description.
3. That AGENTS.md, BOOK.md and FORMAT.md are binding. Behaviour is checked against the specification on the `main` branch of itinera-dev/spec, fetched fresh since a local checkout may be old, and against the language's package at the version or revision the language's directory pins, such as the `itinera` lines of `rust/Cargo.toml`.
4. The checklist below, in full.
5. The report it returns: findings marked BLOCKING or NIT, each with `file:line` and a concrete fix, then a verdict, PASS or CHANGES NEEDED.

When findings have been fixed, resume the same reviewer with what changed, so that it keeps its context.

## The checklist

For each item, the reviewer lists every place the item applies, not only the first, and checks each against the section named in quotes.

### Each idea in its own chapter

1. **The ledger.** For every page the change touches, list each idea that appears in a region the page quotes or in its prose: every concept of the specification, and every attribute, derive, trait, type, method, macro, feature, event, violation, and item of the language or its libraries that the book explains rather than assumes. Next to each, write the chapter that teaches it, found as item 2 of BOOK.md's "Each idea in its own chapter" says. Then check the page against item 1 of that section. From chapter 3 on, on every page, including the appendices, an idea that neither the page itself nor an earlier chapter from chapter 3 on teaches is flagged, and so is every mention of or link to a later chapter, even in passing. The welcome pages and chapters 1 and 2 are checked against the exception item 1 gives them.
2. **Unshown code.** For every function, type or reporter outside the quoted regions, check whether the prose explains it, describes its output in detail, or depends on it to make sense, against item 3. If it does, it is shown and taught, or the prose stops relying on it. Then, against item 7, list everything of the shop's code that the page's prose or quoted regions, synchronous and asynchronous, name or use without showing it, such as a step under test, a policy, a service's trait, a type, a provider or a helper. For each, find the last page whose regions show it as the program still has it, and check that the page says which page that is, with a link. Flag each one it doesn't point to, a pointer to a page that does not show it (including "the rest is chapter N's" when chapter N does not show it), and anything no page shows as the program has it. Names in a workflow declaration may share one sentence of pointers; an asynchronous form may point to its synchronous page and the chapter on the two execution modes; a faked outside system needs no pointer.
3. **Paths from data.** List every path the prose or a diagram talks about: each success, failure, skip, retry, rescue, abort and refusal. For each, name the order or input whose data takes it there and the lines of the recorded output that show it, against item 4. Flag any path that hidden state decides, such as a counter, a flag that flips or a fake that fails only on its first call, unless the page's point is a failure that passes and the prose says so plainly. Flag any branch that the prose or a diagram talks about and that no order takes, and any sentence that invites the reader to change a value to see another path; search for "Try" and "try".
4. **Taught once.** For every idea a page uses that an earlier chapter from chapter 3 on teaches, check that the page links back rather than teaching it again, against item 5.
5. **The ripple.** For every idea, region, output line or sentence the change removes, moves or changes, search every other page, before and after it, for text that relied on it, against item 6. That includes "as in chapter N" and "the same as" sentences, other pages quoting the changed example's regions, the Outcomes tables of other chapters, the welcome pages, the cheat sheet, the glossary and the site. Name each page checked.

### Where behaviour comes from

6. Every statement about what Itinera does, against "Where behaviour comes from" in AGENTS.md and item 3 of "Voice" in BOOK.md. Check it against the specification and against the pinned package's code and documentation. A gap or contradiction is reported, never decided in the book.
7. Every changed page, against its own entry in BOOK.md's "Contents", and against hard rule 1 of AGENTS.md, "BOOK.md comes first": a page that no longer does what its entry says needs BOOK.md changed first.
8. **One topic, from a need.** Every changed chapter from chapter 3 on, against the first paragraph of "The shape of a chapter" and "How the chapters follow one another" in BOOK.md: its "What you want to do" starts from something the reader needs and cannot yet do with what earlier chapters taught, and the chapter teaches that one topic and nothing else. Flag each section, paragraph or region that teaches a second topic, and an idea the chapter needs that an earlier chapter should have taught first.

### What the pages show

9. Every page's prose against its recorded output, line by line, against hard rule 3 of AGENTS.md, "Every piece of code shown is checked in CI", and item 4 of "Each idea in its own chapter". Ignore what changes between runs, such as timestamps and journey IDs. Check each "How it works" item, each event walk-through and each sentence that says what the program prints.
10. Every piece of code shown, against hard rule 3 and FORMAT.md items 7 and 8 of "Each language's book": it is a marked region of a complete program, and its output is recorded.
11. Every example the change touches, against FORMAT.md item 2 of "Each language": its comments describe the code as it is, and it holds no line, import, helper or region marker that nothing uses.
12. **Test support.** Every use of what exists only for tests, such as a package option turned on for tests only, a recorder, a scripted run or a fake provider, against hard rule 7 of AGENTS.md and FORMAT.md item 5 of "Each language": it sits inside a test of a tested program, and no prose, cheat-sheet line or region outside a test presents it as available to code that ships. Confirm that the build of the tested programs without test support passed.
13. Every word of the specification's vocabulary, against "Vocabulary" in AGENTS.md and item 2 of "Voice" in BOOK.md: used exactly, and explained the first time the book uses it.

### Links and order

14. Every link and every mention of another chapter, part or section, against item 1 of "Each idea in its own chapter" and FORMAT.md's "Web pages": it points to the page that really teaches or shows what the sentence claims, and its anchor exists on the built site.
15. The cheat sheet and the glossary, when the change touches what they list, against their entries under "Appendices" in BOOK.md: the cheat sheet's rows in the order the book teaches them, each with the chapter that teaches it. The contents, `chapters/README.md`, when the change adds, removes, renames or moves a page, against FORMAT.md item 1 of "Each language's book".

### Writing

16. Every page, against "Voice" and "The shape of a chapter" in BOOK.md, hard rule 5 of AGENTS.md, "Screen-reader friendly writing", and FORMAT.md items 4 and 5 of "Each language's book": the four sections in order, rules in bold where they are said, no level 3 heading standing alone, and every diagram followed by text that says everything it shows.
17. **Outcomes.** Every changed chapter's Outcomes, against "The Outcomes table" in BOOK.md: the sentence starts "We have" and says what the chapter has shown; the table has one to three rows, with one sentence in each cell; each left cell is fair to what competent code usually does, names no library or framework, never says "always", and admits it when the usual way is perfectly reasonable; each right cell is shown by the chapter's own code, quoted or run, or for chapter 1 by chapter 2's; no row repeats a row of an earlier chapter, checked by reading every earlier chapter's table; and "outcome" is used only in the specification's sense.
18. **Length.** Every changed chapter's prose, counted without its code, output and Outcomes table, against the length in "The shape of a chapter": flag a chapter above 800 words (1,300 for chapters 1 and 2), and say which topic in it could be its own chapter.
19. The commit message or pull request description, against "Commits and pull requests" in AGENTS.md.

### Checks

20. In the worktree, the checks of BOOK.md's "How the examples are checked" and of each language's README: the language's examples check, such as `cargo test --locked` in `rust/`, and in `site/`, `npx playwright install chromium`, then `npm run verify`, as CI runs it. Where Playwright cannot run, run `npm run -s build:strict` and `npm run -s pages`, and say which checks were skipped. Report any that fail.

## Handling the findings

1. Verify each finding before acting on it. Fix the real ones, including nits that are plainly right.
2. A finding you reject goes back to the reviewer with your reason. A rejected BLOCKING finding is closed only when the resumed reviewer concedes it, or when the maintainer decides.
3. Run the checks, commit, and resume the reviewer with what changed. The change goes out only once the review passes.
4. When you tell the maintainer a change has landed, mention the review.

## Auditing the whole book

After a change to BOOK.md's order of chapters, a release of a language's package, or whenever the maintainer asks, run items 1 to 5, 7 to 9, 11, 12 and 14 to 18 of the checklist over every page of a language's book, in reading order: the welcome pages, then each part, then the appendices. The auditor writes its findings to a file, page by page, each with a severity and a fix, and the fixes are then made and reviewed as any other change.
