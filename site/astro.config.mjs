// @ts-check

import { join, sep } from 'node:path';

import { defineConfig } from 'astro/config';

import { base, editionDirectories, editions, root, site } from './book.config.mjs';

/**
 * Reloads the dev server's pages when the book changes.
 *
 * The chapters and each language's files live outside this directory, and the
 * pages read them with `fs` rather than importing them, so Vite would never
 * notice an edit. This adds them to its watcher and asks the browser for a full
 * reload whenever one changes. A language's `target/` is left out on purpose:
 * a build of the examples writes thousands of files there.
 *
 * @returns {import('vite').Plugin}
 */
function watchTheBook() {
	const directories = [
		join(root, 'chapters'),
		...editions.flatMap((edition) => editionDirectories.map((directory) => join(root, edition.directory, directory))),
	];
	/** @param {string} file */
	const inTheBook = (file) => directories.some((directory) => file.startsWith(directory + sep));
	return {
		name: 'itinera-book:watch',
		apply: 'serve',
		configureServer(server) {
			server.watcher.add(directories);
			/** @param {string} file */
			const reload = (file) => {
				if (inTheBook(file)) server.ws.send({ type: 'full-reload', path: '*' });
			};
			server.watcher.on('change', reload);
			server.watcher.on('add', reload);
			server.watcher.on('unlink', reload);
		},
	};
}

// The chapters are not an Astro content collection: they live outside this
// directory, in chapters/, and each edition renders them with its own
// language's code. src/lib/render.ts does that, and the pages call it.
export default defineConfig({
	site,
	base,
	trailingSlash: 'always',
	build: { format: 'directory' },
	vite: { plugins: [watchTheBook()] },
});
