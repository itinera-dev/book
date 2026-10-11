// What the checks that drive a browser share: the built pages, served over
// HTTP under the book's base path as GitHub Pages serves them, and the list of
// every page that was built.
//
// The pages are served rather than opened from disk because they link their
// styles and each other by absolute path, /book/..., which a file:// URL cannot
// resolve, and an unstyled page passes colour checks it would fail in a browser.

import { readFile, readdir } from 'node:fs/promises';
import { createServer } from 'node:http';
import { extname, join, relative } from 'node:path';

import { base } from '../book.config.mjs';

export const DIST = join(process.cwd(), 'dist');

const TYPES = {
	'.html': 'text/html; charset=utf-8',
	'.css': 'text/css; charset=utf-8',
	'.js': 'text/javascript; charset=utf-8',
	'.json': 'application/json',
	'.svg': 'image/svg+xml',
	'.png': 'image/png',
	'.woff2': 'font/woff2',
};

/** Every built page's path, such as `/book/rust/06-a-workflow-of-one-step/`, in order, then the 404 page. */
export async function findPages(dir = DIST) {
	const pages = [];
	for (const entry of await readdir(dir, { withFileTypes: true })) {
		const full = join(dir, entry.name);
		if (entry.isDirectory()) {
			if (entry.name === '_astro') continue;
			pages.push(...(await findPages(full)));
		} else if (entry.name === 'index.html') {
			const path = relative(DIST, dir).split('\\').join('/');
			pages.push(path === '' ? `${base}/` : `${base}/${path}/`);
		}
	}
	return pages.sort();
}

/** The file a URL path names in the built site, or undefined when it is outside the base. */
export function fileFor(path) {
	if (path !== base && !path.startsWith(`${base}/`)) return undefined;
	const inside = decodeURIComponent(path.slice(base.length)) || '/';
	return inside.endsWith('/') ? join(DIST, inside, 'index.html') : join(DIST, inside);
}

/** Serves the built site until `close` is called. */
export async function serve() {
	const server = createServer(async (request, response) => {
		const file = fileFor((request.url ?? '/').split('?')[0]);
		try {
			if (!file) throw new Error('outside the base');
			const body = await readFile(file);
			response.writeHead(200, { 'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream' });
			response.end(body);
		} catch {
			response.writeHead(404, { 'Content-Type': 'text/html; charset=utf-8' });
			response.end(await readFile(join(DIST, '404.html')).catch(() => 'not found'));
		}
	});
	await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
	const address = server.address();
	const origin = `http://127.0.0.1:${typeof address === 'object' && address ? address.port : 0}`;
	return { origin, close: () => new Promise((resolve) => server.close(resolve)) };
}
