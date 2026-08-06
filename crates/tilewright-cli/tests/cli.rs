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

#[test]
fn inventory_reports_entries_for_people() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    File::create(root.join("game.rmmzproject")).unwrap();
    File::create(root.join("data/Actors.json")).unwrap();
    File::create(root.join("data/Map001.json")).unwrap();
    File::create(root.join("data/PluginData.json")).unwrap();
    File::create(root.join("unknown.txt")).unwrap();

    let output = tilewright(&["inventory", root.to_str().unwrap()]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let stdout = stdout(&output);
    assert!(stdout.contains("Inventory for"));
    assert!(stdout.contains("data [dir] (known (standard root entry))"));
    assert!(stdout.contains("game.rmmzproject [file] (known (standard root entry))"));
    assert!(stdout.contains("data/Actors.json [file] (known (standard data file))"));
    assert!(stdout.contains("data/Map001.json [file] (known (map data file))"));
    assert!(stdout.contains("data/PluginData.json [file] (extension candidate (data json))"));
    assert!(stdout.contains("unknown.txt [file] (unknown)"));
}

#[test]
fn inventory_emits_versioned_json_for_scripts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    File::create(root.join("game.rmmzproject")).unwrap();

    let output = tilewright(&["inventory", root.to_str().unwrap(), "--format", "json"]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);

    let entries = report["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);

    assert_eq!(entries[0]["path"]["utf8"].as_str().unwrap(), "data");
    assert_eq!(entries[0]["kind"], "directory");
    assert_eq!(entries[0]["classification"]["type"], "known");
    assert_eq!(
        entries[0]["classification"]["family"],
        "standard_root_entry"
    );

    assert_eq!(
        entries[1]["path"]["utf8"].as_str().unwrap(),
        "game.rmmzproject"
    );
    assert_eq!(entries[1]["kind"], "file");
    assert_eq!(entries[1]["classification"]["type"], "known");
    assert_eq!(
        entries[1]["classification"]["family"],
        "standard_root_entry"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn inventory_refuses_non_utf8_paths_in_json() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    let invalid_name = OsString::from_vec(b"unknown-\xff".to_vec());
    File::create(root.join(&invalid_name)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tilewright"))
        .arg("inventory")
        .arg(root)
        .args(["--format", "json"])
        .output()
        .expect("tilewright CLI should run");

    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert!(
        report["error"]["message"].as_str().unwrap().contains(
            "JSON output cannot safely represent non-UTF-8 paths without lossy conversion"
        )
    );
}

#[cfg(target_os = "linux")]
#[test]
fn inventory_allows_non_utf8_paths_in_human_output() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    let invalid_name = OsString::from_vec(b"unknown-\xff".to_vec());
    File::create(root.join(&invalid_name)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tilewright"))
        .arg("inventory")
        .arg(root)
        .output()
        .expect("tilewright CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unknown-"));
    assert!(stdout.contains("[file] (unknown)"));
}

#[cfg(unix)]
#[test]
fn inventory_reports_symlinks_without_following() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    File::create(outside.join("must-not-appear")).unwrap();

    symlink(&outside, root.join("linked")).unwrap();

    let output = tilewright(&["inventory", root.to_str().unwrap()]);

    assert!(output.status.success());
    let stdout = stdout(&output);
    assert!(stdout.contains("linked [symlink] (unknown)"));
    assert!(
        !stdout.contains("must-not-appear"),
        "stdout was: {}",
        stdout
    );
}

#[cfg(unix)]
#[test]
fn inventory_escapes_terminal_controls_in_human_output() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let malicious_name = "test\n\x1b[31mdir";
    fs::create_dir(root.join(malicious_name)).unwrap();

    let output = tilewright(&["inventory", root.to_str().unwrap()]);

    assert!(output.status.success());
    let stdout = stdout(&output);
    assert!(!stdout.contains("\x1b[31m"));
    assert!(!stdout.contains("test\ndir"));
    assert!(stdout.contains("test\\n\\u{1b}[31mdir"));
}

#[test]
fn inventory_operational_errors_have_human_and_json_forms() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("missing\n\x1b[31mdir");
    let missing_arg = missing.to_str().unwrap();

    let human = tilewright(&["inventory", missing_arg]);
    assert_eq!(human.status.code(), Some(1));
    assert!(stdout(&human).is_empty());
    let stderr_str = stderr(&human);
    assert!(stderr_str.contains("error: failed to open project root"));
    assert!(!stderr_str.contains("\x1b[31m"));
    assert!(stderr_str.contains("missing\\n\\u{1b}[31mdir"));

    let json = tilewright(&["inventory", missing_arg, "--format", "json"]);
    assert_eq!(json.status.code(), Some(1));
    assert!(stderr(&json).is_empty());
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert!(
        report["error"]["message"]
            .as_str()
            .unwrap()
            .contains("failed to open project root")
    );
    assert!(report["error"]["cause"].is_string());
}

#[cfg(unix)]
#[test]
fn inventory_root_symlink_is_resolved_known_limitation() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let real_dir = temp.path().join("real_dir");
    fs::create_dir(&real_dir).unwrap();
    File::create(real_dir.join("game.rmmzproject")).unwrap();

    let symlink_dir = temp.path().join("symlink_dir");
    symlink(&real_dir, &symlink_dir).unwrap();

    let output = tilewright(&["inventory", symlink_dir.to_str().unwrap()]);

    assert!(output.status.success());
    let stdout = stdout(&output);
    assert!(stdout.contains("game.rmmzproject"));
}
