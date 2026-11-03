import groovy.json.JsonSlurper
import javax.inject.Inject
import org.gradle.api.tasks.testing.logging.TestExceptionFormat

plugins {
    java
    id("com.diffplug.spotless") version "8.10.4"
}

// The version of itinera the programs use: on `next`, the one `./gradlew itinera` publishes.
val itinera = "next"

sourceSets {
    // Every example and tested program as it would ship: without the tests of the tested programs,
    // and without test support, so a program that uses it outside its tests does not compile.
    main {
        java.setSrcDirs(listOf("examples", "tested"))
        java.exclude("**/*.test.java")
        resources.setSrcDirs(emptyList<String>())
    }
    // The tested programs again, with their tests and test support.
    create("tested") {
        java.setSrcDirs(listOf("tested"))
        resources.setSrcDirs(emptyList<String>())
    }
    // The checks that every program prints, reports or refuses what the book shows.
    test {
        java.setSrcDirs(listOf("tests"))
        resources.setSrcDirs(emptyList<String>())
    }
}

val consoleLauncher = configurations.dependencyScope("consoleLauncher")
val launcher = configurations.resolvable("launcher") { extendsFrom(consoleLauncher.get()) }
val checker = configurations.dependencyScope("checker")
val checkerPath = configurations.resolvable("checkerPath") { extendsFrom(checker.get()) }

