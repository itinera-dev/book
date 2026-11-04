// The Java book's programs: its examples, tested programs and compile errors, and the checks that
// each prints, reports or refuses what the book shows. README.md says how to run them.

pluginManagement {
    repositories {
        gradlePluginPortal()
        mavenCentral()
    }
}

dependencyResolutionManagement {
    repositoriesMode = RepositoriesMode.FAIL_ON_PROJECT_REPOS
    repositories {
        mavenCentral()
    }
}

rootProject.name = "itinera-book-java"
