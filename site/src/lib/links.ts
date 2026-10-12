// Turns the relative links of the book's Markdown into links that work on the
// web pages.
//
// The chapters link to each other by file name, `06-a-workflow-of-one-step.md`,
// so that they work on GitHub. On the site such a link goes to the edition's
// page for that chapter. A link to any other file of the repository, such as
// `../../BOOK.md`, goes to that file on GitHub. A link with a scheme, or one to a
// fragment of the same page, is left alone. A link to a chapter that is not in
// the contents fails the build, since it would be a dead link.

import { posix } from 'node:path';

import type { Root } from 'mdast';
import { visit } from 'unist-util-visit';

import { chapters, pageHref, sourceHref } from '../../book.config.mjs';

const SCHEME = /^[a-z][a-z0-9+.-]*:/i;

/**
 * Where a link of a Markdown file points on the site.
 *
 * @param url The link as written.
 * @param from The file it is written in, relative to the repository.
 * @param pages The slugs of the book's pages.
 */
export function resolveLink(url: string, from: string, pages: ReadonlySet<string>): string {
	if (url === '' || url.startsWith('#') || url.startsWith('//') || SCHEME.test(url)) return url;

	const hash = url.indexOf('#');
	const path = hash === -1 ? url : url.slice(0, hash);
	const fragment = hash === -1 ? '' : url.slice(hash);
	const target = posix.normalize(posix.join(posix.dirname(from), path));
	if (target.startsWith('..')) {
		throw new Error(`${from}: the link "${url}" leaves the repository.`);
	}

	if (target === `${chapters}/README.md`) return pageHref() + fragment;
	const chapter = target.startsWith(`${chapters}/`) ? /^([a-z0-9-]+)\.md$/.exec(target.slice(chapters.length + 1)) : null;
	if (chapter) {
		if (!pages.has(chapter[1])) {
			throw new Error(`${from}: the link "${url}" names ${target}, which is not a page of the book's contents.`);
		}
		return pageHref(chapter[1]) + fragment;
	}
	// The repository's root normalises to `./`, which GitHub names by no path at all.
	return sourceHref(target === './' ? '' : target) + fragment;
}

/** Rewrites every link and link definition of a tree written in `from`. */
export function rewriteLinks(tree: Root, from: string, pages: ReadonlySet<string>): void {
	visit(tree, (node) => {
		if (node.type === 'link' || node.type === 'definition') {
			node.url = resolveLink(node.url, from, pages);
		}
	});
}
