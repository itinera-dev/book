// Checks the built site for what a reader would find missing: a page of the
// book that was not built, a page with nothing in it, and a link or image that
// leads nowhere.
//
// The pages that must exist are worked out here from the repository, not from
// the site's own code, so a chapter that the contents forgot is caught too:
// the front page, the 404 page, and for the edition this build renders its
// contents and one page for every file of its chapters/ but the README.

import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { join } from 'node:path';

import { base, build, chapters, editions, root } from '../book.config.mjs';
import { DIST, fileFor, findPages } from './serve.mjs';

const problems = [];

const edition = `${base}/${build.segment}/`;
const chapterPages = readdirSync(join(root, chapters))
	.filter((name) => name.endsWith('.md') && name !== 'README.md')
	.map((name) => `${edition}${name.replace(/\.md$/, '')}/`);
const expected = [`${base}/`, edition, ...chapterPages];

const built = await findPages();
for (const page of expected) {
	if (!built.includes(page)) problems.push(`${page} was not built.`);
}
if (!existsSync(join(DIST, '404.html'))) problems.push('404.html was not built.');

// Another language's edition is built by a build of its own, so a link into
// it cannot be checked from this one.
const elsewhere = editions
	.filter((other) => other.language !== build.edition.language)
	.map((other) => `${base}/${other.language}/`);

const ids = new Map();
const idsOf = (file) => {
	if (!ids.has(file)) {
		const html = readFileSync(file, 'utf8');
		ids.set(file, new Set([...html.matchAll(/\sid="([^"]+)"/g)].map(([, id]) => id)));
	}
	return ids.get(file);
};

const pages = [...built.map((page) => ({ page, file: fileFor(page) })), { page: `${base}/404.html`, file: join(DIST, '404.html') }];
let links = 0;

for (const { page, file } of pages) {
	const html = readFileSync(file, 'utf8');

	const headings = html.match(/<h1[\s>]/g)?.length ?? 0;
	if (headings !== 1) problems.push(`${page} has ${headings} level 1 headings; it must have one.`);

	const main = /<main[^>]*>([\s\S]*)<\/main>/.exec(html)?.[1] ?? '';
	if (main.replace(/<[^>]+>/g, '').trim().length < 40) problems.push(`${page} has nothing in its main landmark.`);
	if (/<article class="prose"[^>]*>\s*<\/article>/.test(html)) problems.push(`${page} has an empty chapter.`);

	for (const [, attribute, url] of html.matchAll(/\s(href|src|srcset)="([^"]*)"/g)) {
		if (url.startsWith('data:') || /^[a-z][a-z0-9+.-]*:/i.test(url) || url.startsWith('//')) continue;
		links += 1;
		const [path, fragment] = url.split('#');
		if (path === '' && fragment !== undefined) {
			if (!idsOf(file).has(fragment)) problems.push(`${page} links to #${fragment}, which is not on the page.`);
			continue;
		}
		if (!path.startsWith('/')) {
			problems.push(`${page} has the relative ${attribute} "${url}"; links in the book are absolute.`);
			continue;
		}
		if (elsewhere.some((prefix) => path.startsWith(prefix))) continue;
		const target = fileFor(path);
		if (!target || !existsSync(target)) {
			problems.push(`${page} links to ${url}, which was not built.`);
		} else if (fragment && target.endsWith('.html') && !idsOf(target).has(fragment)) {
			problems.push(`${page} links to ${url}, and there is no "${fragment}" on that page.`);
		}
	}
}

if (problems.length > 0) {
	console.error(`\n${problems.length} problem(s) with the built pages:\n`);
	for (const problem of problems) console.error(`  ${problem}`);
	process.exit(1);
}
console.log(`\nAll ${expected.length} expected pages were built, each has one h1 and a body, and all ${links} internal links resolve.`);
