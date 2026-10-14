//! Every program in `tested/` builds as it would ship, without test support, and its tests print
//! the report the book shows: the one recorded next to it.
//!
//! Each program is a binary of this package. Cargo builds a binary without the dev-dependencies,
//! so the `testing` feature of `itinera` is off unless the binary's tests are being built: code
//! outside `#[cfg(test)]` that uses it fails the first test. The tests run one at a time, in the
//! order of their names, and the time the run took is the one placeholder of the report.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;
use snapbox::{Assert, Data, Redactions};

fn tested() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tested")
}

/// The names of the programs in `tested/`, in order.
fn programs() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(tested())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .map(|path| path.file_stem().unwrap().to_str().unwrap().to_owned())
        .collect();
    names.sort();
    assert!(!names.is_empty());
    names
}

#[test]
fn every_tested_program_builds_without_test_support() {
    let built = Command::new(env!("CARGO"))
        .args(["build", "--bins", "--quiet"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .unwrap();
    assert!(
        built.success(),
        "a tested program uses test support outside its tests"
    );
}

/// What running the tests of the program `name` prints, without the blank line the test runner
/// starts with.
fn report(name: &str) -> String {
    let output = Command::new(env!("CARGO"))
        .args(["test", "--bin", name, "--", "--test-threads=1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(output.status.success(), "the tests of {name} failed");
    String::from_utf8(output.stdout)
        .unwrap()
        .trim_start_matches('\n')
        .to_owned()
}

#[test]
fn every_tested_program_reports_its_recorded_test_run() {
    let mut changing = Redactions::new();
    changing
        .insert(
            "[DURATION]",
            Regex::new(r"finished in (?<redacted>\d+\.\d+s)").unwrap(),
        )
        .unwrap();
    let exactly = Assert::new().normalize_paths(false);
    let mut differences = Vec::new();
    for name in programs() {
        let printed = report(&name);
        let shown = fs::read_to_string(tested().join(&name).with_extension("txt")).unwrap();
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
