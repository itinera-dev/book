// The tests of each program in `tested/` print the report the book shows: the one recorded next to
// it.
//
// A tested program `NAME.kt` keeps its tests beside it, in `NAME.test.kt`, as a Kotlin project
// keeps its tests apart from the code it ships, and only they may use itinera-testing. Gradle
// builds every example and tested program first without their tests and without test support, so a
// program that uses test support outside its tests does not compile. The tests run with JUnit's
// console launcher, in a JVM of their own, one program at a time, each class's tests in the order
// of their names, and how long the run took is the one placeholder of the report.

package book

import java.io.File
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assumptions.assumeFalse
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.DynamicTest.dynamicTest
import org.junit.jupiter.api.TestFactory

class TestedTest {
    @TestFactory
    fun `the tests of every tested program print their recorded report`(): List<DynamicTest> {
        val tested = names("tested", ".kt").filterNot { it.endsWith(".test") }
        val tests = names("tested", ".test.kt")
        val reports = names("tested", ".txt")
        assumeFalse(
            tested.isEmpty() && tests.isEmpty() && reports.isEmpty(),
            "the book has no tested programs yet",
        )
        assertEquals(
            tested,
            tests,
            "every tested program has its tests in NAME.test.kt, and every test file its program",
        )
        assertEquals(
            tested,
            reports,
            "every tested program has its report in NAME.txt, and every report its program",
        )
        return tested.map { name -> dynamicTest(name) { reportsItsTests(name) } }
    }

    private fun reportsItsTests(name: String) {
        val ran =
            java(
                "-cp",
                classPath("tested") + File.pathSeparator + classPath("launcher"),
                "org.junit.platform.console.ConsoleLauncher",
                *System.getProperty("book.report").split(" ").toTypedArray(),
                "--select-package=$name",
            )
        assertEquals(0, ran.status) { "the tests of $name failed:\n${ran.stdout}${ran.stderr}" }
        assertEquals(
            reportWithoutChanges(read("tested/$name.txt")),
            reportWithoutChanges(ran.stdout),
        )
    }
}
