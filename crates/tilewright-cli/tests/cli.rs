// SPDX-License-Identifier: MPL-2.0

use serde_json::Value;
use std::fs::{self, File};
use std::process::{Command, Output};
use tempfile::TempDir;

fn tilewright(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tilewright"))
        .args(args)
        .output()
        .expect("tilewright CLI should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

#[test]
fn help_and_version_are_available() {
    let help = tilewright(&["--help"]);
    assert!(help.status.success());
    assert!(stdout(&help).contains("discover"));

    let version = tilewright(&["--version"]);
    assert!(version.status.success());
    assert_eq!(
        stdout(&version),
        format!("tilewright {}\n", tilewright::VERSION)
    );
}

#[test]
fn discover_reports_a_candidate_for_people() {
    let temp = TempDir::new().unwrap();
    let marker = temp.path().join("game.rmmzproject");
    File::create(&marker).unwrap();

    let output = tilewright(&["discover", temp.path().to_str().unwrap()]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let stdout = stdout(&output);
    assert!(stdout.contains("RPG Maker MZ candidate"));
    assert!(stdout.contains(&marker.display().to_string()));
    assert!(stdout.contains("regular file"));
}

#[test]
fn discover_emits_versioned_json_for_scripts() {
    let temp = TempDir::new().unwrap();
    let marker = temp.path().join("game.rmmzproject");
    File::create(&marker).unwrap();

    let output = tilewright(&[
        "discover",
        temp.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["result"], "candidate");
    assert_eq!(report["markers"][0]["kind"], "regular_file");
    assert_eq!(
        report["markers"][0]["path"]["utf8"].as_str(),
        marker.to_str()
    );
}

#[test]
fn no_marker_is_a_successful_negative_finding() {
    let temp = TempDir::new().unwrap();

    let output = tilewright(&["discover", temp.path().to_str().unwrap()]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("No RPG Maker MZ marker found"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn case_variant_is_distinct_in_json() {
    let temp = TempDir::new().unwrap();
    File::create(temp.path().join("Game.rmmzproject")).unwrap();

    let output = tilewright(&[
        "discover",
        temp.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["result"], "case_variant_candidate");
}

#[test]
fn directory_marker_is_reported_without_becoming_a_candidate() {
    let temp = TempDir::new().unwrap();
    fs::create_dir(temp.path().join("game.rmmzproject")).unwrap();

    let output = tilewright(&[
        "discover",
        temp.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["result"], "non_regular_marker");
    assert_eq!(report["markers"][0]["kind"], "directory");
}

#[cfg(target_os = "linux")]
#[test]
fn non_utf8_root_is_explicit_in_json() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = TempDir::new().unwrap();
    let root = temp
        .path()
        .join(OsString::from_vec(b"project\xff".to_vec()));
    fs::create_dir(&root).unwrap();
    File::create(root.join("game.rmmzproject")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tilewright"))
        .arg("discover")
        .arg(&root)
        .args(["--format", "json"])
        .output()
        .expect("tilewright CLI should run");

    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["result"], "candidate");
    assert!(report["root"]["utf8"].is_null());
    assert!(report["root"]["display"].is_string());
}

#[test]
fn operational_errors_have_human_and_json_forms() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("missing");
    let missing_arg = missing.to_str().unwrap();

    let human = tilewright(&["discover", missing_arg]);
    assert_eq!(human.status.code(), Some(1));
    assert!(stdout(&human).is_empty());
    assert!(stderr(&human).contains("error: failed to inspect root path"));
    assert!(stderr(&human).contains("caused by:"));

    let json = tilewright(&["discover", missing_arg, "--format", "json"]);
    assert_eq!(json.status.code(), Some(1));
    assert!(stderr(&json).is_empty());
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert!(
        report["error"]["message"]
            .as_str()
            .unwrap()
            .contains("failed to inspect root path")
    );
    assert!(report["error"]["cause"].is_string());
}
