# The Kotlin book

The Kotlin book: its [chapters](chapters/README.md), and the examples, tested programs and compile errors they show, laid out as [FORMAT.md](../FORMAT.md) describes. They depend on Itinera's artifacts as [build.gradle.kts](build.gradle.kts) names them: on `main`, the latest release from Maven Central, named exactly; on `next`, a commit of [itinera-j](https://github.com/itinera-dev/itinera-j), the one [itinera.json](https://github.com/itinera-dev/book/blob/next/kotlin/itinera.json) names, as [BOOK.md](../BOOK.md#how-the-examples-are-checked) explains.

Everything here runs with Gradle, through the wrapper, `./gradlew`, which fetches Gradle and every library at the versions the build pins: the plugins in [build.gradle.kts](build.gradle.kts), the libraries in [gradle.lockfile](gradle.lockfile). Gradle needs a JDK 17 or later to start.

## Getting itinera

On `main`, Gradle fetches itinera from Maven Central like any other dependency. On `next`, `./gradlew itinera` publishes it first:

1. It fetches the commit that `itinera.json` names into `.itinera/source`, and has that checkout's own build publish every artifact, as Maven Central would receive them, at the version `next`. itinera-j's build needs a JDK 25, which Gradle downloads when none is installed.
2. It copies them into `.itinera/repository`, where [settings.gradle.kts](settings.gradle.kts) has Gradle look for itinera, and only there.
3. It does nothing when `.itinera/repository` already comes from that commit, so run it once, and again whenever `itinera.json` changes. On `main`, where there is no `itinera.json`, it does nothing at all.

With `ITINERA_J` set to a checkout of itinera-j, `./gradlew itinera` publishes that checkout as it stands instead, every time it runs, so the book can run against code not pushed yet: `ITINERA_J=../../itinera-j ./gradlew itinera`.

## Running the examples

From this directory, after `./gradlew itinera`:

- `./gradlew -q example -Pname=NAME` runs one example, and what it prints must match `expected/NAME.txt`. Each example declares the package named after it, `package NAME`, so that the classes of one example never meet another's, and its `main` function.
- `./gradlew -q tests -Pname=NAME` runs the tests of one tested program, and the report it prints must match `tested/NAME.txt`.
- `./gradlew check` runs everything CI runs, as described below.

`./gradlew check` builds every example and every program in `tested/` as it would ship, without the tests in `tested/NAME.test.kt` and without `itinera-testing`, so a program that uses test support outside its tests does not compile. Every program is compiled for JDK 17, the oldest itinera runs on, with every compiler warning an error, and with itinera's checker run by KSP, as an application adds it. Then it checks that:

1. every example prints its recorded output, run in a JVM of its own. An event's timestamp and a generated journey ID change from one run to the next, so they are compared as placeholders, and the recorded output keeps the values of the run that recorded it;
2. the tests of every tested program, built with `itinera-testing`, print their recorded report. They run with JUnit's console launcher, each class's tests in the order of their names, and the time the run took is a placeholder;
3. every program in `compile-errors/`, compiled on its own with KSP and itinera's checker, is refused with the message recorded next to it: every error, one to a line, with the program's path as `compile-errors/NAME.kt`. Compiler messages change between Kotlin releases, so [build.gradle.kts](build.gradle.kts) pins the compiler exactly; moving to a newer one means recording them again;
4. every program and check is formatted as ktfmt's Kotlin style formats it, which `./gradlew spotlessApply` does.

To record an example's output, run `./gradlew -q example -Pname=NAME > expected/NAME.txt`, and a tested program's report, `./gradlew -q tests -Pname=NAME > tested/NAME.txt`. A compile error's message is written to `build/compile-errors/NAME.txt` each time `./gradlew check` runs, from where it can be copied. When the build's dependencies change, `./gradlew dependencies --write-locks` writes [gradle.lockfile](gradle.lockfile) again.
