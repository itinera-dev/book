// @ts-check
// The logo, drawn once. Every form of it, on the pages, as the favicon, on the
// Brand page's downloads and in the sharing images, is made here.
//
// The itinerary: a short straight route with three round stations on it,
// evenly spaced. The first two are filled dots and the last is a ring, the
// journey's end. Drawn upright, with the top station set apart from the route,
// it also reads as a lowercase "i". It comes from the Roman itinerarium, a
// list of the stations along a road, in order.

/** How the logo is described in words, wherever it cannot be seen. */
export const DESCRIPTION =
	'The Itinera logo, the itinerary: three stations on a short upright route, the first two filled dots and the last a ring, which together also read as a lowercase "i".';

/** The mark's own box: 20 wide and 40 tall. */
export const MARK_BOX = { width: 20, height: 40 };

/**
 * The mark's shapes, in its own box, in one colour.
 *
 * @param {string} colour
 */
export function markShapes(colour) {
	return [
		`<circle cx="10" cy="6" r="4.5" fill="${colour}"/>`,
		`<path d="M10 19.5V28.6" stroke="${colour}" stroke-width="5"/>`,
		`<circle cx="10" cy="19.5" r="5" fill="${colour}"/>`,
		`<circle cx="10" cy="33" r="4.1" fill="none" stroke="${colour}" stroke-width="2.8"/>`,
	].join('');
}

/**
 * The mark alone, as a standalone SVG file.
 *
 * @param {string} colour
 */
export function markSvg(colour) {
	return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${MARK_BOX.width} ${MARK_BOX.height}" width="${MARK_BOX.width * 4}" height="${MARK_BOX.height * 4}" role="img" aria-labelledby="title"><title id="title">${DESCRIPTION}</title>${markShapes(colour)}</svg>\n`;
}

/**
 * The icon: the mark in the page colour on a rounded square of green, for the
 * favicon and app icons, where a mark alone would vanish on a dark tab bar.
 *
 * @param {string} ground
 * @param {string} figure
 * @param {{ square?: boolean }} [options] `square` leaves the corners square,
 *   for a phone's home screen, which rounds them itself.
 */
export function iconSvg(ground, figure, options = {}) {
	// The mark, 40 tall, scaled into the middle of a 32 by 32 square with room around it.
	const scale = 0.68;
	const x = 16 - (MARK_BOX.width * scale) / 2;
	const y = 16 - (MARK_BOX.height * scale) / 2;
	return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="${options.square ? 0 : 7}" fill="${ground}"/><g transform="translate(${x} ${y}) scale(${scale})">${markShapes(figure)}</g></svg>\n`;
}

/**
 * The full logo: the mark, then the name, set in the site's serif.
 *
 * @param {string} colour The mark's colour.
 * @param {string} ink The name's colour.
 */
export function logoSvg(colour, ink) {
	return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 132 44" width="396" height="132" role="img" aria-labelledby="title"><title id="title">Itinera. ${DESCRIPTION}</title><g transform="translate(2 2)">${markShapes(colour)}</g><text x="30" y="33" fill="${ink}" font-family="Charter, 'Bitstream Charter', 'Sitka Text', Cambria, Georgia, serif" font-size="32" font-weight="700">Itinera</text></svg>\n`;
}
