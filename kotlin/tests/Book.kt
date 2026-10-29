// What the checks of the book share: where its files are, which programs it holds, how to run a
// program, and what changes from one run of a program to the next.

package book

import java.nio.file.Path
import java.util.concurrent.TimeUnit
import kotlin.io.path.Path
import kotlin.io.path.createTempFile
import kotlin.io.path.deleteExisting
import kotlin.io.path.exists
import kotlin.io.path.listDirectoryEntries
import kotlin.io.path.name
import kotlin.io.path.readText

/** The book's `kotlin` directory. */
val book: Path = Path(System.getProperty("book.directory"))

/** A class path the build gives the checks, by the name of its system property. */
fun classPath(name: String): String = System.getProperty("book.$name")

/**
 * The names of the files in one of the book's directories ending with [suffix], without it, in
 * order.
 */
fun names(directory: String, suffix: String): List<String> {
    val path = book.resolve(directory)
    if (!path.exists()) return emptyList()
    return path.listDirectoryEntries("*$suffix").map { it.name.removeSuffix(suffix) }.sorted()
}

/** A file of the book, as text. */
fun read(path: String): String = book.resolve(path).readText()

/** What a program printed, and how it ended. */
class Ran(val status: Int, val stdout: String, val stderr: String)

/**
 * Runs a fresh JVM, the one running the checks, with [arguments], in the book's directory, and
 * waits for it to end.
 */
fun java(vararg arguments: String): Ran {
    val java = Path(System.getProperty("java.home"), "bin", "java").toString()
    val errors = createTempFile("itinera-book-", ".txt")
    try {
        val process =
            ProcessBuilder(java, *arguments)
                .directory(book.toFile())
                .redirectError(errors.toFile())
                .start()
        val stdout = process.inputStream.bufferedReader().use { it.readText() }
        check(process.waitFor(5, TimeUnit.MINUTES)) { "java ${arguments.last()} took too long" }
        return Ran(process.exitValue(), stdout, errors.readText())
    } finally {
        errors.deleteExisting()
    }
}

/**
 * The class holding the `main` function of the program [name]: the program declares the package
 * named after it, and Kotlin names the class of its file after the file.
 */
fun mainClassOf(name: String): String = "$name.${name.replaceFirstChar(Char::uppercaseChar)}Kt"

/**
 * What changes from one run of a program to the next: an event's timestamp, an instant in ISO 8601
 * in UTC, and a generated journey ID, a random UUID. The book's own data holds neither.
 */
private val changing =
    listOf(
        Regex("""\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z""") to "[TIMESTAMP]",
        Regex("""[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}""") to "[JOURNEY_ID]",
    )

/** What also changes from one run of a program's tests to the next: how long they took. */
private val changingInAReport =
    changing + (Regex("""Test run finished after \d+ ms""") to "Test run finished after [DURATION]")

private fun replaced(printed: String, changes: List<Pair<Regex, String>>): String =
    changes.fold(printed) { text, (pattern, placeholder) -> text.replace(pattern, placeholder) }

/**
 * What a program printed, with what changes from one run to the next replaced by a placeholder. The
 * recorded output keeps the values of the run that recorded it, so the book shows a real run.
 */
fun withoutChanges(printed: String): String = replaced(printed, changing)

/** What a program's tests printed, with how long they took a placeholder too. */
fun reportWithoutChanges(printed: String): String = replaced(printed, changingInAReport)
