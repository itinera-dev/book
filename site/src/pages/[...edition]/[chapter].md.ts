// The Markdown copy of each page of the edition this build renders, next to the
// page itself: /book/rust/05-a-first-journey.md.

import type { APIRoute, GetStaticPaths } from 'astro';

import { build } from '../../../book.config.mjs';
import { contents, type Page } from '../../lib/contents.ts';
import { pageMarkdown } from '../../lib/markdown.ts';

export const getStaticPaths: GetStaticPaths = () =>
	contents().pages.map((page) => ({ params: { edition: build.segment, chapter: page.slug }, props: { page } }));

export const GET: APIRoute = async ({ props }) =>
	new Response(await pageMarkdown((props as { page: Page }).page), { headers: { 'Content-Type': 'text/markdown; charset=utf-8' } });
