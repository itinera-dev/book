// Checks every pair of colours the pages use against WCAG 2.1 AA, in both
// colour schemes, reading the colours from src/styles/tokens.css so the check
// and the site cannot drift apart. Contrast is arithmetic: it is asserted here,
// not judged by eye. Borrowed, with its thresholds, from the author's own site.
//
// The highlighted code is not in this list: its colours come from the
// highlighter's themes, and the axe sweep checks them on the built pages.

import { readFileSync } from 'node:fs';

const AA_TEXT = 4.5;
const AA_NON_TEXT = 3;

const css = readFileSync(new URL('../src/styles/tokens.css', import.meta.url), 'utf8');
const layout = readFileSync(new URL('../src/layouts/Base.astro', import.meta.url), 'utf8');

/** The `--name: #hex;` declarations of one block of the file. */
function tokens(block) {
	return Object.fromEntries([...block.matchAll(/--([a-z-]+):\s*(#[0-9a-f]{6})\s*;/gi)].map(([, name, value]) => [name, value]));
}

const dark = css.indexOf('@media (prefers-color-scheme: dark)');
if (dark === -1) throw new Error('tokens.css has no dark scheme.');
const schemes = { light: tokens(css.slice(0, dark)), dark: tokens(css.slice(dark)) };

const channel = (value) => {
	const c = value / 255;
	return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
};
const luminance = (hex) => {
	const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
	return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
};
const ratio = (a, b) => {
	const [light, darker] = [luminance(a), luminance(b)].sort((x, y) => y - x);
	return (light + 0.05) / (darker + 0.05);
};

// [what, foreground, background, threshold]
const CHECKS = [
	['body text on the page', 'ink-body', 'paper', AA_TEXT],
	['headings on the page', 'ink', 'paper', AA_TEXT],
	['muted text (captions, labels) on the page', 'ink-muted', 'paper', AA_TEXT],
	['links on the page', 'accent', 'paper', AA_TEXT],
	['link hover on the page', 'accent-strong', 'paper', AA_TEXT],
	['body text in a note', 'ink-body', 'paper-raised', AA_TEXT],
	['note title', 'ink', 'paper-raised', AA_TEXT],
	['links in a note', 'accent', 'paper-raised', AA_TEXT],
	['inline code', 'ink-body', 'paper-raised', AA_TEXT],
	['inline code in a note', 'ink-body', 'paper', AA_TEXT],
	['muted text in a note', 'ink-muted', 'paper-raised', AA_TEXT],
	['the current edition (page colour on ink)', 'paper', 'ink', AA_TEXT],
	['the "not compiled yet" and "does not compile" markers', 'caution-ink', 'caution-paper', AA_TEXT],
	['skip link', 'accent-strong', 'paper', AA_TEXT],
	['focus ring against the page', 'accent-strong', 'paper', AA_NON_TEXT],
	['focus ring against a note', 'accent-strong', 'paper-raised', AA_NON_TEXT],
	['focus ring against a code block', 'accent-strong', 'code-paper', AA_NON_TEXT],
	['edition switcher border against the page', 'rule-strong', 'paper', AA_NON_TEXT],
	['border of code that does not run, against the page', 'caution-ink', 'paper', AA_NON_TEXT],
];

let failures = 0;
for (const [scheme, colours] of Object.entries(schemes)) {
	console.log(`\n${scheme} scheme`);
	for (const [what, fg, bg, threshold] of CHECKS) {
		if (!colours[fg] || !colours[bg]) {
			failures += 1;
			console.log(`  FAIL  --${colours[fg] ? bg : fg} is not defined  ${what}`);
			continue;
		}
		const value = ratio(colours[fg], colours[bg]);
		const ok = value >= threshold;
		if (!ok) failures += 1;
		console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${value.toFixed(2)}:1 (needs ${threshold}:1)  ${what}`);
	}
}

// The browser's own furniture is painted in the page colour each scheme names
// in the layout's theme-color, which must be the page colour of tokens.css.
console.log('\ntheme-color');
for (const [scheme, colours] of Object.entries(schemes)) {
	const declared = new RegExp(`name="theme-color" content="(#[0-9a-f]{6})" media="\\(prefers-color-scheme: ${scheme}\\)"`, 'i').exec(layout)?.[1];
	const ok = declared?.toLowerCase() === colours.paper?.toLowerCase();
	if (!ok) failures += 1;
	console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${scheme}: Base.astro says ${declared ?? 'nothing'}, tokens.css says ${colours.paper}`);
}

console.log(failures === 0 ? '\nAll contrast checks passed.' : `\n${failures} contrast check(s) failed.`);
process.exit(failures === 0 ? 0 : 1);
