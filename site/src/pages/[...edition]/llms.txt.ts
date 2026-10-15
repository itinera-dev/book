// /book/rust/llms.txt: what this edition of the book is, and every page of it
// by its Markdown copy, following the llms.txt convention.

import type { APIRoute, GetStaticPaths } from 'astro';

import { build, markdownHref, site } from '../../../book.config.mjs';
import { noticeMarkdown } from '../../lib/markdown.ts';
import { contents, pageName } from '../../lib/contents.ts';

export const getStaticPaths: GetStaticPaths = () => [{ params: { edition: build.segment } }];

export const GET: APIRoute = () => {
	const { parts } = contents();
	const lines = [
		`# The Itinera book in ${build.edition.name}${build.version && `, ${build.version}`}`,
		'',
		`${noticeMarkdown()}> How to build with Itinera in ${build.edition.name}, a workflow framework that keeps business rules separate from flow control. Every example is a complete program that compiles and runs, and what it prints is shown under it. What Itinera does is defined only by the specification: https://github.com/itinera-dev/spec`,
		'',
		`- [Contents](${site}${markdownHref()}): every part and chapter, in reading order.`,
		...parts.flatMap((part) => ['', `## ${part.title}`, '', ...part.pages.map((page) => `- [${pageName(page)}](${site}${markdownHref(page.slug)})`)]),
		'',
		'## Optional',
		'',
		`- [Itinera](${site}/llms.txt): the project, its specification and its languages.`,
		'',
	];
	return new Response(lines.join('\n'), { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
