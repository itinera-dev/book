// A new scratch directory for a build, under node_modules/.cache rather than
// the system's temporary directory. Astro moves what it built from .astro into
// the build's directory by renaming, which fails when the two are on different
// file systems, as /tmp often is, on GitHub's Ubuntu 26.04 runners among others.

import { mkdirSync, mkdtempSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const cache = fileURLToPath(new URL('../node_modules/.cache/', import.meta.url));

/** Makes a new, empty directory whose name starts with `prefix`, and returns its path. */
export function scratchDirectory(prefix) {
	mkdirSync(cache, { recursive: true });
	return mkdtempSync(join(cache, prefix));
}
