# Installing Itinera

## What you want to do

The shop is a Kotlin program you already have, built with Gradle or Maven. You want to add Itinera to it, so that you can start writing the shop with it.

## The code

Itinera for Kotlin is a small family of artifacts, published under the group `org.itinera-dev`, all at one version. A Kotlin application adds two of them, as it adds any other dependency:

1. `itinera-bom`, the bill of materials, which names that one version for every artifact of the family;
2. `itinera-kotlin`, which is Itinera for Kotlin, and brings the core, `itinera`, with it.

With Gradle's Kotlin DSL, that is two lines under `dependencies`: `implementation(platform("org.itinera-dev:itinera-bom:VERSION"))`, then `implementation("org.itinera-dev:itinera-kotlin")`, with no version of its own, since the bill of materials gives it. With Maven, you import `itinera-bom` under `dependencyManagement`, with `<type>pom</type>` and `<scope>import</scope>`, then add `itinera-kotlin` under `dependencies`, again with no version.

`VERSION` is the release you install, and Itinera for Kotlin has none yet. From its first release, every artifact is on Maven Central, where Gradle and Maven find it as they find any other library. Until then, this book follows Itinera's code ahead of a release: its programs build against a commit of [itinera-j](https://github.com/itinera-dev/itinera-j), the repository Itinera for Kotlin lives in, as the [README of the book's `kotlin` directory](../README.md) says.

It needs JDK 17 or later, and Kotlin 2.3 or later.

That is all you need to start. `itinera-kotlin` is all a Kotlin application needs, and when a chapter needs another artifact of the family, it says how to add it.

There is no program in this chapter, so nothing is printed.

## How it works

1. **It is a library, and nothing else.** Itinera runs inside your program. There is no server to start, no database to create and no background process to keep running.
2. **The bill of materials keeps the family together.** You write the version once, on `itinera-bom`, and every artifact of the family you add takes the same one, so two of them can never come from different releases. When a release comes out, moving to it is one line.
3. **A Kotlin application adds `itinera-kotlin`, never the core alone.** The core is what every application on the JVM shares, and `itinera-kotlin` is what reads your classes as Kotlin declares them, such as which of their parameters and properties may be `null`, which plain Java can't tell. **Without `itinera-kotlin`, the core refuses the Kotlin classes you declare with Itinera**, so it is the one artifact you add besides the bill of materials, and the core comes along with it.
4. **Each concept has a package of its own.** Everything Itinera offers is under the package `org.itineradev.itinera`, in a package of its own for each of Itinera's concepts, named after the concept. Each chapter's program shows its imports, so you always see which package a name comes from.
5. **Each release says which specification it implements.** Itinera is defined once, by a [specification](https://github.com/itinera-dev/spec) that names no programming language, and each release of Itinera for Kotlin will say which version of it the release implements. Until a first release, features may be added and the API may still change.
6. **The book's programs are pinned.** Each of them is built against exactly the commit of itinera-j it was written for, so what you read is what that code does.

## Outcomes

We have shown that Itinera is a library your build adds, with nothing else to install or run.

| The usual way | With Itinera |
|---|---|
| A workflow engine usually runs as a service of its own, with its own storage and workers to deploy, which lets work outlive a crash of the program that started it. | Itinera is a library inside your program: two dependencies of your build are the whole installation. |
