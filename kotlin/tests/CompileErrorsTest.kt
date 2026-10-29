// Every program in `compile-errors/` does not compile, and the compiler says what the book shows:
// the message recorded next to it.
//
// Each program is compiled on its own, against itinera and the libraries the examples use, by the
// Kotlin compiler that builds the examples, with itinera's checker running in KSP, as it runs on
// every example. The message is every error the compiler and the checker give, one per line,
// without warnings. The program's path in it is written as a reader of the book sees it,
// `compile-errors/NAME.kt`, never as a path on the machine that compiled it. Each message is also
// written to `build/compile-errors/NAME.txt`, from where it can be recorded.

@file:OptIn(ExperimentalCompilerApi::class)

package book

import com.google.devtools.ksp.processing.SymbolProcessorProvider
import com.tschuchort.compiletesting.KotlinCompilation
import com.tschuchort.compiletesting.SourceFile
import com.tschuchort.compiletesting.configureKsp
import java.io.File
import java.io.OutputStream
import java.util.ServiceLoader
import kotlin.io.path.createDirectories
import kotlin.io.path.writeText
import org.jetbrains.kotlin.compiler.plugin.ExperimentalCompilerApi
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotEquals
import org.junit.jupiter.api.Assumptions.assumeFalse
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.DynamicTest.dynamicTest
import org.junit.jupiter.api.TestFactory

class CompileErrorsTest {
    @TestFactory
    fun `every compile error gives its recorded message`(): List<DynamicTest> {
        val programs = names("compile-errors", ".kt")
        val messages = names("compile-errors", ".txt")
        assumeFalse(programs.isEmpty() && messages.isEmpty(), "the book has no compile errors yet")
        assertEquals(
            programs,
            messages,
            "every program in compile-errors/ has its message, and every message its program",
        )
        return programs.map { name -> dynamicTest(name) { givesItsMessage(name) } }
    }

    private fun givesItsMessage(name: String) {
        val program = book.resolve("compile-errors/$name.kt").toFile()
        val compilation =
            KotlinCompilation().apply {
                sources = listOf(SourceFile.kotlin(program.name, program.readText()))
                classpaths = classPath("itinera").split(File.pathSeparator).map(::File)
                jvmTarget = "17"
                messageOutputStream = OutputStream.nullOutputStream()
                configureKsp { symbolProcessorProviders += checker() }
            }
        val result = compilation.compile()
        assertNotEquals(KotlinCompilation.ExitCode.OK, result.exitCode) { "$name compiles" }
        assertNotEquals(KotlinCompilation.ExitCode.INTERNAL_ERROR, result.exitCode, result.messages)
        val message = errorsOf(result.messages, name).joinToString("\n")
        val given = book.resolve("build/compile-errors").createDirectories()
        given.resolve("$name.txt").writeText("$message\n")
        assertEquals(read("compile-errors/$name.txt").trimEnd(), message)
    }

    private fun checker(): List<SymbolProcessorProvider> =
        ServiceLoader.load(SymbolProcessorProvider::class.java).toList()
}

/**
 * The errors among the compiler's [messages], each as the book shows it. A message starts with its
 * severity, `e:` for an error, and may go on over several lines.
 */
private fun errorsOf(messages: String, name: String): List<String> {
    val path = Regex("""(?:file://)?\S*/${Regex.escape(name)}\.kt""")
    return messages
        .split(Regex("""\n(?=[ewiv]: )"""))
        .filter { it.startsWith("e: ") }
        .map { it.trimEnd().replace(path, "compile-errors/$name.kt") }
}
