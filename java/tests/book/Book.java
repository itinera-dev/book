// What the checks of the book share: where its files are, which programs it holds, how to run a
// program, and what changes from one run of a program to the next.

package book;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.TimeUnit;
import java.util.regex.Pattern;
import java.util.stream.Stream;

final class Book {
  private Book() {}

  /** The book's {@code java} directory. */
  static final Path BOOK = Path.of(System.getProperty("book.directory"));

  /** A setting the build gives the checks, such as a class path, by its system property's name. */
  static String setting(String name) {
    return System.getProperty("book." + name);
  }

  /**
   * The names of the files in one of the book's directories ending with {@code suffix}, without it,
   * in order.
   */
  static List<String> names(String directory, String suffix) {
    var path = BOOK.resolve(directory);
    if (!Files.exists(path)) return List.of();
    try (Stream<Path> files = Files.list(path)) {
      return files
          .map(file -> file.getFileName().toString())
          .filter(name -> name.endsWith(suffix))
          .map(name -> name.substring(0, name.length() - suffix.length()))
          .sorted()
          .toList();
    } catch (IOException error) {
      throw new UncheckedIOException(error);
    }
  }

  /** A file of the book, as text. */
  static String read(String path) {
    try {
      return Files.readString(BOOK.resolve(path));
    } catch (IOException error) {
      throw new UncheckedIOException(error);
    }
  }

  /** What a program printed, and how it ended. */
  record Ran(int status, String stdout, String stderr) {}

  /**
   * Runs a fresh JVM, the one running the checks, with {@code arguments}, in the book's directory,
   * and waits for it to end.
   */
  static Ran java(List<String> arguments) throws IOException, InterruptedException {
    var java = Path.of(System.getProperty("java.home"), "bin", "java").toString();
    var output = Files.createTempFile("itinera-book-", ".txt");
    var errors = Files.createTempFile("itinera-book-", ".txt");
    try {
      var command = Stream.concat(Stream.of(java), arguments.stream()).toList();
      var process =
          new ProcessBuilder(command)
              .directory(BOOK.toFile())
              .redirectOutput(output.toFile())
              .redirectError(errors.toFile())
              .start();
      if (!process.waitFor(5, TimeUnit.MINUTES)) {
        process.destroyForcibly();
        throw new IllegalStateException(
            "java " + arguments.get(arguments.size() - 1) + " took more than five minutes");
      }
      return new Ran(process.exitValue(), Files.readString(output), Files.readString(errors));
    } finally {
      Files.delete(output);
      Files.delete(errors);
    }
  }

  /** The class holding the {@code main} method of the program {@code name}. */
  static String mainClassOf(String name) {
    return name + ".Main";
  }

  /**
   * What changes from one run of a program to the next: an event's timestamp, an instant in ISO
   * 8601 in UTC, and a generated journey ID, a random UUID. The book's own data holds neither.
   */
  private static final Pattern TIMESTAMP =
      Pattern.compile("\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}(?:\\.\\d+)?Z");

  private static final Pattern JOURNEY_ID =
      Pattern.compile("[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}");

  /** What also changes from one run of a program's tests to the next: how long they took. */
  private static final Pattern DURATION = Pattern.compile("Test run finished after \\d+ ms");

  /**
   * What a program printed, with what changes from one run to the next replaced by a placeholder.
   * The recorded output keeps the values of the run that recorded it, so the book shows a real run.
   */
  static String withoutChanges(String printed) {
    var timeless = TIMESTAMP.matcher(printed).replaceAll("[TIMESTAMP]");
    return JOURNEY_ID.matcher(timeless).replaceAll("[JOURNEY_ID]");
  }

  /** What a program's tests printed, with how long they took a placeholder too. */
  static String reportWithoutChanges(String printed) {
    return DURATION
        .matcher(withoutChanges(printed))
        .replaceAll("Test run finished after [DURATION]");
  }
}
