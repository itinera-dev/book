// @ts-check
// What this build renders, read in one place.
//
// The book has one edition per programming language. A build renders one
// edition, at a path made of the edition's language and an optional version
// segment: /book/rust/ by default, /book/rust/next/ or /book/rust/0.1.0/ when
// BOOK_VERSION says so. Every link, the edition switcher and the contents derive
// from the values exported here, so nothing else in the site builds a path by
// hand.
//
// Adding a language is one entry in `editions`.

import { resolve } from 'node:path';

/**
 * @typedef {object} Edition
 * @property {string} language The language's identifier, used in URLs: `rust`.
 * @property {string} name The language's name, as a reader says it: `Rust`.
 * @property {string} directory The language's directory in the repository, holding
 *   `examples/`, `expected/`, `tested/`, `notes/`, `compile-errors/` and `pending/`.
 * @property {string} extension The extension of its programs, without the dot.
 * @property {string} highlight The highlighter's name for the language.
 * @property {boolean} asynchronous Whether the language's executors have an
 *   asynchronous execution mode. Without one, every `asynchronous` key of a
 *   directive is ignored and only the synchronous form is shown.
 */

/** @type {readonly Edition[]} */
export const editions = [
	{ language: 'rust', name: 'Rust', directory: 'rust', extension: 'rs', highlight: 'rust', asynchronous: true },
];

/** The directories of a language's directory that the book reads. */
export const editionDirectories = ['chapters', 'examples', 'expected', 'tested', 'compile-errors', 'pending'];

/** The site's origin, and the path the whole book is served under. */
export const site = 'https://itinera-dev.org';
export const base = '/book';

/** The repository on GitHub, for links to files that are not pages of the book. */
export const repository = 'https://github.com/itinera-dev/book';

/** The checkout of this repository: the directory holding each language's directory. */
export const root = resolve(process.env.BOOK_ROOT ?? resolve(process.cwd(), '..'));

const SEGMENT = /^[a-z0-9][a-z0-9.-]*$/;

/** The version segment of the edition built from the `next` branch, ahead of any release. */
const NEXT = 'next';

/**
 * The edition this build renders: its language, from BOOK_LANGUAGE, and an
 * optional version segment, from BOOK_VERSION.
 */
function configured() {
	const language = process.env.BOOK_LANGUAGE || editions[0].language;
	const version = process.env.BOOK_VERSION || '';
	const edition = editions.find((candidate) => candidate.language === language);
	if (!edition) {
		throw new Error(
			`BOOK_LANGUAGE is "${language}", which is not an edition. Known: ${editions.map((e) => e.language).join(', ')}.`,
		);
	}
	if (version !== '' && !SEGMENT.test(version)) {
		throw new Error(
			`BOOK_VERSION is "${version}". It must be one path segment of lower-case letters, digits, dots and hyphens, such as "next" or "0.1.0".`,
		);
	}
	return {
		edition,
		version,
		/**
		 * Whether this build is a released edition: one whose version names a
		 * release, not `next`. A released edition has no pending examples.
		 */
		release: version !== '' && version !== NEXT,
		/** The edition's path under the base, without slashes at either end: `rust` or `rust/next`. */
		segment: version === '' ? edition.language : `${edition.language}/${version}`,
		/** The git ref links to repository files point at. */
		sourceRef: process.env.BOOK_SOURCE_REF || 'main',
	};
}

export const build = configured();

/** The directory of the chapters this build renders, relative to the repository: `rust/chapters`. */
export const chapters = `${build.edition.directory}/chapters`;

/**
 * The URL of a page of the edition this build renders.
 *
 * @param {string} [page] A chapter's file name without `.md`; omitted for the contents.
 */
export function pageHref(page) {
	return page ? `${base}/${build.segment}/${page}/` : `${base}/${build.segment}/`;
}

/**
 * The URL of a page's Markdown copy, for agents and other readers of plain text.
 *
 * @param {string} [page] A chapter's file name without `.md`; omitted for the contents.
 */
export function markdownHref(page) {
	return `${base}/${build.segment}/${page ?? 'index'}.md`;
}

/**
 * The URL of a page in another edition. Another language is linked at its own
 * latest path, since its versions are not this build's to know.
 *
 * @param {Edition} edition
 * @param {string} [page]
 */
export function editionHref(edition, page) {
	if (edition.language === build.edition.language) return pageHref(page);
	return page ? `${base}/${edition.language}/${page}/` : `${base}/${edition.language}/`;
}

/**
 * A file of this repository on GitHub.
 *
 * @param {string} path The path inside the repository, such as `rust/examples/ch06_one_step.rs`.
 */
export function sourceHref(path) {
	const kind = path === '' || path.endsWith('/') ? 'tree' : 'blob';
	return `${repository}/${kind}/${build.sourceRef}/${path}`;
}
