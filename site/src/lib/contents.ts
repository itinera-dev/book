// The book's contents, read from chapters/README.md: the parts in order, each
// with its pages, numbered as the README numbers them. The navigation, the
// previous and next links and the part a page belongs to all come from here,
// so the README stays the one place that orders the book.

import { join } from 'node:path';

import type { Heading, List, ListItem, Paragraph, PhrasingContent, Root } from 'mdast';
import remarkParse from 'remark-parse';
import { unified } from 'unified';

import { root } from '../../book.config.mjs';
import { readText } from './text.ts';

export type PageKind = 'front' | 'chapter' | 'without-itinera' | 'appendix';

export interface Page {
	/** The file name without `.md`, which is also the page's URL segment. */
	slug: string;
	/** The file, relative to the repository. */
	file: string;
	kind: PageKind;
	/** The chapter's number, for a chapter. */
	number?: number;
	/** How the contents names the page: the chapter's or front page's title, "Without Itinera", or "Appendix 1: Cheat sheet". */
	label: string;
	/** The part the page belongs to. */
	part: Part;
}

export interface Part {
	/** The part's heading in the contents, such as "Part 2: Getting started" or "Appendices". */
	title: string;
	/** A stable identifier for the part's heading. */
	id: string;
	pages: Page[];
}

export interface Contents {
	/** The Markdown source of the paragraphs above the first part. */
	introduction: string;
	parts: Part[];
	/** Every page, in reading order. */
	pages: Page[];
}

const CONTENTS = 'chapters/README.md';

function text(nodes: PhrasingContent[]): string {
	return nodes
		.map((node) => {
			if (node.type === 'text' || node.type === 'inlineCode') return node.value;
			if ('children' in node) return text(node.children as PhrasingContent[]);
			return '';
		})
		.join('');
}

/** A numbered entry is a chapter; an unnumbered one is front matter, unless its file says otherwise. */
function kindOf(slug: string, numbered: boolean): PageKind {
	if (/^part-\d+-without-itinera$/.test(slug)) return 'without-itinera';
	if (/^appendix-\d+-/.test(slug)) return 'appendix';
	return numbered ? 'chapter' : 'front';
}

function pageOf(item: ListItem, number: number | undefined, part: Part): Page {
	const paragraph = item.children.find((child): child is Paragraph => child.type === 'paragraph');
	const link = paragraph?.children.find((child) => child.type === 'link');
	if (!paragraph || !link || link.type !== 'link') {
		throw new Error(`${CONTENTS}: an entry of "${part.title}" is not a link to a chapter.`);
	}
	const match = /^([a-z0-9-]+)\.md$/.exec(link.url);
	if (!match) {
		throw new Error(`${CONTENTS}: "${link.url}" in "${part.title}" is not a chapter file of this directory.`);
	}
	const slug = match[1];
	const kind = kindOf(slug, number !== undefined);
	const label = text(paragraph.children).trim();
	return {
		slug,
		file: `chapters/${slug}.md`,
		kind,
		number: kind === 'chapter' ? number : undefined,
		label: kind === 'chapter' || kind === 'front' ? text(link.children).trim() : label,
		part,
	};
}

function slugOf(title: string): string {
	return title
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-|-$/g, '');
}

function read(): Contents {
	const source = readText(join(root, CONTENTS));
	const tree = unified().use(remarkParse).parse(source) as Root;
	const parts: Part[] = [];
	const introduction: string[] = [];

	for (const node of tree.children) {
		if (node.type === 'heading' && (node as Heading).depth === 2) {
			const title = text((node as Heading).children).trim();
			parts.push({ title, id: slugOf(title), pages: [] });
		} else if (node.type === 'paragraph' && parts.length === 0) {
			introduction.push(source.slice(node.position?.start.offset, node.position?.end.offset));
		} else if (node.type === 'list') {
			const part = parts.at(-1);
			if (!part) throw new Error(`${CONTENTS}: a list comes before the first part's heading.`);
			const list = node as List;
			const start = list.start ?? 1;
			list.children.forEach((item, index) => {
				part.pages.push(pageOf(item, list.ordered ? start + index : undefined, part));
			});
		}
	}

	const pages = parts.flatMap((part) => part.pages);
	if (pages.length === 0) throw new Error(`${CONTENTS}: no chapter was found.`);
	return { introduction: introduction.join('\n\n'), parts, pages };
}

let cached: Contents | undefined;

/**
 * The contents, read once. The dev server restarts when the README changes or a
 * chapter file is added or removed (see astro.config.mjs), and a restart reads
 * them again.
 */
export function contents(): Contents {
	cached ??= read();
	return cached;
}

/** The name of a page wherever it is listed away from the contents, such as a previous or next link. */
export function pageName(page: Page): string {
	if (page.kind === 'chapter') return `${page.number}. ${page.label}`;
	if (page.kind === 'without-itinera') return `${page.label}, ${page.part.title}`;
	return page.label;
}