dependencies {
    implementation(platform("org.itinera-dev:itinera-bom:$itinera"))
    implementation("org.itinera-dev:itinera-java")
    annotationProcessor(platform("org.itinera-dev:itinera-bom:$itinera"))
    annotationProcessor("org.itinera-dev:itinera-checker")

    "testedImplementation"(platform("org.itinera-dev:itinera-bom:$itinera"))
    "testedImplementation"("org.itinera-dev:itinera-java")
    "testedImplementation"("org.itinera-dev:itinera-testing")
    "testedImplementation"(platform("org.junit:junit-bom:6.1.3"))
    "testedImplementation"("org.junit.jupiter:junit-jupiter")
    "testedAnnotationProcessor"(platform("org.itinera-dev:itinera-bom:$itinera"))
    "testedAnnotationProcessor"("org.itinera-dev:itinera-checker")
    "consoleLauncher"(platform("org.junit:junit-bom:6.1.3"))
    "consoleLauncher"("org.junit.platform:junit-platform-console")
    "checker"(platform("org.itinera-dev:itinera-bom:$itinera"))
    "checker"("org.itinera-dev:itinera-checker")

    testImplementation(platform("org.junit:junit-bom:6.1.3"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

dependencyLocking {
    lockAllConfigurations()
    ignoredDependencies.add("org.itinera-dev:*")
}

/**
 * How javac compiles every program: for the oldest JVM itinera runs on, with every warning an
 * error, and with the names of parameters kept, which itinera reads as the keys they ask for.
 */
val javacOptions = listOf("--release", "17", "-Xlint:all", "-Werror", "-parameters")

tasks.withType<JavaCompile>().configureEach {
    options.release = 17
    options.compilerArgs.addAll(javacOptions.drop(2))
}

spotless {
    java {
        target(
            "examples/**/*.java",
            "tested/**/*.java",
            "compile-errors/**/*.java",
            "tests/**/*.java",
        )
        googleJavaFormat("1.28.0")
    }
    kotlinGradle {
        target("*.gradle.kts")
        ktfmt("0.64").kotlinlangStyle()
    }
}

/**
 * How the tests of a tested program run: with JUnit's console launcher, which prints their report
 * without colours, each class's tests in the order of their names.
 */
val report =
    listOf(
        "execute",
        "--details=testfeed",
        "--disable-banner",
        "--disable-ansi-colors",
        "--config=junit.jupiter.testclass.order.default=org.junit.jupiter.api.ClassOrderer\$ClassName",
        "--config=junit.jupiter.testmethod.order.default=org.junit.jupiter.api.MethodOrderer\$MethodName",
    )

/**
 * What the checks read: each class path they run or compile a program with, and the book's files.
 */
abstract class Book : CommandLineArgumentProvider {
    /** The examples and the tested programs as they ship, and what they run with. */
    @get:Classpath abstract val programs: ConfigurableFileCollection

    /** The tested programs with their tests, and what those run with. */
    @get:Classpath abstract val tested: ConfigurableFileCollection

    /** JUnit's console launcher, which runs the tests of a tested program and prints its report. */
    @get:Classpath abstract val launcher: ConfigurableFileCollection

    /** What the console launcher is told, but for the tested program whose tests it runs. */
    @get:Input abstract val report: ListProperty<String>

    /** What a program compiles against: itinera and what it brings. */
    @get:Classpath abstract val itinera: ConfigurableFileCollection

    /** itinera's checker, which javac finds on the annotation processor path. */
    @get:Classpath abstract val checker: ConfigurableFileCollection

    /** What javac is told, as it is told for every program. */
    @get:Input abstract val javac: ListProperty<String>

    @get:Internal abstract val directory: DirectoryProperty

    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val recorded: ConfigurableFileCollection

    override fun asArguments(): List<String> =
        listOf(
            "-Dbook.directory=${directory.get().asFile}",
            "-Dbook.programs=${programs.asPath}",
            "-Dbook.tested=${tested.asPath}",
            "-Dbook.launcher=${launcher.asPath}",
            "-Dbook.report=${report.get().joinToString(" ")}",
            "-Dbook.itinera=${itinera.asPath}",
            "-Dbook.checker=${checker.asPath}",
            "-Dbook.javac=${javac.get().joinToString(" ")}",
        )
}

tasks.test {
    useJUnitPlatform()
    // The checks compile the compile errors with the javac of the JDK they run on, whose messages
    // change between JDK releases, so they always run on one.
    javaLauncher = javaToolchains.launcherFor { languageVersion = JavaLanguageVersion.of(25) }
    val book = objects.newInstance<Book>()
    book.programs.from(sourceSets.main.map { it.runtimeClasspath })
    book.tested.from(sourceSets.named("tested").map { it.runtimeClasspath })
    book.launcher.from(launcher.get())
    book.report = report
    book.itinera.from(configurations.compileClasspath)
    book.checker.from(checkerPath.get())
    book.javac = javacOptions
    book.directory = layout.projectDirectory
    book.recorded.from(
        fileTree("expected"),
        fileTree("tested") { include("*.txt") },
        fileTree("compile-errors"),
    )
    jvmArgumentProviders.add(book)
    testLogging {
        events("failed")
        exceptionFormat = TestExceptionFormat.FULL
    }
}

/**
 * Publishes itinera into `.itinera/repository`, where the build finds it: from the checkout of
 * itinera-j that `ITINERA_J` names, as it stands, or else from the commit that itinera.json names,
 * fetched into `.itinera/source`. It does nothing when the repository already holds that commit,
 * and when there is no itinera.json, since itinera then comes from Maven Central.
 */
abstract class PublishItinera
@Inject
constructor(private val exec: ExecOperations, private val files: FileSystemOperations) :
    DefaultTask() {
    @get:Internal abstract val pin: RegularFileProperty

    @get:Internal abstract val checkout: DirectoryProperty

    @get:Internal abstract val directory: DirectoryProperty

    @TaskAction
    fun publish() {
        val pinned = pin.get().asFile
        if (!pinned.exists()) {
            logger.lifecycle("There is no itinera.json, so itinera comes from Maven Central.")
            return
        }
        val pins = JsonSlurper().parse(pinned) as Map<*, *>
        val commit = pins["commit"] as String
        val repository = directory.dir("repository").get().asFile
        val published = repository.resolve("commit")
        val source =
            if (checkout.isPresent) {
                checkout.get().asFile
            } else {
                if (published.exists() && published.readText() == commit) {
                    logger.lifecycle("itinera is already published from $commit.")
                    return
                }
                fetched(pins["repository"] as String, commit)
            }
        val windows = System.getProperty("os.name").startsWith("Windows")
        exec.exec {
            commandLine(
                source.resolve(if (windows) "gradlew.bat" else "gradlew"),
                "--project-dir=$source",
                "--no-daemon",
                "--quiet",
                "-Pversion=next",
                "publishAllPublicationsToPackageRepository",
            )
        }
        files.delete { delete(repository) }
        files.copy {
            from(source.resolve("build/package")) { include("org/itinera-dev/*/next/**") }
            into(repository)
        }
        if (!checkout.isPresent) published.writeText(commit)
    }

    private fun fetched(repository: String, commit: String): File {
        val source = directory.dir("source").get().asFile
        if (!source.resolve(".git").exists()) git(source.parentFile, "init", "--quiet", source.name)
        git(source, "fetch", "--quiet", "--depth=1", repository, commit)
        git(source, "checkout", "--quiet", "--force", "FETCH_HEAD")
        return source
    }

    private fun git(directory: File, vararg arguments: String) {
        directory.mkdirs()
        exec.exec { commandLine("git", "-C", directory, *arguments) }
    }
}

tasks.register<PublishItinera>("itinera") {
    description = "Publishes the itinera that itinera.json names into .itinera/repository."
    pin = layout.projectDirectory.file("itinera.json")
    // A relative path is read from this directory, wherever Gradle runs.
    val here = layout.projectDirectory
    checkout = providers.environmentVariable("ITINERA_J").map { here.dir(it) }
    directory = layout.projectDirectory.dir(".itinera")
}

val program = providers.gradleProperty("name")

tasks.register<JavaExec>("example") {
    description = "Runs the example that -Pname=NAME names."
    classpath = sourceSets.main.get().runtimeClasspath
    // The program's class `Main`, in the package named after the program.
    mainClass = program.map { "$it.Main" }
}

tasks.register<JavaExec>("tests") {
    description = "Runs the tests of the tested program that -Pname=NAME names."
    classpath = sourceSets["tested"].runtimeClasspath + launcher.get()
    mainClass = "org.junit.platform.console.ConsoleLauncher"
    args(report)
    val selected = program.map { "--select-package=$it" }
    argumentProviders.add(CommandLineArgumentProvider { listOf(selected.get()) })
}
