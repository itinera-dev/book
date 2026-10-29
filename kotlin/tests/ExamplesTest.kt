// Every example prints what the book shows: the output recorded for it in `expected/`.
//
// Gradle builds the examples first, so they are checked as they are now. Each runs in a JVM of its
// own. What changes from one run to the next is a placeholder on both sides before they are
// compared (see `withoutChanges`); everything else is compared exactly.

package book

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assumptions.assumeFalse
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.DynamicTest.dynamicTest
import org.junit.jupiter.api.TestFactory

class ExamplesTest {
    @TestFactory
    fun `every example prints its recorded output`(): List<DynamicTest> {
        val programs = names("examples", ".kt")
        val recorded = names("expected", ".txt")
        assumeFalse(programs.isEmpty() && recorded.isEmpty(), "the book has no examples yet")
        assertEquals(
            programs,
            recorded,
            "every example has its output in expected/, and every output in expected/ its example",
        )
        return programs.map { name -> dynamicTest(name) { printsItsOutput(name) } }
    }

    private fun printsItsOutput(name: String) {
        val ran = java("-cp", classPath("programs"), mainClassOf(name))
        assertEquals(0, ran.status) { "$name failed:\n${ran.stderr}" }
        assertEquals(
            withoutChanges(read("expected/$name.txt")),
            withoutChanges(ran.stdout),
        )
    }
}
