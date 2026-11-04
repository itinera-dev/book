// Every example prints what the book shows: the output recorded for it in `expected/`.
//
// Gradle builds the examples first, so they are checked as they are now. Each runs in a JVM of its
// own. What changes from one run to the next is a placeholder on both sides before they are
// compared (see `withoutChanges`); everything else is compared exactly.

package book;

import static book.Book.java;
import static book.Book.mainClassOf;
import static book.Book.names;
import static book.Book.read;
import static book.Book.setting;
import static book.Book.withoutChanges;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assumptions.assumeFalse;
import static org.junit.jupiter.api.DynamicTest.dynamicTest;

import java.io.IOException;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

class ExamplesTest {
  @TestFactory
  @DisplayName("every example prints its recorded output")
  List<DynamicTest> everyExamplePrintsItsRecordedOutput() {
    var programs = names("examples", ".java");
    var recorded = names("expected", ".txt");
    assumeFalse(programs.isEmpty() && recorded.isEmpty(), "the book has no examples yet");
    assertEquals(
        programs,
        recorded,
        "every example has its output in expected/, and every output in expected/ its example");
    return programs.stream().map(name -> dynamicTest(name, () -> printsItsOutput(name))).toList();
  }

  private static void printsItsOutput(String name) throws IOException, InterruptedException {
    var ran = java(List.of("-cp", setting("programs"), mainClassOf(name)));
    assertEquals(0, ran.status(), () -> name + " failed:\n" + ran.stderr());
    assertEquals(withoutChanges(read("expected/" + name + ".txt")), withoutChanges(ran.stdout()));
  }
}
