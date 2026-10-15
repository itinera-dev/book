// Builds every published edition of the book into one site, as GitHub Pages
// serves it. For each language:
//
// - the latest release at /book/LANGUAGE/, from this checkout: `main` when
//   publishing, a pull request's own when checking it;
// - the `next` branch at /book/LANGUAGE/next/;
// - every release tagged LANGUAGE-VERSION, such as rust-0.1.0-beta.1, with a
//   version starting with a digit, as the Pages workflow's tag filter says, at
//   /book/LANGUAGE/VERSION/.
//
// This checkout's site renders all of them, each from the book's files at its
// own branch or tag, read from a worktree. Each edition is a strict build of
// its own; the latest release's build gives the site its front page, its 404
// page and its sitemap, and every other build gives only its edition's pages
// and their assets. The pages of every edition are then checked in the whole
// site, so links from one edition to another are checked too.
//
// It needs every branch and tag fetched: in CI, a checkout with its whole
// history.

import { spawnSync } from 'node:child_process';
import { cpSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { dist, editions, root } from '../book.config.mjs';

/** Runs git in this checkout and returns what it printed, or undefined when it failed. */
function git(...args) {
	const result = spawnSync('git', args, { cwd: root, encoding: 'utf8' });
	return result.status === 0 ? result.stdout.trim() : undefined;
}

/** The ref of the `next` branch: the remote's, as CI fetches it, or a local one. */
function nextRef() {
	return ['origin/next', 'next'].find((ref) => git('rev-parse', '--verify', '--quiet', `${ref}^{commit}`) !== undefined);
}

/** Every edition to publish for a language, the latest release first. */
function editionsOf(language, next) {
	const tags = (git('tag', '--list', `${language}-[0-9]*`) ?? '').split('\n').filter(Boolean);
	return [
		{ language, version: '', ref: undefined, source: 'main' },
		...(next ? [{ language, version: 'next', ref: next, source: 'next' }] : []),
		...tags.map((tag) => ({ language, version: tag.slice(language.length + 1), ref: tag, source: tag })),
	];
}

/** Runs one of these scripts for an edition, and stops everything when it fails. */
function run(script, edition, checkout, output) {
	const env = {
		...process.env,
		BOOK_LANGUAGE: edition.language,
		BOOK_VERSION: edition.version,
		BOOK_SOURCE_REF: edition.source,
		BOOK_ROOT: checkout,
		BOOK_DIST: output,
	};
	const result = spawnSync(process.execPath, [script], { stdio: 'inherit', env });
	if (result.status !== 0) {
		throw new Error(`${script} failed for the ${edition.language} edition ${edition.version || 'of the latest release'}.`);
	}
}

const next = nextRef();
if (!next) console.warn('There is no `next` branch, so no edition is built from it.');
const planned = editions.flatMap((edition) => editionsOf(edition.language, next));

const scratch = mkdtempSync(join(tmpdir(), 'itinera-book-editions-'));
const worktrees = [];

/** A checkout of the book at an edition's ref, or this one for the latest release. */
function checkoutOf(edition, index) {
	if (!edition.ref) return root;
	const path = join(scratch, `checkout-${index}`);
	if (git('worktree', 'add', '--detach', path, edition.ref) === undefined) {
		throw new Error(`Could not check out ${edition.ref}.`);
	}
	worktrees.push(path);
	return path;
}

// A run that was killed leaves its worktrees registered.
git('worktree', 'prune');

try {
	rmSync(dist, { recursive: true, force: true });
	const checkouts = planned.map(checkoutOf);
	planned.forEach((edition, index) => {
		const output = join(scratch, `site-${index}`);
		run('scripts/build-strict.mjs', edition, checkouts[index], output);
		if (edition.version === '') {
			cpSync(output, dist, { recursive: true });
		} else {
			const segment = join(edition.language, edition.version);
			cpSync(join(output, segment), join(dist, segment), { recursive: true });
			cpSync(join(output, '_astro'), join(dist, '_astro'), { recursive: true });
		}
	});
	planned.forEach((edition, index) => run('scripts/pages.mjs', edition, checkouts[index], dist));
	console.log(`\nBuilt ${planned.length} edition(s): ${planned.map((edition) => `${edition.language} ${edition.version || 'latest'}`).join(', ')}.`);
} finally {
	for (const path of worktrees) git('worktree', 'remove', '--force', path);
	rmSync(scratch, { recursive: true, force: true });
}
