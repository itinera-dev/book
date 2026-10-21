// Builds and checks, from this checkout, the edition of every language that
// it holds, each as it would be published: a language with a release as its
// latest release, at /book/LANGUAGE/, and one with none yet as its `next`
// edition, at /book/LANGUAGE/next/, since that is the only edition it has. So a
// pull request's own pages are checked in every language, whichever edition
// they will reach.
//
// Each edition gets a strict build of its own, into a directory of its own,
// then the checks of its pages, accessibility and layout, and the first that
// fails stops everything.

import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { editions, NEXT, root } from '../book.config.mjs';

const CHECKS = ['scripts/build-strict.mjs', 'scripts/pages.mjs', 'scripts/a11y.mjs', 'scripts/layout.mjs'];

const scratch = mkdtempSync(join(tmpdir(), 'itinera-book-languages-'));
try {
	for (const { language, directory, released } of editions) {
		if (!existsSync(join(root, directory, 'chapters', 'README.md'))) {
			console.warn(`This checkout has no ${directory}/chapters/README.md, so its ${language} edition is not checked.`);
			continue;
		}
		const version = released ? '' : NEXT;
		const env = {
			...process.env,
			BOOK_LANGUAGE: language,
			BOOK_VERSION: version,
			BOOK_SOURCE_REF: released ? 'main' : NEXT,
			BOOK_DIST: join(scratch, language),
		};
		for (const check of CHECKS) {
			console.log(`\n${check}, for the ${language} edition ${version || 'of the latest release'}:\n`);
			const result = spawnSync(process.execPath, [check], { stdio: 'inherit', env });
			if (result.status !== 0) {
				throw new Error(`${check} failed for the ${language} edition ${version || 'of the latest release'}.`);
			}
		}
	}
} finally {
	rmSync(scratch, { recursive: true, force: true });
}
