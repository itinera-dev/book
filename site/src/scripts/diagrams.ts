// Draws the Mermaid diagrams of a page in the reader's browser, as GitHub
// does, in the colours of the reader's colour scheme, and draws them again
// when that scheme changes.
//
// Mermaid is loaded only by a page that has a diagram. While the diagrams are
// being drawn, the page's root element carries `data-diagrams="drawing"`;
// once they are all drawn, `data-diagrams` names the colour scheme they were
// drawn in, `light` or `dark`, so the browser checks of `npm run verify` know
// when to look at them.

/** How much wider than tall a diagram is before it scrolls on a narrow screen instead of shrinking. */
const WIDE = 2;

const figures = [...document.querySelectorAll<HTMLElement>('figure.diagram')];

if (figures.length > 0) {
	const dark = window.matchMedia('(prefers-color-scheme: dark)');
	const root = document.documentElement;
	let drawings = 0;

	const draw = async () => {
		const scheme = dark.matches ? 'dark' : 'light';
		root.dataset.diagrams = 'drawing';
		const { default: mermaid } = await import('mermaid');
		mermaid.initialize({ startOnLoad: false, theme: 'base', themeVariables: palette(), securityLevel: 'strict' });
		for (const figure of figures) await drawOne(mermaid, figure);
		root.dataset.diagrams = scheme;
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
		const drawing = document.createElement('div');
		drawing.className = 'diagram__drawing';
		drawing.innerHTML = svg;
		box.append(drawing);
		source.hidden = true;
		const picture = drawing.querySelector('svg');
		const [, , width, height] = (picture?.getAttribute('viewBox') ?? '').split(/\s+/).map(Number);
		if (width > WIDE * height) makeWide(figure, box);
	};

	// A wide diagram leaves the text column, and its box scrolls. A box that
	// scrolls must be reachable by keyboard and say what it is, so it becomes
	// a focusable group named by the diagram's caption.
	const makeWide = (figure: HTMLElement, box: HTMLElement) => {
		figure.classList.add('diagram--wide');
		box.tabIndex = 0;
		box.setAttribute('role', 'group');
		const caption = figure.querySelector('figcaption');
		if (caption?.id) box.setAttribute('aria-labelledby', caption.id);
		else box.setAttribute('aria-label', 'Diagram');
	};

	void draw();
	dark.addEventListener('change', () => void draw());
}
