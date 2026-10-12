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

1. **The ledger.** For every page the change touches, list each idea that appears in a region the page quotes or in its prose: every concept of the specification, and every attribute, derive, trait, type, method, macro, feature, event, violation, and item of the language or its libraries that the book explains rather than assumes. Next to each, write the chapter that teaches it, found as item 2 of BOOK.md's "Each idea in its own chapter" says. Then check the page against item 1 of that section. From chapter 4 on, on every page, including the "Without Itinera" sections of Parts 2 to 5 and the appendices, an idea whose chapter comes later, or is one of chapters 1 to 3 only, is flagged, unless it is named in one sentence with a link and no syntax. The welcome pages, chapters 1 to 3 and Part 1's "Without Itinera" section are checked against the exception item 1 gives them.
2. **Unshown code.** For every function, type or reporter outside the quoted regions, check whether the prose explains it, describes its output in detail, or depends on it to make sense, against item 3. If it does, it is shown and taught, or the prose stops relying on it.
3. **Paths from data.** List every path the prose or a diagram talks about: each success, failure, skip, retry, rescue, abort and refusal. For each, name the order or input whose data takes it there and the lines of the recorded output that show it, against item 4. Flag any path that hidden state decides, such as a counter, a flag that flips or a fake that fails only on its first call, unless the page's point is a failure that passes and the prose says so plainly. Flag any branch that the prose or a diagram talks about and that no order takes, and any sentence that invites the reader to change a value to see another path; search for "Try" and "try".
4. **Taught once.** For every idea a page uses that an earlier chapter from chapter 4 on teaches, check that the page links back rather than teaching it again, against item 5.
5. **The ripple.** For every idea, region, output line or sentence the change removes, moves or changes, search every other page, before and after it, for text that relied on it, against item 6. That includes "as in chapter N" and "the same as" sentences, other pages quoting the changed example's regions, the "Without Itinera" sections, the welcome pages, the cheat sheet, the glossary and the site. Name each page checked.

### Where behaviour comes from

6. Every statement about what Itinera does, against "Where behaviour comes from" in AGENTS.md and item 3 of "Voice" in BOOK.md. Check it against the specification and against the pinned package's code and documentation. A gap or contradiction is reported, never decided in the book.
7. Every changed page, against its own entry in BOOK.md's "Contents", and against hard rule 1 of AGENTS.md, "BOOK.md comes first": a page that no longer does what its entry says needs BOOK.md changed first.

### What the pages show

8. Every page's prose against its recorded output, line by line, against hard rule 3 of AGENTS.md, "Every piece of code shown is checked in CI", and item 4 of "Each idea in its own chapter". Ignore what changes between runs, such as timestamps and journey IDs. Check each "How it works" item, each event walk-through and each sentence that says what the program prints.
9. Every piece of code shown, against hard rule 3 and FORMAT.md items 8 and 9 of "Each language's book": it is a marked region of a complete program, and its output is recorded.
10. Every example the change touches, against FORMAT.md item 2 of "Each language": its comments describe the code as it is, and it holds no line, import, helper or region marker that nothing uses.
11. Every word of the specification's vocabulary, against "Vocabulary" in AGENTS.md and item 2 of "Voice" in BOOK.md: used exactly, and explained the first time the book uses it.

### Links and order

12. Every link and every mention of another chapter, part or section, against item 1 of "Each idea in its own chapter" and FORMAT.md's "Web pages": it points to the page that really teaches or shows what the sentence claims, and its anchor exists on the built site.
13. The cheat sheet and the glossary, when the change touches what they list, against their entries under "Appendices" in BOOK.md: the cheat sheet's rows in the order the book teaches them, each with the chapter that teaches it. The contents, `chapters/README.md`, when the change adds, removes, renames or moves a page, against FORMAT.md item 1 of "Each language's book".

### Writing

14. Every page, against "Voice" and "The shape of a chapter" in BOOK.md, hard rule 5 of AGENTS.md, "Screen-reader friendly writing", and FORMAT.md items 5 and 6 of "Each language's book": the four sections in order, no level 3 heading standing alone, and every diagram followed by text that says everything it shows.
15. The commit message or pull request description, against "Commits and pull requests" in AGENTS.md.

### Checks

16. In the worktree, the checks of BOOK.md's "How the examples are checked" and of each language's README: the language's examples check, such as `cargo test --locked` in `rust/`, and in `site/`, `npx playwright install chromium`, then `npm run verify`, as CI runs it. Where Playwright cannot run, run `npm run -s build:strict` and `npm run -s pages`, and say which checks were skipped. Report any that fail.

## Handling the findings

1. Verify each finding before acting on it. Fix the real ones, including nits that are plainly right.
2. A finding you reject goes back to the reviewer with your reason. A rejected BLOCKING finding is closed only when the resumed reviewer concedes it, or when the maintainer decides.
3. Run the checks, commit, and resume the reviewer with what changed. The change goes out only once the review passes.
4. When you tell the maintainer a change has landed, mention the review.

## Auditing the whole book

After a change to BOOK.md's order of chapters, a release of a language's package, or whenever the maintainer asks, run items 1 to 5, 7, 8, 10 and 12 to 14 of the checklist over every page of a language's book, in reading order: the welcome pages, then each part with its "Without Itinera" section, then the appendices. The auditor writes its findings to a file, page by page, each with a severity and a fix, and the fixes are then made and reviewed as any other change.
