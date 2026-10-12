// The Markdown copy of the edition's contents: /book/rust/index.md.

import type { APIRoute, GetStaticPaths } from 'astro';

import { build, chapters } from '../../../book.config.mjs';
import { markdownCopy } from '../../lib/markdown.ts';

export const getStaticPaths: GetStaticPaths = () => [{ params: { edition: build.segment } }];

export const GET: APIRoute = async () =>
	new Response(await markdownCopy(`${chapters}/README.md`), { headers: { 'Content-Type': 'text/markdown; charset=utf-8' } });
