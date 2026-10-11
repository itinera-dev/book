// Reads a file of the book as text with Unix line endings. Git for Windows
// checks text out with CRLF line endings unless told otherwise, and the
// directives, the anchors and the recorded output are all read line by line.

import { readFileSync } from 'node:fs';

/** The file's text, with every CRLF line ending turned into LF. */
export function readText(path: string): string {
	return readFileSync(path, 'utf8').replace(/\r\n/g, '\n');
}
