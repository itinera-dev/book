// The Kotlin book's programs: its examples, tested programs and compile errors, and the checks
// that each prints, reports or refuses what the book shows. README.md says how to run them.

pluginManagement {
    repositories {
        gradlePluginPortal()
        mavenCentral()
    }
}

dependencyResolutionManagement {
    repositoriesMode = RepositoriesMode.FAIL_ON_PROJECT_REPOS
    repositories {
        // itinera as `./gradlew itinera` publishes it, from the commit of itinera-j that
        // itinera.json names.
        maven(settingsDir.resolve(".itinera/repository")) {
            content { includeGroup("org.itinera-dev") }
        }
        mavenCentral { content { excludeGroup("org.itinera-dev") } }
    }
}

rootProject.name = "itinera-book-kotlin"
