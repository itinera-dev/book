// Two small finishing passes over the rendered HTML.

import type { Element, Root } from 'hast';
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
 * dead stop in the tab order, so each one is labelled by the caption of the
 * figure it sits in: "Rust, from the program ch06_one_step".
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
				inner.properties.ariaLabelledBy = [id];
				inner.properties.tabIndex = 0;
			});
		});
	};
}

/**
 * Puts each diagram in a figure captioned with its title.
 *
 * The diagrams are drawn at build time as images, one for each colour scheme,
 * and the image's text alternative is the diagram's `accDescr`. Its `accTitle`
 * arrives as a `title` attribute, which most screen readers either ignore or
 * read twice; it becomes the figure's visible caption instead. The text after
 * each diagram in the chapter still says everything it shows.
 */
export function rehypeDiagrams() {
	return (tree: Root) => {
		visit(tree, 'element', (node, index, parent) => {
			if (node.tagName !== 'picture' || !parent || index === undefined) return;
			const image = node.children.find((child): child is Element => child.type === 'element' && child.tagName === 'img');
			if (!image) return;
			const title = typeof image.properties.title === 'string' ? image.properties.title : '';
			delete image.properties.title;
			image.properties.loading = 'lazy';
			image.properties.decoding = 'async';
			const figure: Element = {
				type: 'element',
				tagName: 'figure',
				properties: { className: ['diagram'] },
				children: [
					node,
					...(title === ''
						? []
						: [
								{
									type: 'element' as const,
									tagName: 'figcaption',
									properties: { className: ['diagram__caption'] },
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
