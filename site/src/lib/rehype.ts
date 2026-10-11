// Small finishing passes over the rendered HTML.

import type { Element, Root } from 'hast';
import { toString } from 'hast-util-to-string';
import type { VFile } from 'vfile';
import { visit } from 'unist-util-visit';

const hasClass = (node: Element, name: string) => {
	const classes = node.properties.className;
	return Array.isArray(classes) && classes.includes(name);
};

/**
 * Names each code block after its caption.
 *
 * The highlighter makes every `<pre>` focusable, so a keyboard can scroll a
 * wide block at phone width. A focusable element that announces nothing is a
 * dead stop in the tab order, so each one is a group labelled by the caption
 * of the figure it sits in: "Rust, from the program ch06_one_step". The role
 * is what lets a screen reader announce that name: on a bare `<pre>`, many
 * ignore it.
 */
export function rehypeListingNames() {
	return (tree: Root) => {
		visit(tree, 'element', (node) => {
			if (node.tagName !== 'figure' || !hasClass(node, 'listing')) return;
			const caption = node.children.find((child): child is Element => child.type === 'element' && child.tagName === 'figcaption');
			const id = caption?.properties.id;
			if (typeof id !== 'string') return;
			// The highlighter may leave its block wrapped in a fragment, so the
			// `<pre>` is looked for anywhere inside the figure.
			visit(node, 'element', (inner) => {
				if (inner.tagName !== 'pre') return;
				inner.properties.role = 'group';
				inner.properties.ariaLabelledBy = [id];
			});
		});
	};
}

/**
 * Puts each Mermaid diagram in a figure captioned with its title, to be drawn
 * in the reader's browser, as GitHub draws one.
 *
 * The diagram's source stays in the page, so a reader without JavaScript still
 * reads it as text; `src/scripts/diagrams.ts` draws it in its place. Its
 * `accTitle` becomes the figure's visible caption, which also names the
 * drawing, and its `accDescr` describes the drawing. A diagram without either
 * fails the build. The text after each diagram in the chapter still says
 * everything it shows.
 *
 * Each diagram sits in a box of its own. A wide one, such as two pictures side
 * by side, keeps a size its words can be read at and scrolls inside that box
 * on a narrow screen; the script then makes the box a focusable group named by
 * the caption, as a table's is.
 */
export function rehypeDiagrams() {
	return (tree: Root, file: VFile) => {
		let diagrams = 0;
		visit(tree, 'element', (node, index, parent) => {
			if (node.tagName !== 'pre' || !parent || index === undefined) return;
			const code = node.children.find((child): child is Element => child.type === 'element' && child.tagName === 'code');
			if (!code || !hasClass(code, 'language-mermaid')) return;
			const source = toString(code).replace(/\n+$/, '');
			const title = /^[ \t]*accTitle[ \t]*:[ \t]*(\S.*?)[ \t]*$/m.exec(source)?.[1];
			if (title === undefined || !/^\s*accDescr\s*[:{]/m.test(source)) {
				const where = `${file.data.repositoryPath ?? 'a chapter'}, line ${node.position?.start.line ?? '?'}`;
				throw new Error(`${where}: a diagram needs an accTitle and an accDescr, which name and describe it for a screen reader.`);
			}
			diagrams += 1;
			const caption = `diagram-${diagrams}`;
			const figure: Element = {
				type: 'element',
				tagName: 'figure',
				properties: { className: ['diagram'] },
				children: [
					{
						type: 'element',
						tagName: 'div',
						properties: { className: ['diagram__scroll'] },
						children: [
							{
								type: 'element',
								tagName: 'pre',
								properties: { className: ['diagram__source'] },
								children: [{ type: 'text', value: source }],
							},
						],
					},
					{
						type: 'element',
						tagName: 'figcaption',
						properties: { id: caption, className: ['diagram__caption'] },
						children: [{ type: 'text', value: `Diagram: ${title}` }],
					},
				],
			};
			parent.children[index] = figure;
			return index + 1;
		});
	};
}

/**
 * Lets a wide table scroll inside its own box at phone width.
 *
 * The table keeps its own display, so screen readers still treat it as a
 * table; the box around it scrolls instead. A box that scrolls must be
 * reachable by keyboard, and a stop in the tab order must say what it is, so
 * the box is a focusable group named after the table's column headings.
 */
export function rehypeTables() {
	return (tree: Root) => {
		visit(tree, 'element', (node, index, parent) => {
			if (node.tagName !== 'table' || !parent || index === undefined) return;
			const headings: string[] = [];
			visit(node, 'element', (cell) => {
				if (cell.tagName === 'th') headings.push(toString(cell).trim());
			});
			parent.children[index] = {
				type: 'element',
				tagName: 'div',
				properties: {
					className: ['table-scroll'],
					role: 'group',
					tabIndex: 0,
					ariaLabel: headings.length > 0 ? `Table: ${headings.join(', ')}` : 'Table',
				},
				children: [node],
			};
			return index + 1;
		});
	};
}
