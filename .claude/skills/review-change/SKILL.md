---
name: review-change
description: How Claude has a change to the Itinera book reviewed by a separate reviewer agent, and the checklist that reviewer works through, starting with ideas used before the chapter that teaches them. Use before a change is opened as a pull request or pushed, after every later push, when a reviewer's findings come back, and to audit the whole book.
---

# Reviewing a change to the book

[AGENTS.md](../../../AGENTS.md) and [BOOK.md](../../../BOOK.md) are the rulebook, and [FORMAT.md](../../../FORMAT.md) says how pages and examples are laid out. This skill adds only how Claude runs the review. It states no rule of its own: every item below points at the bullet or section that is its criterion.

## Starting the reviewer

Start a separate agent with the most capable model available. Its prompt says:

1. It is read-only. It edits, commits and pushes nothing, and it never switches branches in the shared checkout. To build a revision, it adds a `git worktree` under the session's scratchpad directory, or under `/tmp` when there is none, and removes the worktree afterwards. It never runs the checks in the shared checkout, where someone else's uncommitted work may be.
2. Which commits to review, as revisions it can diff, what the change is meant to do, and the draft commit message or pull request description.
3. That AGENTS.md, BOOK.md and FORMAT.md are binding. Behaviour is checked against the specification on the `main` branch of itinera-dev/spec, fetched fresh since a local checkout may be old, and against the language's package at the revision the language's directory pins, such as the `rev` in `rust/Cargo.toml`.
4. The checklist below, in full.
5. The report it returns: findings marked BLOCKING or NIT, each with `file:line` and a concrete fix, then a verdict, PASS or CHANGES NEEDED.

When findings have been fixed, resume the same reviewer with what changed, so that it keeps its context.

## The checklist

For each item, the reviewer lists every place the item applies, not only the first, and checks each against the section named in quotes.

**Each idea in its own chapter**

1. **The ledger.** For every page the change touches, list each idea that appears in a region the page quotes or in its prose: every concept of the specification, attribute, derive, trait, type, method, feature, event and violation. Next to each, write the chapter that teaches it, from BOOK.md's "Contents". Flag every idea whose chapter comes later, against item 1 of "Each idea in its own chapter". A later idea named in one sentence with a link and no syntax passes. A sentence that explains it, or shows its syntax, does not.
2. **Unshown code.** For every function, type or reporter outside the quoted regions, check whether the prose explains it, describes its output in detail, or depends on it to make sense, against item 2. If it does, it is shown and taught, or the prose stops relying on it.
3. **Paths from data.** List every path the prose talks about: each success, failure, skip, retry, rescue, abort and refusal. For each, name the order or input whose data takes it there and the lines of the recorded output that show it, against item 3. Flag any path decided by hidden state, such as a counter, a flag that flips or a fake that fails only on its first call. Flag any sentence that invites the reader to change a value to see another path; search for "Try" and "try".
4. **The ripple.** For every idea, region, output line or sentence the change removes or moves, search every later page for text that relied on it, against item 5. That includes "as in chapter N" and "the same as" sentences, other pages quoting the changed example's regions, the "Without Itinera" sections, the quick start, the cheat sheet, the glossary and the site. Name each page checked.

**Where behaviour comes from**

5. Every statement about what Itinera does, against "Where behaviour comes from" in AGENTS.md and item 3 of "Voice" in BOOK.md. Check it against the specification and against the pinned package's code and documentation. A gap or contradiction is reported, never decided in the book.
6. Every changed chapter, against its own entry in BOOK.md's "Contents", and against hard rule 1, "BOOK.md comes first": a chapter that no longer does what its entry says needs BOOK.md changed first.

**What the pages show**

7. Every page's prose against its recorded output, line by line, ignoring what changes between runs such as timestamps and journey IDs. That covers each "How it works" item, each event walk-through and each sentence that says what the program prints.
8. Every piece of code shown, against hard rule 3, "Every piece of code shown is checked in CI", and FORMAT.md: it is a marked region of a complete program, and output is recorded. Every comment and doc comment in a quoted region describes what is there.
9. Every line of every example the change touches: a line that does nothing, an import or helper nothing uses, or an anchor no page quotes, is removed.
10. Every word of the specification's vocabulary, against "Vocabulary" in AGENTS.md and item 2 of "Voice": used exactly, explained the first time the book uses it, and event names written one way within a chapter.

**Links and order**

11. Every link and every mention of another chapter, part or section: it points to the page that really teaches or shows what the sentence claims, its anchor exists on the built site, and a chapter mentioned by number is linked.
12. The cheat sheet, the glossary and the contents, when the change touches what they list, against their entries in BOOK.md's "Appendices": rows in the order the book teaches them, with the right chapter, naming only what some chapter teaches.

**Writing**

13. Every page, against "Voice" and "The shape of a chapter" in BOOK.md, and against hard rule 5, "Screen-reader friendly writing": the four sections in order, headings that a screen reader's list of headings makes sense of, and every diagram followed by text that says everything it shows.
14. The commit message or pull request description, against "Commits and pull requests" in AGENTS.md.

**Checks**

15. In the worktree at the reviewed revision, run each language's examples check, such as `cargo test --locked` in `rust/`, then `npm run -s build:strict` and `npm run -s pages` in `site/`. Report any that fail.

## Handling the findings

1. Verify each finding before acting on it. Fix the real ones, including nits that are plainly right.
2. A finding you reject goes back to the reviewer with your reason. A rejected BLOCKING finding is closed only when the resumed reviewer concedes it, or when the maintainer decides.
3. Run the checks, commit, and resume the reviewer with what changed. The change goes out only once the review passes.
4. When you tell the maintainer a change has landed, mention the review.

## Auditing the whole book

After a change to BOOK.md's order of chapters, a release of a language's package, or whenever the maintainer asks, run items 1 to 4 of the checklist over every page of a language's book, in reading order: the welcome pages, then each part with its "Without Itinera" section, then the appendices. The auditor writes its findings to a file, page by page, each with a severity and a fix, and the fixes are then made and reviewed as any other change.
