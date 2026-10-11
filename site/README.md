# The book's web pages

To read the book locally, from a clone of this repository:

```sh
cd site
npm ci
npm run dev
```

Then open <http://localhost:4321/book/rust/>. The dev server reloads the page when a chapter, or a file of the Rust edition, changes: anything in `../chapters/`, `../rust/examples/`, `../rust/expected/`, `../rust/notes/`, `../rust/compile-errors/` or `../rust/pending/`.

It needs Node 22.12 or later. The diagrams are drawn at build time in Playwright's Chromium; if Playwright has no browser on your machine, `npx playwright install chromium` installs it once.

## What this is

An [Astro](https://astro.build) site that renders the book for the web, published by GitHub Pages at <https://itinera-dev.github.io/book/>, with one edition per programming language under it: `/book/rust/` for Rust. Nothing of the book is copied here. At build time the pages read the chapters from `../chapters/`, the order of the book from `../chapters/README.md`, and each language's code, output and notes from its directory, such as `../rust/`, as [FORMAT.md](../FORMAT.md) lays them out.

- `book.config.mjs` holds the list of editions and says which one a build renders. A build renders one edition, at `/book/LANGUAGE/` or, when `BOOK_VERSION` is set, `/book/LANGUAGE/VERSION/`; `BOOK_LANGUAGE` picks the language, and defaults to `rust`. Every link, the edition switcher and the contents derive from it. Adding a language is one entry in its list.
- `src/lib/` renders the Markdown: `contents.ts` reads the contents, `directives.ts` replaces each directive with what the edition supplies, `links.ts` turns links between chapters into links between pages, and `render.ts` puts the pipeline together.
- `src/pages/` has the front page, the edition's contents, one page per chapter, and the 404 page.
- `scripts/` holds the checks that `npm run verify` runs.

## Building and previewing

- `npm run build` builds the site into `dist/`.
- `npm run preview` serves that build at <http://localhost:4321/book/>.
- `BOOK_VERSION=next npm run build` builds the same edition at `/book/rust/next/`.

A directive that names a file which does not exist, a region missing from its program, or a link to a chapter that is not in the contents fails the build, with a message naming the chapter and the file.

## Verifying

`npm run verify` is the gate the Pages workflow runs on every pull request and before every deployment. It runs, in order, and stops at the first failure:

1. `npm run check`: Astro's type check of the site's code.
2. `npm run contrast`: every pair of colours in `src/styles/tokens.css` against WCAG AA, in the light and the dark scheme.
3. `npm run build:strict`: a forced build, ignoring every cache, that also fails when the build printed an error.
4. `npm run pages`: that every page exists (the front page, the 404 page, the contents, and one page for each file of `../chapters/`), that each has one level 1 heading and a body, and that every internal link and image leads to something that was built.
5. `npm run a11y`: axe on every page, in the light and the dark scheme.
6. `npm run layout`: every page at 360, 390, 768 and 1280 pixels wide, checking that nothing scrolls sideways except code and tables, inside their own box.

The last two drive Playwright's Chromium, as the build does for the diagrams.
