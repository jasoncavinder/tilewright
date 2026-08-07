// SPDX-License-Identifier: MPL-2.0

use serde_json::Value;
use std::fs::{self, File};
use std::path::PathBuf;
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

fn write_map_project(temp: &TempDir, map_infos: &[u8], map_documents: &[u32]) -> PathBuf {
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    fs::write(root.join("data/MapInfos.json"), map_infos).unwrap();
    for id in map_documents {
        fs::write(root.join(format!("data/Map{id:03}.json")), b"{}").unwrap();
    }
    root
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
    let data_path = PathBuf::from("data");
    assert!(stdout.contains(&format!(
        "{} [file] (known (standard data file))",
        data_path.join("Actors.json").display()
    )));
    assert!(stdout.contains(&format!(
        "{} [file] (known (map data file))",
        data_path.join("Map001.json").display()
    )));
    assert!(stdout.contains(&format!(
        "{} [file] (extension candidate (data json))",
        data_path.join("PluginData.json").display()
    )));
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

#[test]
fn help_lists_snapshot_and_resource_limits() {
    let help = tilewright(&["--help"]);
    assert!(help.status.success());
    assert!(stdout(&help).contains("snapshot"));

    let snapshot_help = tilewright(&["snapshot", "--help"]);
    assert!(snapshot_help.status.success());
    let out = stdout(&snapshot_help);
    assert!(out.contains("--format"));
    assert!(out.contains("--max-documents"));
    assert!(out.contains("--max-bytes-per-document"));
    assert!(out.contains("--max-aggregate-bytes"));
}

#[test]
fn snapshot_reports_loaded_documents_for_people() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    fs::create_dir_all(root.join("nested")).unwrap();
    fs::write(root.join("package.json"), r#"{"name":"synthetic"}"#).unwrap();
    fs::write(
        root.join("data/System.json"),
        r#"{"gameTitle":"Synthetic"}"#,
    )
    .unwrap();
    fs::write(root.join("data/Map001.json"), r#"{"events":[]}"#).unwrap();
    fs::write(root.join("data/PluginData.json"), r#"{"unknown":true}"#).unwrap();
    fs::write(root.join("nested/ignored.json"), r#"{"ignored":true}"#).unwrap();

    let output = tilewright(&["snapshot", root.to_str().unwrap()]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let stdout = stdout(&output);
    assert!(stdout.contains("Snapshot for"));
    assert!(stdout.contains("Completeness: complete"));
    assert!(stdout.contains("Loaded documents: 4"));
    assert!(stdout.contains("Diagnostics: 0"));
    assert!(
        stdout.contains(
            &PathBuf::from("data")
                .join("Map001.json")
                .display()
                .to_string()
        )
    );
    assert!(
        stdout.contains(
            &PathBuf::from("data")
                .join("PluginData.json")
                .display()
                .to_string()
        )
    );
    assert!(stdout.contains("package.json"));
    assert!(
        !stdout.contains(
            &PathBuf::from("nested")
                .join("ignored.json")
                .display()
                .to_string()
        )
    );
    assert!(!stdout.contains("Synthetic"));
}

#[test]
fn snapshot_emits_deterministic_versioned_json_without_source_contents() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    fs::write(root.join("package.json"), r#"{"secret":"package-secret"}"#).unwrap();
    fs::write(root.join("data/Actors.json"), r#"["actors-secret"]"#).unwrap();
    fs::write(root.join("data/Map001.json"), r#"{"secret":"map-secret"}"#).unwrap();

    let output = tilewright(&["snapshot", root.to_str().unwrap(), "--format", "json"]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["completeness"], "complete");
    assert_eq!(report["loaded_document_count"], 3);
    assert_eq!(report["diagnostic_count"], 0);
    assert_eq!(report["limits"]["max_documents"], 1_024);
    assert_eq!(report["limits"]["max_bytes_per_document"], 16_777_216);
    assert_eq!(report["limits"]["max_aggregate_bytes"], 268_435_456);

    let paths: Vec<_> = report["documents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|document| document["path"]["utf8"].as_str().unwrap())
        .collect();
    let expected_paths = [
        PathBuf::from("data").join("Actors.json"),
        PathBuf::from("data").join("Map001.json"),
        PathBuf::from("package.json"),
    ];
    let expected_paths: Vec<_> = expected_paths
        .iter()
        .map(|path| path.to_str().unwrap())
        .collect();
    assert_eq!(paths, expected_paths);
    assert!(
        report["documents"]
            .as_array()
            .unwrap()
            .iter()
            .all(|document| document["byte_length"].is_number())
    );

    let output_text = stdout(&output);
    assert!(!output_text.contains("package-secret"));
    assert!(!output_text.contains("actors-secret"));
    assert!(!output_text.contains("map-secret"));
}

#[test]
fn snapshot_partial_results_and_limits_are_structured() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    fs::write(root.join("data/Actors.json"), r#"{"broken":}"#).unwrap();
    fs::write(root.join("data/Map001.json"), r#"{}"#).unwrap();
    fs::write(root.join("data/System.json"), r#"{"title":"too large"}"#).unwrap();

    let output = tilewright(&[
        "snapshot",
        root.to_str().unwrap(),
        "--format",
        "json",
        "--max-documents",
        "2",
        "--max-bytes-per-document",
        "16",
        "--max-aggregate-bytes",
        "64",
    ]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["completeness"], "partial");
    assert_eq!(report["loaded_document_count"], 1);
    assert_eq!(report["diagnostic_count"], 2);
    assert_eq!(report["limits"]["max_documents"], 2);

    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert_eq!(
        diagnostics[0]["path"]["utf8"],
        PathBuf::from("data").join("Actors.json").to_str().unwrap()
    );
    assert_eq!(diagnostics[0]["category"], "invalid_syntax");
    assert!(diagnostics[0]["byte_range"]["start"].is_number());
    assert_eq!(
        diagnostics[1]["path"]["utf8"],
        PathBuf::from("data").join("System.json").to_str().unwrap()
    );
    assert_eq!(diagnostics[1]["category"], "exceeds_document_count_limit");
}

#[test]
fn snapshot_distinguishes_parse_and_byte_limit_diagnostics() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    fs::write(root.join("data/Actors.json"), b"\xff").unwrap();
    fs::write(root.join("data/Map001.json"), b"\xef\xbb\xbf{}").unwrap();

    let parse_output = tilewright(&["snapshot", root.to_str().unwrap(), "--format", "json"]);
    assert!(parse_output.status.success());
    assert!(stderr(&parse_output).is_empty());
    let report: Value = serde_json::from_slice(&parse_output.stdout).unwrap();
    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics[0]["category"], "invalid_utf8");
    assert_eq!(diagnostics[1]["category"], "utf8_bom");

    fs::write(root.join("data/Actors.json"), r#"{"long":true}"#).unwrap();
    fs::remove_file(root.join("data/Map001.json")).unwrap();

    let document_limit = tilewright(&[
        "snapshot",
        root.to_str().unwrap(),
        "--format",
        "json",
        "--max-bytes-per-document",
        "2",
        "--max-aggregate-bytes",
        "64",
    ]);
    assert!(document_limit.status.success());
    let report: Value = serde_json::from_slice(&document_limit.stdout).unwrap();
    assert_eq!(
        report["diagnostics"][0]["category"],
        "exceeds_document_byte_limit"
    );

    let aggregate_limit = tilewright(&[
        "snapshot",
        root.to_str().unwrap(),
        "--format",
        "json",
        "--max-bytes-per-document",
        "64",
        "--max-aggregate-bytes",
        "2",
    ]);
    assert!(aggregate_limit.status.success());
    let report: Value = serde_json::from_slice(&aggregate_limit.stdout).unwrap();
    assert_eq!(
        report["diagnostics"][0]["category"],
        "exceeds_aggregate_byte_limit"
    );
}

#[test]
fn snapshot_rejects_zero_resource_limits() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_str().unwrap();

    for option in [
        "--max-documents",
        "--max-bytes-per-document",
        "--max-aggregate-bytes",
    ] {
        let output = tilewright(&["snapshot", root, option, "0"]);
        assert_eq!(output.status.code(), Some(2), "option: {option}");
        assert!(stdout(&output).is_empty(), "option: {option}");
        assert!(
            stderr(&output).contains("must be greater than zero"),
            "option: {option}"
        );
    }
}

#[test]
fn snapshot_operational_errors_have_human_and_json_forms() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("missing\n\x1b[31mdir");
    let missing_arg = missing.to_str().unwrap();

    let human = tilewright(&["snapshot", missing_arg]);
    assert_eq!(human.status.code(), Some(1));
    assert!(stdout(&human).is_empty());
    let human_error = stderr(&human);
    assert!(human_error.contains("error: failed to open project root"));
    assert!(!human_error.contains("\x1b[31m"));
    assert!(human_error.contains("missing\\n\\u{1b}[31mdir"));

    let json = tilewright(&["snapshot", missing_arg, "--format", "json"]);
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
fn snapshot_reports_candidate_symlinks_without_following() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    let outside = temp.path().join("outside.json");
    fs::write(&outside, r#"{"secret":"must-not-be-read"}"#).unwrap();
    symlink(&outside, root.join("data/Linked.json")).unwrap();

    let output = tilewright(&["snapshot", root.to_str().unwrap(), "--format", "json"]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["completeness"], "partial");
    assert_eq!(report["loaded_document_count"], 0);
    assert_eq!(
        report["diagnostics"][0]["category"],
        "unsupported_entry_kind"
    );
    assert!(!stdout(&output).contains("must-not-be-read"));
}

#[cfg(unix)]
#[test]
fn snapshot_escapes_controls_and_documents_root_symlink_limit() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let real_root = temp.path().join("real-root");
    fs::create_dir(&real_root).unwrap();
    fs::create_dir(real_root.join("data")).unwrap();
    let controlled_name = "Control\n\x1b[31m.json";
    fs::write(real_root.join("data").join(controlled_name), r#"{}"#).unwrap();

    let linked_root = temp.path().join("linked-root");
    symlink(&real_root, &linked_root).unwrap();
    let output = tilewright(&["snapshot", linked_root.to_str().unwrap()]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let stdout = stdout(&output);
    assert!(stdout.contains("Loaded documents: 1"));
    assert!(!stdout.contains("\x1b[31m"));
    assert!(!stdout.contains("Control\n.json"));
    assert!(stdout.contains("Control\\n\\u{1b}[31m.json"));
}

#[cfg(target_os = "linux")]
#[test]
fn snapshot_json_marks_non_utf8_paths_without_claiming_exact_text() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    let filename = OsString::from_vec(b"Plugin-\xff.json".to_vec());
    fs::write(root.join("data").join(filename), r#"{}"#).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tilewright"))
        .arg("snapshot")
        .arg(&root)
        .args(["--format", "json"])
        .output()
        .expect("tilewright CLI should run");

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["loaded_document_count"], 1);
    assert!(report["documents"][0]["path"]["utf8"].is_null());
    assert!(report["documents"][0]["path"]["display"].is_string());
}

#[test]
fn help_lists_maps_and_resource_limits() {
    let help = tilewright(&["--help"]);
    assert!(help.status.success());
    assert!(stdout(&help).contains("maps"));

    let maps_help = tilewright(&["maps", "--help"]);
    assert!(maps_help.status.success());
    let out = stdout(&maps_help);
    assert!(out.contains("--format"));
    assert!(out.contains("--max-documents"));
    assert!(out.contains("--max-bytes-per-document"));
    assert!(out.contains("--max-aggregate-bytes"));
}

#[test]
fn maps_reports_display_order_and_escapes_controls_for_people() {
    let temp = TempDir::new().unwrap();
    let root = write_map_project(
        &temp,
        br#"[null,{"id":1,"name":"Later\n\u001b[31m","order":2,"parentId":0},{"id":2,"name":"Child","order":1,"parentId":1}]"#,
        &[1, 2],
    );

    let output = tilewright(&["maps", root.to_str().unwrap()]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let out = stdout(&output);
    assert!(out.contains("Maps: 2"));
    assert!(out.contains("Findings: 0"));
    assert!(out.contains("2: Child (order 1, parent 1)"));
    assert!(out.contains("1: Later\\n\\u{1b}[31m (order 2, parent none)"));
    assert!(out.find("2: Child").unwrap() < out.find("1: Later").unwrap());
    assert!(!out.contains('\u{1b}'));
}

#[test]
fn maps_emits_deterministic_versioned_json_without_unprojected_contents() {
    let temp = TempDir::new().unwrap();
    let root = write_map_project(
        &temp,
        br#"[null,{"id":1,"name":"Later","order":2,"parentId":0,"secret":"catalog-secret"},{"id":2,"name":"Child","order":1,"parentId":1}]"#,
        &[1, 2],
    );
    fs::write(
        root.join("data/Actors.json"),
        r#"[{"secret":"actor-secret"}]"#,
    )
    .unwrap();
    fs::write(root.join("data/Map001.json"), r#"{"secret":"map-secret"}"#).unwrap();

    let output = tilewright(&["maps", root.to_str().unwrap(), "--format", "json"]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["snapshot_completeness"], "complete");
    assert_eq!(report["map_count"], 2);
    assert_eq!(report["finding_count"], 0);
    assert_eq!(report["snapshot_diagnostic_count"], 0);
    assert_eq!(report["limits"]["max_documents"], 1_024);
    assert_eq!(report["maps"][0]["id"], 2);
    assert_eq!(report["maps"][0]["name"], "Child");
    assert_eq!(report["maps"][0]["order"], 1);
    assert_eq!(report["maps"][0]["parent_id"], 1);
    assert_eq!(report["maps"][1]["id"], 1);
    assert!(report["maps"][1]["parent_id"].is_null());
    assert_eq!(report["findings"], Value::Array(Vec::new()));
    assert_eq!(report["snapshot_diagnostics"], Value::Array(Vec::new()));

    let output_text = stdout(&output);
    assert!(!output_text.contains("catalog-secret"));
    assert!(!output_text.contains("actor-secret"));
    assert!(!output_text.contains("map-secret"));
}

#[test]
fn maps_reports_contextual_findings_without_failing() {
    let temp = TempDir::new().unwrap();
    let root = write_map_project(
        &temp,
        br#"[null,{"id":1,"name":"One","order":1,"parentId":2},{"id":2,"name":"Two","order":1,"parentId":1},null]"#,
        &[0, 2, 3],
    );

    let output = tilewright(&["maps", root.to_str().unwrap(), "--format", "json"]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["map_count"], 2);
    assert_eq!(report["finding_count"], 5);
    let categories: Vec<_> = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["category"].as_str().unwrap())
        .collect();
    assert_eq!(
        categories,
        [
            "parent_cycle",
            "duplicate_order",
            "missing_map_document",
            "unrecognized_map_document_identity",
            "orphan_map_document",
        ]
    );
    assert_eq!(report["findings"][0]["map_ids"], serde_json::json!([1, 2]));
    assert_eq!(report["findings"][2]["map_id"], 1);
    assert_eq!(
        report["findings"][2]["expected_path"]["utf8"],
        PathBuf::from("data").join("Map001.json").to_str().unwrap()
    );
}

#[test]
fn maps_keeps_snapshot_diagnostics_separate_from_catalog_findings() {
    let temp = TempDir::new().unwrap();
    let root = write_map_project(
        &temp,
        br#"[null,{"id":1,"name":"One","order":1,"parentId":0}]"#,
        &[1],
    );
    fs::write(root.join("data/Actors.json"), br#"{"broken":}"#).unwrap();

    let output = tilewright(&["maps", root.to_str().unwrap(), "--format", "json"]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["snapshot_completeness"], "partial");
    assert_eq!(report["map_count"], 1);
    assert_eq!(report["finding_count"], 0);
    assert_eq!(report["snapshot_diagnostic_count"], 1);
    assert_eq!(
        report["snapshot_diagnostics"][0]["category"],
        "invalid_syntax"
    );
}

#[test]
fn maps_projection_errors_have_human_and_json_forms() {
    let temp = TempDir::new().unwrap();
    let missing_root = temp.path().join("missing-map-infos");
    fs::create_dir(&missing_root).unwrap();

    let human = tilewright(&["maps", missing_root.to_str().unwrap()]);
    assert_eq!(human.status.code(), Some(1));
    assert!(stdout(&human).is_empty());
    assert!(stderr(&human).contains("data/MapInfos.json is missing"));

    let json = tilewright(&["maps", missing_root.to_str().unwrap(), "--format", "json"]);
    assert_eq!(json.status.code(), Some(1));
    assert!(stderr(&json).is_empty());
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["error"]["category"], "missing_document");
    assert_eq!(report["snapshot_completeness"], "complete");

    let malformed = TempDir::new().unwrap();
    let malformed_root = write_map_project(&malformed, br#"{}"#, &[]);
    let json = tilewright(&["maps", malformed_root.to_str().unwrap(), "--format", "json"]);
    assert_eq!(json.status.code(), Some(1));
    assert!(stderr(&json).is_empty());
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["error"]["category"], "unexpected_root_kind");
    assert_eq!(report["error"]["actual_kind"], "object");
}

#[test]
fn maps_reports_unavailable_map_infos_with_bounded_snapshot_diagnostic() {
    let temp = TempDir::new().unwrap();
    let root = write_map_project(
        &temp,
        br#"[null,{"id":1,"name":"One","order":1,"parentId":0}]"#,
        &[1],
    );

    let output = tilewright(&[
        "maps",
        root.to_str().unwrap(),
        "--format",
        "json",
        "--max-bytes-per-document",
        "8",
    ]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["error"]["category"], "unavailable_document");
    assert_eq!(report["snapshot_completeness"], "partial");
    let map_infos_path = PathBuf::from("data").join("MapInfos.json");
    assert!(
        report["snapshot_diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |diagnostic| diagnostic["path"]["utf8"] == map_infos_path.to_str().unwrap()
                    && diagnostic["category"] == "exceeds_document_byte_limit"
            )
    );
}

#[test]
fn maps_operational_errors_preserve_stream_separation() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("missing\n\u{1b}[31mdir");
    let missing_arg = missing.to_str().unwrap();

    let human = tilewright(&["maps", missing_arg]);
    assert_eq!(human.status.code(), Some(1));
    assert!(stdout(&human).is_empty());
    assert!(!stderr(&human).contains('\u{1b}'));
    assert!(stderr(&human).contains("missing\\n\\u{1b}[31mdir"));

    let json = tilewright(&["maps", missing_arg, "--format", "json"]);
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
}

#[test]
fn help_lists_inspect_json() {
    let help = tilewright(&["--help"]);
    assert!(help.status.success());
    assert!(stdout(&help).contains("inspect-json"));

    let inspect_help = tilewright(&["inspect-json", "--help"]);
    assert!(inspect_help.status.success());
    let out = stdout(&inspect_help);
    assert!(out.contains("--format"));
    assert!(out.contains("--max-bytes"));
}

#[test]
fn inspect_json_accepts_compact_and_formatted_strict_json() {
    let temp = TempDir::new().unwrap();

    let compact = temp.path().join("compact.json");
    fs::write(&compact, r#"{"a":1,"b":"hello"}"#).unwrap();
    let out = tilewright(&["inspect-json", compact.to_str().unwrap()]);
    assert!(out.status.success());
    assert!(stderr(&out).is_empty());
    assert!(stdout(&out).contains("Strict JSON syntax accepted"));
    assert!(stdout(&out).contains("Byte-identical serialization: true"));

    let formatted = temp.path().join("formatted.json");
    fs::write(&formatted, "{\n  \"a\": 1,\n  \"b\": \"hello\"\n}\n").unwrap();
    let out = tilewright(&["inspect-json", formatted.to_str().unwrap()]);
    assert!(out.status.success());
    assert!(stderr(&out).is_empty());
    assert!(stdout(&out).contains("Strict JSON syntax accepted"));
    assert!(stdout(&out).contains("Byte-identical serialization: true"));
}

#[test]
fn inspect_json_accepts_duplicate_keys_and_unusual_lexemes() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("unusual.json");
    fs::write(&file, r#"{"a":1,"a":2,"b":1e2,"c":"\u0061"}"#).unwrap();

    let out = tilewright(&["inspect-json", file.to_str().unwrap()]);
    assert!(out.status.success());
    assert!(stderr(&out).is_empty());
    assert!(stdout(&out).contains("Strict JSON syntax accepted"));
    assert!(stdout(&out).contains("Byte-identical serialization: true"));
}

#[test]
fn inspect_json_invalid_syntax_returns_structured_diagnostic() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("invalid.json");
    fs::write(&file, r#"{"a":1,}"#).unwrap();

    let out = tilewright(&["inspect-json", file.to_str().unwrap(), "--format", "json"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).is_empty());
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["error"]["category"], "invalid_syntax");
    assert!(report["error"]["message"].is_string());
    assert!(report["error"]["byte_range"]["start"].is_number());
    assert!(report["error"]["byte_range"]["end"].is_number());
}

#[test]
fn inspect_json_refuses_comments_and_trailing_commas() {
    let temp = TempDir::new().unwrap();

    let comments = temp.path().join("comments.json");
    fs::write(&comments, r#"{"a":1}// comment"#).unwrap();
    let out = tilewright(&["inspect-json", comments.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).is_empty());
    assert!(stderr(&out).contains("error:"));

    let trailing = temp.path().join("trailing.json");
    fs::write(&trailing, r#"{"a":1,}"#).unwrap();
    let out = tilewright(&["inspect-json", trailing.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).is_empty());
    assert!(stderr(&out).contains("error:"));
}

#[test]
fn inspect_json_refuses_empty_input() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("empty.json");
    fs::write(&file, " \t\r\n").unwrap();

    let out = tilewright(&["inspect-json", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).is_empty());
    assert!(stderr(&out).contains("error:"));
}

#[test]
fn inspect_json_invalid_utf8_and_bom_are_distinct() {
    let temp = TempDir::new().unwrap();

    let invalid_utf8 = temp.path().join("invalid_utf8.json");
    fs::write(&invalid_utf8, b"{\"a\":\"\xff\"}").unwrap();
    let out = tilewright(&[
        "inspect-json",
        invalid_utf8.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).is_empty());
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["error"]["category"], "invalid_utf8");

    let bom = temp.path().join("bom.json");
    fs::write(&bom, b"\xef\xbb\xbf{\"a\":1}").unwrap();
    let out = tilewright(&["inspect-json", bom.to_str().unwrap(), "--format", "json"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).is_empty());
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["error"]["category"], "utf8_bom");
}

#[test]
fn inspect_json_missing_file_behavior() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("missing.json");

    let human = tilewright(&["inspect-json", missing.to_str().unwrap()]);
    assert_eq!(human.status.code(), Some(1));
    assert!(stdout(&human).is_empty());
    assert!(stderr(&human).contains("error:"));

    let json = tilewright(&[
        "inspect-json",
        missing.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(json.status.code(), Some(1));
    assert!(stderr(&json).is_empty());
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["error"]["category"], "io_error");
}

#[test]
fn inspect_json_max_bytes_boundary() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("data.json");
    let content = r#"{"a":1}"#; // 7 bytes
    fs::write(&file, content).unwrap();

    // Exact boundary
    let exact = tilewright(&["inspect-json", file.to_str().unwrap(), "--max-bytes", "7"]);
    assert!(exact.status.success());
    assert!(stderr(&exact).is_empty());

    // One byte over
    let over = tilewright(&["inspect-json", file.to_str().unwrap(), "--max-bytes", "6"]);
    assert_eq!(over.status.code(), Some(1));
    assert!(stdout(&over).is_empty());
    assert!(stderr(&over).contains("exceeds maximum size"));

    // Zero rejection
    let zero = tilewright(&["inspect-json", file.to_str().unwrap(), "--max-bytes", "0"]);
    assert_eq!(zero.status.code(), Some(2));
    assert!(stdout(&zero).is_empty());
    assert!(stderr(&zero).contains("max_bytes must be greater than 0"));

    // Overflow rejection
    let overflow = tilewright(&[
        "inspect-json",
        file.to_str().unwrap(),
        "--max-bytes",
        &usize::MAX.to_string(),
    ]);
    assert_eq!(overflow.status.code(), Some(2));
    assert!(stdout(&overflow).is_empty());
    assert!(stderr(&overflow).contains("max_bytes must be less than"));
}

#[cfg(unix)]
#[test]
fn inspect_json_escapes_terminal_controls_in_human_output() {
    let temp = TempDir::new().unwrap();
    let malicious_name = "test\n\x1b[31m.json";
    let file = temp.path().join(malicious_name);
    fs::write(&file, r#"{"a":1}"#).unwrap();

    let out = tilewright(&["inspect-json", file.to_str().unwrap()]);
    assert!(out.status.success());
    assert!(stderr(&out).is_empty());
    let stdout = stdout(&out);
    assert!(!stdout.contains("\x1b[31m"));
    assert!(!stdout.contains("test\n.json"));
    assert!(stdout.contains("test\\n\\u{1b}[31m.json"));
}

#[cfg(target_os = "linux")]
#[test]
fn inspect_json_non_utf8_paths() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = TempDir::new().unwrap();
    let invalid_name = OsString::from_vec(b"unknown-\xff.json".to_vec());
    let file = temp.path().join(&invalid_name);
    fs::write(&file, r#"{"a":1}"#).unwrap();

    let human = Command::new(env!("CARGO_BIN_EXE_tilewright"))
        .arg("inspect-json")
        .arg(&file)
        .output()
        .expect("tilewright CLI should run");
    assert!(human.status.success());

    let json = Command::new(env!("CARGO_BIN_EXE_tilewright"))
        .arg("inspect-json")
        .arg(&file)
        .args(["--format", "json"])
        .output()
        .expect("tilewright CLI should run");
    assert!(json.status.success());
    assert!(stderr(&json).is_empty());
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["strict_syntax_accepted"], true);
    assert!(report["path"]["utf8"].is_null());
    assert!(report["path"]["display"].is_string());
}

#[test]
fn inspect_json_json_output_contains_no_source_document_contents() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("data.json");
    fs::write(&file, r#"{"secret_key":"secret_value"}"#).unwrap();

    let out = tilewright(&["inspect-json", file.to_str().unwrap(), "--format", "json"]);
    assert!(out.status.success());
    assert!(stderr(&out).is_empty());
    let stdout = stdout(&out);
    assert!(!stdout.contains("secret_key"));
    assert!(!stdout.contains("secret_value"));
}
