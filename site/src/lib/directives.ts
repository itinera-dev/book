// Renders the directives of FORMAT.md for one edition of the book.
//
// A chapter never holds code. It holds fenced blocks such as
//
//     ```example
//     name: ch06_one_step
//     region: step
//     ```
//
// and each edition replaces them with what its language supplies, from the
// language's directory of this repository:
//
// - `example`: the region of `examples/NAME.EXT` between `ANCHOR: region` and
//   `ANCHOR_END: region`, every anchor line removed, as highlighted code under a
//   caption naming the program. When the program is in `pending/` instead, the
//   caption says it uses a feature not released yet and is not compiled; a
//   released edition has no pending examples, so there it fails the build.
//   With `asynchronous: NAME`, the asynchronous form follows, only when its
//   region differs and the language has an asynchronous execution mode.
// - `output`: `expected/NAME.txt`, as a plain block captioned as what the
//   program prints. With `asynchronous: NAME`, what the asynchronous form
//   prints follows, under the same two conditions.
// - `note`: `notes/NAME.md`, as prose in a box headed for the language; nothing
//   when the language has no such note.
// - `compile-error`: the program, captioned as one that does not compile, then
//   the compiler's message. A language that cannot make the mistake a compile
//   error supplies a note of the same name, shown instead.
//
// Every file a directive names must exist, and so must every region: anything
// else fails the build with a message naming the chapter and the file, since a
// page with a missing example would teach nothing and say nothing about it.

import { existsSync } from 'node:fs';
import { join, relative } from 'node:path';

import type { Code, PhrasingContent, Root, RootContent } from 'mdast';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import { unified } from 'unified';
import type { VFile } from 'vfile';

import { build, type Edition, root, sourceHref } from '../../book.config.mjs';
import { rewriteLinks } from './links.ts';
import { readText } from './text.ts';

export interface DirectiveOptions {
	edition: Edition;
	/** The slugs of the book's pages, for the links inside notes. */
	pages: () => ReadonlySet<string>;
}

type Kind = 'example' | 'output' | 'note' | 'compile-error';

const KEYS: Record<Kind, readonly string[]> = {
	example: ['name', 'region', 'asynchronous'],
	output: ['name', 'asynchronous'],
	note: ['name'],
	'compile-error': ['name'],
};

const isKind = (lang: string | null | undefined): lang is Kind => lang != null && lang in KEYS;

const NAME = /^[a-z0-9_]+$/;

interface Directive {
	kind: Kind;
	name: string;
	region?: string;
	asynchronous?: string;
}

function parse(node: Code, kind: Kind, chapter: string): Directive {
	const where = `${chapter}, line ${node.position?.start.line ?? '?'}`;
	const fields = new Map<string, string>();
	for (const line of node.value.split('\n')) {
		if (line.trim() === '') continue;
		const match = /^([a-z-]+):\s*(.+?)\s*$/.exec(line);
		if (!match) throw new Error(`${where}: "${line}" in a ${kind} directive is not a "key: value" line.`);
		const [, key, value] = match;
		if (!KEYS[kind].includes(key)) {
			throw new Error(`${where}: a ${kind} directive has no key "${key}". It takes ${KEYS[kind].join(', ')}.`);
		}
		if (!NAME.test(value)) throw new Error(`${where}: "${value}" is not a name of lower-case letters, digits and underscores.`);
		fields.set(key, value);
	}
	const name = fields.get('name');
	if (!name) throw new Error(`${where}: a ${kind} directive needs a name.`);
	return { kind, name, region: fields.get('region'), asynchronous: fields.get('asynchronous') };
}

const ANCHOR_LINE = /\bANCHOR(?:_END)?:\s*[\w-]+\s*$/;

/** The lines of a program between two anchors, or the whole program, with every anchor line removed. */
function region(source: string, anchor: string | undefined, where: string): string {
	const lines = source.replace(/\n+$/, '').split('\n');
	let start = -1;
	let end = lines.length;
	if (anchor !== undefined) {
		start = lines.findIndex((line) => new RegExp(`\\bANCHOR:\\s*${anchor}\\s*$`).test(line));
		if (start === -1) throw new Error(`${where}: there is no "ANCHOR: ${anchor}".`);
		end = lines.findIndex((line, index) => index > start && new RegExp(`\\bANCHOR_END:\\s*${anchor}\\s*$`).test(line));
		if (end === -1) throw new Error(`${where}: "ANCHOR: ${anchor}" is never closed.`);
	}
	const kept = lines.slice(start + 1, end).filter((line) => !ANCHOR_LINE.test(line));
	if (kept.every((line) => line.trim() === '')) {
		throw new Error(anchor === undefined ? `${where}: the program is empty.` : `${where}: the region "${anchor}" is empty.`);
	}
	const indent = Math.min(...kept.filter((line) => line.trim() !== '').map((line) => /^\s*/.exec(line)?.[0].length ?? 0));
	return kept.map((line) => line.slice(indent)).join('\n');
}

