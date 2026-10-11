// Runs axe over every built page, and the 404 page, in the light and the dark
// colour scheme, and fails on any violation. Borrowed from the author's own
// site: each page is loaded once and checked in both schemes on that load,
// several pages at a time, and the results are printed in page order at the
// end so the log can be followed, by ear too.
//
// It is also the check of the highlighted code's colours, which come from the
// highlighter's themes rather than from tokens.css.

import AxeBuilderImport from '@axe-core/playwright';
import { chromium } from 'playwright';

import { base } from '../book.config.mjs';
import { findPages, serve } from './serve.mjs';

const AxeBuilder = AxeBuilderImport.default ?? AxeBuilderImport;
const WORKERS = 4;

const pages = [...(await findPages()), `${base}/404.html`];
const { origin, close } = await serve();
const browser = await chromium.launch();
const results = new Array(pages.length);
let next = 0;

async function worker() {
	const context = await browser.newContext();
	const page = await context.newPage();
	for (let index = next++; index < pages.length; index = next++) {
		await page.goto(origin + pages[index], { waitUntil: 'load' });
		const issues = [];
		for (const scheme of ['light', 'dark']) {
			await page.emulateMedia({ colorScheme: scheme });
			const { violations } = await new AxeBuilder({ page })
				.withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa', 'best-practice'])
				.analyze();
			for (const violation of violations) issues.push({ scheme, violation });
		}
		results[index] = issues;
	}
	await context.close();
}

console.log(`\nChecking ${pages.length} pages in light and dark, ${WORKERS} at a time.\n`);
await Promise.all(Array.from({ length: WORKERS }, worker));
await browser.close();
await close();

let count = 0;
for (const [index, path] of pages.entries()) {
	const issues = results[index];
	if (issues.length === 0) {
		console.log(`  PASS  ${path}`);
		continue;
	}
	count += issues.length;
	console.log(`  FAIL  ${path}`);
	for (const { scheme, violation } of issues) {
		console.log(`        [${violation.impact}] ${violation.id} (${scheme}): ${violation.help}`);
		for (const node of violation.nodes.slice(0, 3)) console.log(`          ${node.target.join(' ')}`);
	}
}

console.log(
	count === 0
		? `\nNo accessibility violations across ${pages.length} pages in both colour schemes.`
		: `\n${count} accessibility violation(s).`,
);
process.exit(count === 0 ? 0 : 1);
