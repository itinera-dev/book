//! Every program in `compile-errors/` fails to compile with the message the book shows: the one
//! recorded next to it.
//!
//! Each program is an example that only the `compile-errors` feature builds, so the test builds
//! them one at a time and keeps the compiler's errors, without Cargo's progress lines. The path of
//! `itinera`, whether in Cargo's checkouts or in a local checkout that replaces it, is written as a
//! path inside its repository, as the recorded messages write it. Nothing else is rewritten: the
//! message is compared exactly.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;
use serde_json::{Deserializer, Value};
use snapbox::{Assert, Data};

fn compile_errors() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("compile-errors")
}

/// The errors the compiler gives for the example `name`, as it renders them.
fn errors(name: &str, checkout: &Regex) -> String {
    let output = Command::new(env!("CARGO"))
        .args([
            "build",
            "--quiet",
            "--message-format=json",
            "--features=compile-errors",
        ])
        .args(["--example", name])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(!output.status.success(), "{name} compiles");
    let rendered: String = Deserializer::from_slice(&output.stdout)
        .into_iter::<Value>()
        .map(Result::unwrap)
        .filter(is_error)
        .filter_map(rendered)
        .collect();
    checkout
        .replace_all(&rendered, "itinera-rs/$crates")
        .into_owned()
}

fn is_error(message: &Value) -> bool {
    message["reason"] == "compiler-message" && message["message"]["level"] == "error"
}

fn rendered(message: Value) -> Option<String> {
    message["message"]["rendered"].as_str().map(str::to_owned)
}

#[test]
fn every_compile_error_gives_its_recorded_message() {
    let checkout = Regex::new(r"/\S*?/(?<crates>crates/itinera[a-z-]*/src/)").unwrap();
    let exactly = Assert::new().normalize_paths(false);
    let mut programs: Vec<PathBuf> = fs::read_dir(compile_errors())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .collect();
    programs.sort();
    assert!(!programs.is_empty());
    let mut differences = Vec::new();
    for program in programs {
        let name = program.file_stem().unwrap().to_str().unwrap().to_owned();
        let given = errors(&name, &checkout);
        let shown = fs::read_to_string(program.with_extension("txt")).unwrap();
        if let Err(difference) = exactly.try_eq(
            Some(&name),
            Data::text(given.trim_end().to_owned()),
            Data::text(shown.trim_end().to_owned()).raw(),
        ) {
            differences.push(difference.to_string());
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}
