// The tests of each program in `tested/` print the report the book shows: the one recorded next to
// it.
//
// A tested program `NAME.java` keeps its tests beside it, in `NAME.test.java`, as a Java project
// keeps its tests apart from the code it ships, and only they may use itinera-testing. Gradle
// builds every example and tested program first without their tests and without test support, so a
// program that uses test support outside its tests does not compile. The tests run with JUnit's
// console launcher, in a JVM of their own, one program at a time, each class's tests in the order
// of their names, and how long the run took is the one placeholder of the report.

package book;

import static book.Book.java;
import static book.Book.names;
import static book.Book.read;
import static book.Book.reportWithoutChanges;
import static book.Book.setting;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assumptions.assumeFalse;
import static org.junit.jupiter.api.DynamicTest.dynamicTest;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

class TestedTest {
  @TestFactory
  @DisplayName("the tests of every tested program print their recorded report")
  List<DynamicTest> theTestsOfEveryTestedProgramPrintTheirRecordedReport() {
    var tested = names("tested", ".java").stream().filter(name -> !name.endsWith(".test")).toList();
    var tests = names("tested", ".test.java");
    var reports = names("tested", ".txt");
    assumeFalse(
        tested.isEmpty() && tests.isEmpty() && reports.isEmpty(),
        "the book has no tested programs yet");
    assertEquals(
        tested,
        tests,
        "every tested program has its tests in NAME.test.java, and every test file its program");
    assertEquals(
        tested,
        reports,
        "every tested program has its report in NAME.txt, and every report its program");
    return tested.stream().map(name -> dynamicTest(name, () -> reportsItsTests(name))).toList();
  }

  private static void reportsItsTests(String name) throws IOException, InterruptedException {
    var arguments = new ArrayList<String>();
    arguments.add("-cp");
    arguments.add(setting("tested") + File.pathSeparator + setting("launcher"));
    arguments.add("org.junit.platform.console.ConsoleLauncher");
    arguments.addAll(List.of(setting("report").split(" ")));
    arguments.add("--select-package=" + name);
    var ran = java(arguments);
    assertEquals(
        0, ran.status(), () -> "the tests of " + name + " failed:\n" + ran.stdout() + ran.stderr());
    assertEquals(
        reportWithoutChanges(read("tested/" + name + ".txt")), reportWithoutChanges(ran.stdout()));
  }
}
