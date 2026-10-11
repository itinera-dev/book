// Small finishing passes over the rendered HTML.

import type { Element, Properties, Root } from 'hast';
import { toString } from 'hast-util-to-string';
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
 * How much wider than tall a diagram is before it scrolls on a narrow screen
 * instead of shrinking until its words are too small to read.
 */
const WIDE = 2;

/**
 * Puts each diagram in a figure captioned with its title.
 *
 * The diagrams are drawn at build time as images, one for each colour scheme,
 * and the image's text alternative is the diagram's `accDescr`. Its `accTitle`
 * arrives as a `title` attribute, which most screen readers either ignore or
 * read twice; it becomes the figure's visible caption instead. The text after
 * each diagram in the chapter still says everything it shows.
 *
 * A wide diagram, such as two pictures side by side, keeps a size its words
 * can be read at and scrolls inside its own box on a narrow screen. Like a
 * table's, that box is a focusable group, named by the diagram's caption.
 */
export function rehypeDiagrams() {
	return (tree: Root) => {
		let diagrams = 0;
		visit(tree, 'element', (node, index, parent) => {
			if (node.tagName !== 'picture' || !parent || index === undefined) return;
			const image = node.children.find((child): child is Element => child.type === 'element' && child.tagName === 'img');
			if (!image) return;
			const title = typeof image.properties.title === 'string' ? image.properties.title : '';
			delete image.properties.title;
			image.properties.loading = 'lazy';
			image.properties.decoding = 'async';
			const wide = Number(image.properties.width) > WIDE * Number(image.properties.height);
			diagrams += 1;
			const caption = `diagram-${diagrams}`;
			const scroll: Properties = { className: ['diagram__scroll'], role: 'group', tabIndex: 0 };
			if (title === '') scroll.ariaLabel = 'Diagram';
			else scroll.ariaLabelledBy = [caption];
			const figure: Element = {
				type: 'element',
				tagName: 'figure',
				properties: { className: wide ? ['diagram', 'diagram--wide'] : ['diagram'] },
				children: [
					wide
						? {
								type: 'element',
								tagName: 'div',
								properties: scroll,
								children: [node],
							}
						: node,
					...(title === ''
						? []
						: [
								{
									type: 'element' as const,
									tagName: 'figcaption',
									properties: { id: caption, className: ['diagram__caption'] },
									children: [{ type: 'text' as const, value: `Diagram: ${title}` }],
								},
							]),
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
