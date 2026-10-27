# The book's web pages

To read the book locally, from a clone of this repository:

```sh
cd site
npm ci
npm run dev
```

Then open <http://localhost:4321/book/rust/>.

- The diagrams are drawn in the reader's browser by [Mermaid](https://mermaid.js.org), as GitHub draws them, so building or serving the site needs no browser of its own.
- It needs Node 22.12 or later.
- The dev server reloads the page when a chapter or a file of the book it renders changes: for the Rust book, anything in `../rust/chapters/`, `../rust/examples/`, `../rust/expected/`, `../rust/tested/`, `../rust/compile-errors/` or `../rust/pending/`. When the contents, `../rust/chapters/README.md`, changes, or a chapter file is added or removed, the server restarts on its own, which takes a few seconds.
- `BOOK_LANGUAGE=typescript npm run dev` serves the TypeScript book instead, at <http://localhost:4321/book/typescript/>, reading `../typescript/` the same way.

## What this is

An [Astro](https://astro.build) site that renders the book for the web, published by GitHub Pages at <https://itinera-dev.org/book/>, with each language's editions under it: for each language, such as Rust, the latest release at `/book/rust/`, `next` at `/book/rust/next/` and each release at `/book/rust/VERSION/`. A language with no release yet has only its `next` edition, at `/book/LANGUAGE/next/`, and every link to it leads there, as BOOK.md's "Where editions are published" says; its entry in `book.config.mjs` marks it as `released: false` until its first release. Nothing of the book is copied here. At build time the pages read each language's book from its directory, such as `../rust/`: the chapters from `../rust/chapters/`, the order of the book from `../rust/chapters/README.md`, and the code and output the directives name, as [FORMAT.md](../FORMAT.md) lays them out.

- `book.config.mjs` holds the list of editions and says which one a build renders. Every link, the edition switcher and the contents derive from it. Adding a language is one entry in its list.
- `src/lib/` renders the Markdown: `contents.ts` reads the contents, `directives.ts` replaces each directive with the code or output it names, `links.ts` turns links between chapters into links between pages, `rehype.ts` names code blocks and puts diagrams and tables in their boxes, and `render.ts` puts the pipeline together.
- `src/scripts/diagrams.ts` draws each page's diagrams in the reader's browser, in the page's colours, and again when the reader's colour scheme changes. Until it has, and for a reader without JavaScript, a diagram shows its source as text.
- `src/pages/` has the front page, the edition's contents, one page per chapter, and the 404 page.
- `scripts/` holds the checks that `npm run verify` runs.

## Building and previewing

- `npm run build` builds the site into `dist/`.
- `npm run preview` serves that build at <http://localhost:4321/book/>.

A build renders one edition. These environment variables, read only in `book.config.mjs`, say which, and from where:

- `BOOK_LANGUAGE`: the edition's language, such as `rust`. Without it, the first edition in the list.
- `BOOK_VERSION`: a version segment, such as `next` or `0.1.0`, which puts the edition at `/book/rust/VERSION/` instead of `/book/rust/`. BOOK.md says which edition is published at which address: the latest release at `/book/rust/`, each release at `/book/rust/VERSION/`, and the `next` branch at `/book/rust/next/`. A version other than `next` is a released edition, and a released edition has no pending examples: a directive that names one fails the build. Every edition but the latest release's says at the top of each page which edition it is, with a link to the latest release, and only the latest release's edition is in the sitemap.
- `BOOK_SOURCE_REF`: the git branch, tag or commit that links to files of this repository point at on GitHub, such as the program under each example. Without it, `main`.
- `BOOK_ROOT`: the checkout of this repository that the book is read from. Without it, the directory above `site/`. It lets the site render another checkout, such as a tag being prepared for release.
- `BOOK_DIST`: the directory the build writes the site into, and the checks read it from. Without it, `dist/`.

## Building every edition

`npm run editions`, run in a checkout with every branch and tag fetched, builds the whole site as GitHub Pages publishes it into `dist/`: for each language, the latest release from this checkout, the `next` branch, and every release tagged `LANGUAGE-VERSION`, such as `rust-0.1.0-beta.1`. A language with no release yet has no latest release, and an edition whose branch or tag does not hold the language's `chapters/README.md` yet is left out, with a warning. The site's code in that checkout, which is `main` when publishing, renders every edition, each from the book's files at its own branch or tag, so a fix to the site reaches every edition while each keeps its own chapters, examples and recorded output. Each edition is a strict build of its own, and then `npm run pages` checks every edition's pages in the whole site.

After every push to `main` or `next` and every tag of a release that passes the gate below, the Pages workflow runs it on `main` and publishes the result. It also runs on every pull request, so a change to the site that cannot render `next` or an older release fails there. Publishing after a push to `next` or a tag needs the repository's `github-pages` environment to allow deploying from `next` and from tags matching `*-[0-9]*`, under Settings, Environments, "Deployment branches and tags".

A directive that names a file which does not exist, a region missing from its program, or a link to a chapter that is not in the contents fails the build, with a message naming the chapter and the file.

## Verifying

`npm run verify` is the gate the Pages workflow runs on every pull request, and on every push before it builds every edition. It runs, in order, and stops at the first failure:

1. `npm run check`: Astro's type check of the site's code.
2. `npm run brand`: that the book's copies of the org site's colours, `src/styles/tokens.css`, logo, `src/lib/logo.mjs` and `src/components/Mark.astro`, and icon, `public/favicon.svg`, are the ones taken from the site at the commit `brand.json` pins. The site owns Itinera's design; to follow it, take the files again from a newer commit of itinera-dev/itinera-dev.github.io, the colours and the logo from the same paths in it, and the icon from the `/favicon.svg` it builds, and pin that commit. The book keeps the site's one meaning for amber, what the reader cannot take as it is shown: here, code that does not compile yet and a recorded compiler message.
3. `npm run contrast`: the pairs of colours the pages use, read from `src/styles/tokens.css`, against WCAG AA in the light and the dark scheme, and the layout's `theme-color` against the page colour of each scheme.
4. `npm run languages`: for every language this checkout holds, the edition it would be published as, its latest release, or its `next` edition for a language with no release yet, built from this checkout into a directory of its own, then the four checks below, so a pull request's pages are checked in every language. Each of them can also be run on its own, for the edition the variables above choose:
   1. `npm run build:strict`: a forced build, ignoring every cache, that also fails when the build printed an error.
   2. `npm run pages`: that every page exists (the front page, the 404 page, the contents, and one page for each file of the edition's `chapters/`), that each has one level 1 heading and a body, and that every internal link and image leads to something that was built.
   3. `npm run a11y`: axe on every page, in the light and the dark scheme, once its diagrams are drawn in that scheme.
   4. `npm run layout`: every page at 360, 390, 768 and 1280 pixels wide, checking that nothing scrolls sideways except code and tables, inside their own box.

The last two open the pages in Playwright's Chromium, so they need it installed once per machine, with `npx playwright install chromium`; on Linux, `npx playwright install --with-deps chromium` also installs the system libraries it needs.
