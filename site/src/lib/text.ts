// Reads a file of the book as text with Unix line endings and no byte order
// mark. Git for Windows checks text out with CRLF line endings unless told
// otherwise, some Windows editors save a byte order mark, and the directives,
// the anchors, the headings and the recorded output are all read line by line.

import { readFileSync } from 'node:fs';

/** The file's text, without a leading byte order mark, and with every CRLF line ending turned into LF. */
export function readText(path: string): string {
	return readFileSync(path, 'utf8').replace(/^\uFEFF/, '').replace(/\r\n/g, '\n');
}
