// Turns a page of the book into its Markdown copy, for agents and anyone who
// reads plain text: the chapter as written, with each directive replaced by
// the code or output it names under its caption, and every link made absolute.
// A link to another page of the book goes to that page's Markdown copy.

import { join } from 'node:path';

import type { Root } from 'mdast';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import remarkStringify from 'remark-stringify';
import { unified } from 'unified';
import { visit } from 'unist-util-visit';
import { VFile } from 'vfile';

import { build, markdownHref, pageHref, root, site } from '../../book.config.mjs';
import { contents, type Page } from './contents.ts';
import remarkBookDirectives from './directives.ts';
import { rewriteLinks } from './links.ts';
import { readText } from './text.ts';

/** The slugs of the book's pages, which links to chapters are checked against. */
function slugs(): ReadonlySet<string> {
	return new Set(contents().pages.map((page) => page.slug));
}

/** Where a link the web page would follow goes in the Markdown copy. */
function absolute(url: string): string {
	if (!url.startsWith('/')) return url;
	const hash = url.indexOf('#');
	const path = hash === -1 ? url : url.slice(0, hash);
	const fragment = hash === -1 ? '' : url.slice(hash);
	if (path === pageHref()) return `${site}${markdownHref()}${fragment}`;
	const page = [...slugs()].find((slug) => path === pageHref(slug));
	return `${site}${page ? markdownHref(page) : path}${fragment}`;
}

function remarkMarkdownCopy(options: { from: string }) {
	return (tree: Root) => {
		rewriteLinks(tree, options.from, slugs());
		visit(tree, (node) => {
			if (node.type === 'link' || node.type === 'definition') node.url = absolute(node.url);
		});
	};
}

/**
 * The Markdown copy of a file of the repository, such as a chapter or the contents.
 *
 * @param repositoryPath The file, relative to the repository.
 */
export async function markdownCopy(repositoryPath: string): Promise<string> {
	const file = new VFile({ value: readText(join(root, repositoryPath)), path: join(root, repositoryPath) });
	const processor = unified()
		.use(remarkParse)
		.use(remarkGfm)
		.use(remarkMarkdownCopy, { from: repositoryPath })
		.use(remarkBookDirectives, { edition: build.edition, markdown: true })
		.use(remarkStringify, { bullet: '-', fences: true, rule: '-' });
	return String(await processor.process(file));
}

/** The Markdown copy of one page of the book. */
export function pageMarkdown(page: Page): Promise<string> {
	return markdownCopy(page.file);
}
