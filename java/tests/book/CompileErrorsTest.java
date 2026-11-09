// Every program in `compile-errors/` does not compile, and the compiler says what the book shows:
// the message recorded next to it.
//
// Each program is compiled on its own, against itinera, by the javac running the checks, told what
// it is told for every example, with itinera's checker on its annotation processor path, as it is
// on every example's. The message is what javac prints for each error, with the line of the
// program it points to, without warnings and without javac's closing count of errors. The
// program's path in it is written as a reader of the book sees it, `compile-errors/NAME.java`,
// never as a path on the machine that compiled it. Each message is also written to
// `build/compile-errors/NAME.txt`, from where it can be recorded.

package book;

import static book.Book.BOOK;
import static book.Book.names;
import static book.Book.read;
import static book.Book.setting;
import static book.Book.withNewlines;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assumptions.assumeFalse;
import static org.junit.jupiter.api.DynamicTest.dynamicTest;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;
import java.util.regex.Pattern;
import javax.tools.ToolProvider;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

class CompileErrorsTest {
  @TestFactory
  @DisplayName("every compile error gives its recorded message")
  List<DynamicTest> everyCompileErrorGivesItsRecordedMessage() {
    var programs = names("compile-errors", ".java");
    var messages = names("compile-errors", ".txt");
    assumeFalse(programs.isEmpty() && messages.isEmpty(), "the book has no compile errors yet");
    assertEquals(
        programs,
        messages,
        "every program in compile-errors/ has its message, and every message its program");
    return programs.stream().map(name -> dynamicTest(name, () -> givesItsMessage(name))).toList();
  }

  private static void givesItsMessage(String name) throws IOException {
    var program = BOOK.resolve("compile-errors/" + name + ".java");
    var classes = Files.createTempDirectory("itinera-book-");
    var arguments = new ArrayList<>(List.of(setting("javac").split(" ")));
    arguments.addAll(
        List.of(
            "-classpath",
            setting("itinera"),
            "-processorpath",
            setting("checker"),
            "-d",
            classes.toString(),
            program.toString()));
    var printed = new ByteArrayOutputStream();
    var status =
        ToolProvider.getSystemJavaCompiler()
            .run(null, null, printed, arguments.toArray(String[]::new));
    assertNotEquals(0, status, () -> name + " compiles");
    var message =
        String.join(
            "\n",
            errorsOf(
                withNewlines(printed.toString(StandardCharsets.UTF_8)), program.toString(), name));
    var given = Files.createDirectories(BOOK.resolve("build/compile-errors"));
    Files.writeString(given.resolve(name + ".txt"), message + "\n");
    assertEquals(read("compile-errors/" + name + ".txt").stripTrailing(), message);
  }

  /**
   * The errors among what javac {@code printed}, each as the book shows it: a message starts with
   * the program's path and goes on over the lines that quote the program, up to the next message or
   * javac's closing count.
   */
  private static List<String> errorsOf(String printed, String path, String name) {
    var shown = "compile-errors/" + name + ".java";
    var messages = printed.replace(path, shown).split("\n(?=" + Pattern.quote(shown) + ":)");
    return List.of(messages).stream()
        .map(CompileErrorsTest::withoutCount)
        .filter(message -> message.startsWith(shown) && message.contains(": error: "))
        .toList();
  }

  private static final Pattern COUNT = Pattern.compile("\n\\d+ (?:errors?|warnings?)\\s*$");

  private static String withoutCount(String message) {
    return COUNT.matcher(message.stripTrailing()).replaceAll("").stripTrailing();
  }
}
