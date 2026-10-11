// Checks every built page in a real browser at a phone's width and a laptop's,
// and fails when the page scrolls sideways, when the text column runs off the
// screen, or when a code block, a table or a diagram is wider than the
// screen instead of scrolling inside its own box.

import { chromium } from 'playwright';

import { diagramsDrawn, findPages, serve } from './serve.mjs';

const WIDTHS = [360, 390, 768, 1280];
const TOLERANCE = 1;

function measure() {
	const root = document.documentElement;
	const viewport = root.clientWidth;
	const main = document.querySelector('main')?.getBoundingClientRect();
	const wide = [...document.querySelectorAll('pre, .table-scroll, .diagram__scroll, .diagram:not(.diagram--wide) svg, .listing')]
		.map((element) => ({ element, box: element.getBoundingClientRect() }))
		.filter(({ box }) => box.right > viewport + 1 || box.left < -1)
		.map(({ element }) => element.getAttribute('aria-labelledby') ?? element.className ?? element.tagName.toLowerCase());
	return { viewport, scrollWidth: root.scrollWidth, mainLeft: main?.left ?? 0, mainRight: main?.right ?? 0, wide };
}

const pages = await findPages();
const { origin, close } = await serve();
const browser = await chromium.launch();
const problems = [];

await Promise.all(
	WIDTHS.map(async (width) => {
		const page = await browser.newPage({ viewport: { width, height: 900 } });
		for (const path of pages) {
			await page.goto(origin + path, { waitUntil: 'load' });
			await diagramsDrawn(page, 'light');
			const m = await page.evaluate(measure);
			if (m.scrollWidth > m.viewport + TOLERANCE) {
				problems.push(`${path} at ${width}px scrolls sideways: ${m.scrollWidth}px wide in ${m.viewport}px.`);
			}
			if (m.mainLeft < -TOLERANCE || m.mainRight > m.viewport + TOLERANCE) {
				problems.push(`${path} at ${width}px: the text column runs off the screen.`);
			}
			for (const what of m.wide) problems.push(`${path} at ${width}px: ${what} is wider than the screen.`);
		}
		await page.close();
	}),
);

await browser.close();
await close();

if (problems.length > 0) {
	console.error(`\n${problems.length} layout problem(s):\n`);
	for (const problem of problems.sort()) console.error(`  ${problem}`);
	process.exit(1);
}
console.log(`\nNo layout problems on ${pages.length} pages at ${WIDTHS.join(', ')} pixels wide.`);