/** A node the Markdown-to-HTML step renders as the given element. */
function element(tagName: string, properties: Record<string, unknown>, children: unknown[]): RootContent {
	return { type: 'bookElement', data: { hName: tagName, hProperties: properties }, children } as unknown as RootContent;
}

const text = (value: string): PhrasingContent => ({ type: 'text', value });
const link = (url: string, value: string): PhrasingContent => ({ type: 'link', url, children: [text(value)] });

interface Listing {
	classes: string[];
	caption: PhrasingContent[];
	lang: string;
	code: string;
}

/** Renders the directives of one chapter. */
export default function remarkBookDirectives(options: DirectiveOptions) {
	const { edition, pages } = options;
	const directory = join(root, edition.directory);
	const repoPath = (...parts: string[]) => [edition.directory, ...parts].join('/');

	const notes = unified().use(remarkParse).use(remarkGfm);

	return (tree: Root, file: VFile) => {
		const chapter = file.path ? relative(root, file.path) : 'a chapter';
		let listings = 0;

		/** Reads a file of the language's directory, or fails naming the chapter and the file. */
		const read = (path: string, what: string) => {
			const full = join(directory, path);
			if (!existsSync(full)) {
				throw new Error(`${chapter}: ${what}, but ${repoPath(path)} does not exist.`);
			}
			return readText(full);
		};

		const figure = ({ classes, caption, lang, code }: Listing): RootContent => {
			listings += 1;
			const id = `listing-${listings}`;
			return element('figure', { className: ['listing', ...classes] }, [
				element('figcaption', { id, className: ['listing__caption'] }, caption),
				{ type: 'code', lang, value: code } satisfies Code,
			]);
		};

		const programCaption = (lead: string, name: string, path: string, after: PhrasingContent[] = []) => [
			element('span', { className: ['listing__language'] }, [text(edition.name)]) as PhrasingContent,
			text(lead),
			link(sourceHref(repoPath(path)), name),
			...after,
		];

		/** Where an example lives: released in `examples/`, or waiting in `pending/`. */
		const locate = (name: string, role: string) => {
			const program = `${name}.${edition.extension}`;
			if (existsSync(join(directory, 'examples', program))) return { path: `examples/${program}`, pending: false };
			if (existsSync(join(directory, 'pending', program))) {
				if (build.release) {
					throw new Error(
						`${chapter}: ${role} names the example ${name}, which is still in ${repoPath('pending', program)}. A released edition has no pending examples.`,
					);
				}
				return { path: `pending/${program}`, pending: true };
			}
			throw new Error(
				`${chapter}: ${role} names the example ${name}, but neither ${repoPath('examples', program)} nor ${repoPath('pending', program)} exists.`,
			);
		};

		/** A visible marker inside a caption, so it is also part of the code block's name. */
		const marker = (label: string): PhrasingContent[] => [
			text('. '),
			element('strong', { className: ['listing__marker'] }, [text(label)]) as PhrasingContent,
		];
		const pendingMarker = () => marker('Not compiled yet: it uses a feature not released yet');

		/** The asynchronous form a directive names, when the language has one. */
		const asynchronousOf = (directive: Directive) => (edition.asynchronous ? directive.asynchronous : undefined);

		const example = (directive: Directive): RootContent[] => {
			const found = locate(directive.name, 'an example directive');
			const code = region(read(found.path, `the example ${directive.name} is shown`), directive.region, `${chapter}: ${repoPath(found.path)}`);
			const shown = [
				figure({
					classes: ['listing--example', ...(found.pending ? ['listing--pending'] : [])],
					caption: programCaption(', from the program ', directive.name, found.path, found.pending ? pendingMarker() : []),
					lang: edition.highlight,
					code,
				}),
			];
			const asynchronous = asynchronousOf(directive);
			if (asynchronous) {
				const other = locate(asynchronous, `the asynchronous form of ${directive.name}`);
				const otherCode = region(
					read(other.path, `the asynchronous form ${asynchronous} is shown`),
					directive.region,
					`${chapter}: ${repoPath(other.path)}`,
				);
				if (otherCode !== code) {
					shown.push(
						figure({
							classes: ['listing--example', 'listing--asynchronous', ...(other.pending ? ['listing--pending'] : [])],
							caption: programCaption(', the same in its asynchronous form, from the program ', asynchronous, other.path, other.pending ? pendingMarker() : []),
							lang: edition.highlight,
							code: otherCode,
						}),
					);
				}
			}
			return shown;
		};

		const recorded = (name: string, what: string) => {
			const path = `expected/${name}.txt`;
			return { path, text: read(path, what).replace(/\n+$/, '') };
		};

		const output = (directive: Directive): RootContent[] => {
			const sync = recorded(directive.name, `an output directive names ${directive.name}`);
			const shown = [
				figure({
					classes: ['listing--output'],
					caption: [text(`What ${directive.name} prints, recorded in `), link(sourceHref(repoPath(sync.path)), `${directive.name}.txt`)],
					lang: 'text',
					code: sync.text,
				}),
			];
			const asynchronous = asynchronousOf(directive);
			if (asynchronous) {
				const other = recorded(asynchronous, `the output of the asynchronous form ${asynchronous} is shown`);
				if (other.text !== sync.text) {
					shown.push(
						figure({
							classes: ['listing--output', 'listing--asynchronous'],
							caption: [
								text(`What the asynchronous form ${asynchronous} prints, recorded in `),
								link(sourceHref(repoPath(other.path)), `${asynchronous}.txt`),
							],
							lang: 'text',
							code: other.text,
						}),
					);
				}
			}
			return shown;
		};

		const note = (name: string): RootContent[] => {
			const path = `notes/${name}.md`;
			if (!existsSync(join(directory, path))) return [];
			const parsed = notes.parse(readText(join(directory, path))) as Root;
			rewriteLinks(parsed, repoPath(path), pages());
			listings += 1;
			const id = `note-${listings}`;
			return [
				element('div', { className: ['note'], role: 'note', ariaLabelledBy: id }, [
					element('p', { id, className: ['note__title'] }, [text(`In ${edition.name}`)]),
					...parsed.children,
				]),
			];
		};

		const compileError = (directive: Directive): RootContent[] => {
			const program = `compile-errors/${directive.name}.${edition.extension}`;
			if (!existsSync(join(directory, program))) {
				const instead = note(directive.name);
				if (instead.length > 0) return instead;
				throw new Error(
					`${chapter}: a compile-error directive names ${directive.name}, but neither ${repoPath(program)} nor the note ${repoPath('notes', `${directive.name}.md`)} that would replace it exists.`,
				);
			}
			const code = region(readText(join(directory, program)), undefined, `${chapter}: ${repoPath(program)}`);
			const message = read(`compile-errors/${directive.name}.txt`, `the compile error ${directive.name} is shown`).replace(/\n+$/, '');
			return [
				figure({
					classes: ['listing--example', 'listing--refused'],
					caption: programCaption(', the program ', directive.name, program, marker('Does not compile')),
					lang: edition.highlight,
					code,
				}),
				figure({
					classes: ['listing--output', 'listing--compiler'],
					caption: [
						text(`What the compiler says about ${directive.name}, recorded in `),
						link(sourceHref(repoPath(`compile-errors/${directive.name}.txt`)), `${directive.name}.txt`),
					],
					lang: 'text',
					code: message,
				}),
			];
		};

		const render = (directive: Directive): RootContent[] => {
			switch (directive.kind) {
				case 'example':
					return example(directive);
				case 'output':
					return output(directive);
				case 'note':
					return note(directive.name);
				case 'compile-error':
					return compileError(directive);
			}
		};

		const walk = (parent: { children: RootContent[] }) => {
			for (let index = 0; index < parent.children.length; index++) {
				const child = parent.children[index];
				if (child.type === 'code' && isKind(child.lang)) {
					const replacement = render(parse(child, child.lang, chapter));
					parent.children.splice(index, 1, ...replacement);
					index += replacement.length - 1;
				} else if ('children' in child) {
					// A replacement is skipped by the index above, so only the
					// chapter's own nodes are walked.
					walk(child);
				}
			}
		};

		walk(tree);
	};
}
