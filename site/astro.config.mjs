// @ts-check

import { join, normalize, sep } from 'node:path';

import sitemap from '@astrojs/sitemap';
import { defineConfig } from 'astro/config';

import { base, build, chapters, dist, editionDirectories, editions, root, site } from './book.config.mjs';

/**
 * Reloads the dev server's pages when the book changes.
 *
 * Each language's chapters and files live outside this directory, and the
 * pages read them with `fs` rather than importing them, so Vite would never
 * notice an edit. This adds them to its watcher and asks the browser for a full
 * reload whenever one changes, which is enough for an edit to a page.
 *
 * A change to the book's shape needs more: the contents, the chapters' README.md,
 * decides which pages exist and how they are numbered and linked, and Astro
 * works the pages out once. So a change to the contents, or a file added to or
 * removed from the chapters, restarts the server instead.
 *
 * A language's `target/` is left out on purpose: a build of the examples
 * writes thousands of files there.
 *
 * @returns {import('vite').Plugin}
 */
function watchTheBook() {
	const directories = editions.flatMap((edition) =>
		editionDirectories.map((directory) => join(root, edition.directory, directory)),
	);
	const chapterDirectory = join(root, chapters);
	const contents = join(chapterDirectory, 'README.md');
	/** @param {string} file */
	const inTheBook = (file) => directories.some((directory) => file.startsWith(directory + sep));
	/** @param {string} file */
	const shapesTheBook = (file) => file.startsWith(chapterDirectory + sep);
	return {
		name: 'itinera-book:watch',
		apply: 'serve',
		configureServer(server) {
			server.watcher.add(directories);
			// On Windows the watcher reports some paths with forward slashes and
			// others with backslashes, so each is normalised before it is compared.
			server.watcher.on('change', (reported) => {
				const file = normalize(reported);
				if (file === contents) server.restart();
				else if (inTheBook(file)) server.ws.send({ type: 'full-reload', path: '*' });
			});
			/** @param {string} reported */
			const addedOrRemoved = (reported) => {
				const file = normalize(reported);
				if (shapesTheBook(file)) server.restart();
				else if (inTheBook(file)) server.ws.send({ type: 'full-reload', path: '*' });
			};
			server.watcher.on('add', addedOrRemoved);
			server.watcher.on('unlink', addedOrRemoved);
		},
	};
}

// The chapters are not an Astro content collection: they live outside this
// directory, in each language's chapters/, and each edition renders its own,
// with its language's code. src/lib/render.ts does that, and the pages call it.
//
// Only the latest release's edition is in the sitemap, so a search engine sends
// readers to it rather than to `next` or to a frozen release.
export default defineConfig({
	site,
	base,
	outDir: dist,
	trailingSlash: 'always',
	build: { format: 'directory' },
	integrations: build.version === '' ? [sitemap({ filter: (page) => !page.endsWith('/404/') })] : [],
	vite: {
		plugins: [watchTheBook()],
		// Mermaid is large, and is fetched only by a page that has a diagram.
		build: { chunkSizeWarningLimit: 1000 },
	},
});
