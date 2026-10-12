// Checks that the book's copies of the org site's design are the ones it
// pinned. The site owns Itinera's colours and logo; the book keeps a copy of
// each, taken from the site at the commit brand.json names, and this check
// fails when a copy has been edited here instead of taken again from the site.
//
// To follow a newer site: copy the colours from that commit of
// itinera-dev/itinera-dev.github.io, take the icon from the /favicon.svg that
// commit builds, which src/pages/favicon.svg.ts draws, then put the commit and
// each copy's sha256 in brand.json.

import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const pin = JSON.parse(readFileSync(new URL('../brand.json', import.meta.url), 'utf8'));

const sha256 = (path) => createHash('sha256').update(readFileSync(new URL(`../${path}`, import.meta.url))).digest('hex');

const changed = pin.files.filter((file) => sha256(file.copy) !== file.sha256);
for (const file of changed) {
	console.error(`${file.copy} differs from ${file.from} at ${pin.repository}@${pin.commit}. Take it again from the site (for the icon, the /favicon.svg it builds), or pin a newer commit in brand.json.`);
}
if (changed.length > 0) process.exit(1);
console.log(`The colours and the logo match ${pin.repository}@${pin.commit.slice(0, 7)}.`);
