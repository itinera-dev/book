// Runs a forced production build, and fails when the build printed an error
// even if it exited with zero. Borrowed from the author's own site, where a page
// that failed to render was once emitted empty and published.
//
// `--force` matters too: Astro keeps caches between builds, and a build that
// serves from them is not a build of the code as it is now.

import { spawn } from 'node:child_process';

const FATAL = ['Error rendering', 'Failed to parse', '[ERROR]'];

let output = '';
const child = spawn('npx', ['astro', 'build', '--force'], { shell: true, stdio: ['inherit', 'pipe', 'pipe'] });
const watch = (chunk) => {
	output += chunk.toString();
	process.stdout.write(chunk);
};
child.stdout.on('data', watch);
child.stderr.on('data', watch);

child.on('close', (code) => {
	const offences = output.split(/\r?\n/).filter((line) => FATAL.some((needle) => line.includes(needle)));
	if (offences.length > 0) {
		console.error('\nThe build printed errors. It did not succeed:\n');
		for (const line of offences) console.error(`  ${line.trim()}`);
		process.exit(1);
	}
	process.exit(code ?? 1);
});
