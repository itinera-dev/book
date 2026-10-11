// Turns the book's Markdown into HTML for the edition this build renders.

import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import rehypeShiki, { type RehypeShikiOptions } from '@shikijs/rehype';
import rehypeMermaid from 'rehype-mermaid';
import rehypeSlug from 'rehype-slug';
import rehypeStringify from 'rehype-stringify';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import type { Root } from 'mdast';
import { unified } from 'unified';
import { VFile } from 'vfile';

import { build, root } from '../../book.config.mjs';
import { contents, type Page } from './contents.ts';
import remarkBookDirectives from './directives.ts';
import { rewriteLinks } from './links.ts';
import { rehypeDiagrams, rehypeListingNames } from './rehype.ts';

function remarkBookLinks(options: { pages: () => ReadonlySet<string> }) {
	return (tree: Root, file: VFile) => {
		rewriteLinks(tree, String(file.data.repositoryPath), options.pages());
	};
}

let processor: ReturnType<typeof create> | undefined;

/** The slugs of the book's pages, read again on every call so the dev server sees a chapter added to the contents. */
function pages(): ReadonlySet<string> {
	return new Set(contents().pages.map((page) => page.slug));
}

function create() {
	return unified()
		.use(remarkParse)
		.use(remarkGfm)
		.use(remarkBookLinks, { pages })
		.use(remarkBookDirectives, { edition: build.edition, pages })
		.use(remarkRehype)
		// Drawn at build time as two images, one per colour scheme, so a diagram
		// needs no JavaScript and its text alternative is its accDescr.
		.use(rehypeMermaid, { strategy: 'img-svg', dark: true })
		.use(rehypeDiagrams)
		.use(rehypeShiki, {
			themes: { light: 'github-light-default', dark: 'github-dark-default' },
			defaultColor: 'light',
			// Plain text, for output and compiler messages, needs no grammar.
			langs: [build.edition.highlight] as RehypeShikiOptions['langs'],
		})
		.use(rehypeListingNames)
		.use(rehypeSlug)
		.use(rehypeStringify);
}

/**
 * Renders Markdown written in a file of the repository.
 *
 * @param source The Markdown.
 * @param repositoryPath The file it comes from, relative to the repository, which relative links are resolved against.
 */
export async function renderMarkdown(source: string, repositoryPath: string): Promise<string> {
	processor ??= create();
	const file = new VFile({ value: source, path: join(root, repositoryPath), data: { repositoryPath } });
	return String(await processor.process(file));
}

export interface RenderedPage {
	/** The page's level 1 heading, as text. */
	title: string;
	html: string;
}

/**
 * Renders one page of the book. Nothing is cached: a build renders each page
 * once anyway, and the dev server must show a chapter as it is now.
 */
export async function renderPage(page: Page): Promise<RenderedPage> {
	const source = readFileSync(join(root, page.file), 'utf8');
	const heading = /^# (.+)$/m.exec(source);
	if (!heading) throw new Error(`${page.file}: the page has no level 1 heading.`);
	return { title: heading[1].replace(/`/g, '').trim(), html: await renderMarkdown(source, page.file) };
}
