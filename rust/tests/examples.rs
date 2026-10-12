//! Every example prints what the book shows: the output recorded for it in `expected/`.
//!
//! The test builds the examples first, so it checks them as they are now. What changes from one
//! run to the next, an event's timestamp and a generated journey ID, is replaced by a placeholder
//! on both sides before they are compared, so the book shows the output of a real run. Nothing
//! else is a placeholder: the recorded output is compared exactly, without snapbox's wildcards.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;
use snapbox::{Assert, Data, Redactions};

/// What differs between two runs of the same example.
fn changing() -> Redactions {
    let mut redactions = Redactions::new();
    redactions
        .insert(
            "[TIMESTAMP]",
            Regex::new(r"Timestamp \{ instant: (?<redacted>SystemTime \{[^}]*\}) \}").unwrap(),
        )
        .unwrap();
    redactions
        .insert(
            "[JOURNEY_ID]",
            Regex::new(r#"JourneyId\("(?<redacted>[0-9a-f-]{36})"\)"#).unwrap(),
        )
        .unwrap();
    redactions
}

/// Builds every example, and returns where Cargo put them: next to the directory of this test's
/// own executable. `cargo test --test examples` builds only this test, so it cannot rely on Cargo
/// having built them.
fn examples() -> PathBuf {
    let built = Command::new(env!("CARGO"))
        .args(["build", "--examples", "--quiet"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .unwrap();
    assert!(built.success(), "the examples do not build");
    let test = std::env::current_exe().unwrap();
    test.parent().unwrap().parent().unwrap().join("examples")
}

fn run(example: &Path) -> String {
    let output = Command::new(example).output().unwrap();
    assert!(output.status.success(), "{} failed", example.display());
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn every_example_prints_its_recorded_output() {
    let changing = changing();
    let built = examples();
    let exactly = Assert::new().normalize_paths(false);
    let mut recorded: Vec<PathBuf> =
        fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("expected"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
    recorded.sort();
    assert!(!recorded.is_empty());
    let mut differences = Vec::new();
    for expected in recorded {
        let name = expected.file_stem().unwrap().to_str().unwrap().to_owned();
        let printed = run(&built
            .join(&name)
            .with_extension(std::env::consts::EXE_EXTENSION));
        let shown = fs::read_to_string(&expected).unwrap();
        if let Err(difference) = exactly.try_eq(
            Some(&name),
            Data::text(changing.redact(&printed)),
            Data::text(changing.redact(&shown)).raw(),
        ) {
            differences.push(difference.to_string());
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}
