# Installing Itinera

## What you want to do

The shop is a TypeScript program you already have. You want to add Itinera to it, so that you can start writing the shop with it.

## The code

Itinera for TypeScript is one npm package, `itinera`, and you add it as you add any other dependency, together with `reflect-metadata`, which it needs beside it. From the command line, that is `npm install itinera@0.1.0-beta.5 reflect-metadata`.

Until `0.1.0`, every version of `itinera` is a pre-release, published under npm's tag `next`, so `itinera@next` installs the newest one, which may be newer than the one this book is written for.

It needs Node 22.12 or later, and TypeScript 5.5 or later, with three compiler options turned on in your `tsconfig.json`: `"strict": true`, `"experimentalDecorators": true` and `"emitDecoratorMetadata": true`.

That is all you need to start. The package has no options to choose: everything it offers comes with it.

There is no program in this chapter, so nothing is printed.

## How it works

1. **It is a library, and nothing else.** Itinera runs inside your program. There is no server to start, no database to create and no background process to keep running.
2. **You declare things with decorators.** What Itinera runs is written as classes, and a decorator written above a class or one of its parameters tells Itinera what it is. TypeScript has two kinds of decorator, and `itinera` uses the older kind, which `experimentalDecorators` turns on. `emitDecoratorMetadata` has the compiler record the type of each parameter a decorator marks, and `reflect-metadata` is how `itinera` reads it back while your program runs. You never import `reflect-metadata` yourself: `itinera` does, and it is a dependency of yours only so that your program and `itinera` share one copy of it.
3. **Your compiler must record that metadata.** `tsc` does, and so does swc. Some build tools, such as esbuild, tsx and Vite, strip the types away without recording anything, so the decorators can't read them; the README that comes with `itinera` says what to do with one of those. The book's own programs are compiled with `tsc`.
4. **Each concept has a subpath of its own.** The package root exports nothing: each concept of Itinera is imported from a subpath named after it. Each chapter's program shows its imports, so you always see which subpath a name comes from. `itinera` works with `import` and `require` alike; the book's programs are ES modules.
5. **Each release says which specification it implements.** Itinera is defined once, by a [specification](https://github.com/itinera-dev/spec) that names no programming language, and each release of `itinera` says which version of it the release implements. This book is written for `0.1.0-beta.5`. It is a beta: until `0.1.0`, features may be added and the API may still change.
6. **You can pin it exactly.** Since a pre-release may still change its API, `npm install --save-exact itinera@0.1.0-beta.5` records exactly that version, so that a later install never chooses another. The book's own programs are pinned too, each built against exactly the code it was written for.

## Outcomes

We have shown that Itinera is one dependency of your program, with nothing else to install or run.

| The usual way | With Itinera |
|---|---|
| A workflow engine usually runs as a service of its own, with its own storage and workers to deploy, which lets work outlive a crash of the program that started it. | Itinera is a package inside your program: one install command and three compiler options are the whole installation. |
