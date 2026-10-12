// Draws the Mermaid diagrams of a page in the reader's browser, as GitHub
// does, in the colours of the reader's colour scheme, and draws them again
// when that scheme changes.
//
// Mermaid is loaded only by a page that has a diagram. While the diagrams are
// being drawn, the page's root element carries `data-diagrams="drawing"`;
// once they are all drawn, `data-diagrams` names the colour scheme they were
// drawn in, `light` or `dark`, and `failed` if Mermaid could not draw them, so
// the browser checks of `npm run verify` know when to look at them.

/** How much wider than tall a diagram is before it scrolls on a narrow screen instead of shrinking. */
const WIDE = 2;

const figures = [...document.querySelectorAll<HTMLElement>('figure.diagram')];

if (figures.length > 0) {
	const dark = window.matchMedia('(prefers-color-scheme: dark)');
	const root = document.documentElement;
	let drawings = 0;

	// One drawing of every diagram at a time: a change of scheme while the
	// diagrams are being drawn waits for that drawing to end, then draws them
	// all again in the new scheme, so the last drawing is always in the scheme
	// in use.
	let requested = 0;
	let drawn = 0;
	let drawing: Promise<void> | undefined;

	const request = () => {
		requested += 1;
		root.dataset.diagrams = 'drawing';
		drawing ??= drawUntilCurrent().finally(() => {
			drawing = undefined;
		});
	};

	const drawUntilCurrent = async () => {
		try {
			const { default: mermaid } = await import('mermaid');
			while (drawn < requested) {
				const current = requested;
				const scheme = dark.matches ? 'dark' : 'light';
				mermaid.initialize({ startOnLoad: false, theme: 'base', themeVariables: palette(), securityLevel: 'strict', suppressErrorRendering: true });
				for (const figure of figures) await drawOne(mermaid, figure);
				drawn = current;
				if (drawn === requested) root.dataset.diagrams = scheme;
			}
		} catch (error) {
			drawn = requested;
			root.dataset.diagrams = 'failed';
			console.error('The diagrams could not be drawn.', error);
		}
	};

	// The diagrams take the page's own colours, from the tokens of the scheme
	// in use, so their text has the contrast the rest of the page has.
	const palette = () => {
		const style = getComputedStyle(root);
		const token = (name: string) => style.getPropertyValue(`--${name}`).trim();
		return {
			darkMode: dark.matches,
			background: token('paper'),
			mainBkg: token('paper-raised'),
			primaryColor: token('paper-raised'),
			primaryTextColor: token('ink'),
			primaryBorderColor: token('rule-strong'),
			secondaryColor: token('paper-raised'),
			tertiaryColor: token('paper'),
			clusterBkg: token('paper'),
			clusterBorder: token('rule-strong'),
			lineColor: token('ink-muted'),
			textColor: token('ink'),
			titleColor: token('ink'),
			edgeLabelBackground: token('paper'),
			fontFamily: style.getPropertyValue('--font-ui').trim(),
		};
	};

	const drawOne = async (mermaid: typeof import('mermaid').default, figure: HTMLElement) => {
		const box = figure.querySelector<HTMLElement>('.diagram__scroll');
		const source = figure.querySelector<HTMLElement>('.diagram__source');
		if (!box || !source) return;
		drawings += 1;
		const { svg } = await mermaid.render(`diagram-drawing-${drawings}`, source.textContent ?? '');
		figure.querySelector('.diagram__drawing')?.remove();
		const holder = document.createElement('div');
		holder.className = 'diagram__drawing';
		holder.innerHTML = svg;
		box.append(holder);
		source.hidden = true;
		const picture = holder.querySelector('svg');
		nameByCaption(figure, picture);
		const [, , width, height] = (picture?.getAttribute('viewBox') ?? '').split(/\s+/).map(Number);
		if (width > WIDE * height) makeWide(figure, box);
	};

	// The drawing is named by the figure's visible caption, made from the
	// diagram's accTitle, rather than by Mermaid's own title, which would also
	// show as a tooltip.
	const nameByCaption = (figure: HTMLElement, picture: SVGSVGElement | null) => {
		const caption = figure.querySelector('figcaption');
		if (!picture || !caption) return;
		picture.querySelector(':scope > title')?.remove();
		picture.setAttribute('aria-labelledby', caption.id);
	};

	// A wide diagram leaves the text column, and its box scrolls. A box that
	// scrolls must be reachable by keyboard and say what it is, so it becomes
	// a focusable group named by the diagram's caption.
	const makeWide = (figure: HTMLElement, box: HTMLElement) => {
		figure.classList.add('diagram--wide');
		box.tabIndex = 0;
		box.setAttribute('role', 'group');
		const caption = figure.querySelector('figcaption');
		if (caption) box.setAttribute('aria-labelledby', caption.id);
	};

	request();
	dark.addEventListener('change', request);
}
