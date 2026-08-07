// SPDX-License-Identifier: MPL-2.0

//! Experimental, read-only RPG Maker MZ raw project snapshot loader.

use crate::json::{LosslessJsonDocument, LosslessJsonError};
use crate::rpg_maker_mz::inventory::{
    ExtensionCandidateFamily, InventoryClassification, InventoryEntryKind, KnownEntryFamily,
    ProjectInventory, inventory_project,
};
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

/// Limits for snapshot loading to prevent resource exhaustion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotLimits {
    /// Maximum number of documents to load.
    pub max_documents: NonZeroUsize,
    /// Maximum bytes to read for a single document.
    pub max_bytes_per_document: NonZeroUsize,
    /// Maximum total bytes to read across all documents.
    pub max_aggregate_bytes: NonZeroUsize,
}

/// A diagnostic for a document that could not be loaded.
#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentDiagnostic {
    /// The entry kind is not supported (e.g., symlink, directory).
    #[non_exhaustive]
    UnsupportedEntryKind { kind: InventoryEntryKind },
    /// Failed to open the file.
    #[non_exhaustive]
    Open { source: io::Error },
    /// Failed to read the file.
    #[non_exhaustive]
    Read { source: io::Error },
    /// The document exceeded the per-document byte limit.
    #[non_exhaustive]
    ExceedsDocumentByteLimit { limit: NonZeroUsize },
    /// The document was skipped because the aggregate byte limit was reached.
    #[non_exhaustive]
    ExceedsAggregateByteLimit { limit: NonZeroUsize },
    /// The document was skipped because the document count limit was reached.
    #[non_exhaustive]
    ExceedsDocumentCountLimit { limit: NonZeroUsize },
    /// Failed to parse the document as strict JSON.
    #[non_exhaustive]
    Parse { source: LosslessJsonError },
}

impl fmt::Display for DocumentDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedEntryKind { kind } => {
                write!(f, "unsupported entry kind: {kind:?}")
            }
            Self::Open { .. } => f.write_str("failed to open file"),
            Self::Read { .. } => f.write_str("failed to read file"),
            Self::ExceedsDocumentByteLimit { limit } => {
                write!(f, "exceeds per-document byte limit of {limit}")
            }
            Self::ExceedsAggregateByteLimit { limit } => {
                write!(f, "exceeds aggregate byte limit of {limit}")
            }
            Self::ExceedsDocumentCountLimit { limit } => {
                write!(f, "exceeds document count limit of {limit}")
            }
            Self::Parse { source } => write!(f, "parse error: {source}"),
        }
    }
}

impl Error for DocumentDiagnostic {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Open { source } | Self::Read { source } => Some(source),
            Self::Parse { source } => Some(source),
            _ => None,
        }
    }
}

/// The completeness of a project snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SnapshotCompleteness {
    /// All candidate documents were successfully loaded.
    Complete,
    /// One or more candidate documents failed to load or were skipped.
    Partial,
}

/// An experimental, read-only snapshot of RPG Maker MZ project data.
#[derive(Debug)]
#[non_exhaustive]
pub struct ProjectSnapshot {
    /// The inventory of the project directory.
    pub inventory: ProjectInventory,
    /// The successfully loaded documents, ordered by path.
    pub documents: BTreeMap<PathBuf, LosslessJsonDocument>,
    /// Diagnostics for candidate documents that could not be loaded.
    pub diagnostics: BTreeMap<PathBuf, DocumentDiagnostic>,
}

impl ProjectSnapshot {
    /// Returns the completeness of the snapshot.
    pub fn completeness(&self) -> SnapshotCompleteness {
        if self.diagnostics.is_empty() {
            SnapshotCompleteness::Complete
        } else {
            SnapshotCompleteness::Partial
        }
    }
}

/// Errors that can occur while loading a project snapshot.
#[derive(Debug)]
#[non_exhaustive]
pub enum SnapshotError {
    /// Failed to inventory the project directory.
    #[non_exhaustive]
    Inventory {
        source: crate::rpg_maker_mz::inventory::InventoryError,
    },
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inventory { .. } => f.write_str("failed to inventory project"),
        }
    }
}

impl Error for SnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Inventory { source } => Some(source),
        }
    }
}

/// Loads an experimental, read-only snapshot of RPG Maker MZ project data.
pub fn load_snapshot(root: &Dir, limits: SnapshotLimits) -> Result<ProjectSnapshot, SnapshotError> {
    let inventory =
        inventory_project(root).map_err(|source| SnapshotError::Inventory { source })?;

    let mut documents = BTreeMap::new();
    let mut diagnostics = BTreeMap::new();

    let mut aggregate_bytes = 0;
    let mut document_count = 0;

    for entry in &inventory.entries {
        if !is_candidate(&entry.path, &entry.classification) {
            continue;
        }

        if entry.kind != InventoryEntryKind::File {
            diagnostics.insert(
                entry.path.clone(),
                DocumentDiagnostic::UnsupportedEntryKind { kind: entry.kind },
            );
            continue;
        }

        if document_count >= limits.max_documents.get() {
            diagnostics.insert(
                entry.path.clone(),
                DocumentDiagnostic::ExceedsDocumentCountLimit {
                    limit: limits.max_documents,
                },
            );
            continue;
        }

        let mut options = OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);

        let mut file = match root.open_with(&entry.path, &options) {
            Ok(file) => file,
            Err(source) => {
                diagnostics.insert(entry.path.clone(), DocumentDiagnostic::Open { source });
                continue;
            }
        };

        let max_read = limits.max_bytes_per_document.get();
        let mut buffer = Vec::new();
        let mut handle = (&mut file).take((max_read + 1) as u64);

        if let Err(source) = handle.read_to_end(&mut buffer) {
            diagnostics.insert(entry.path.clone(), DocumentDiagnostic::Read { source });
            continue;
        }

        if buffer.len() > max_read {
            diagnostics.insert(
                entry.path.clone(),
                DocumentDiagnostic::ExceedsDocumentByteLimit {
                    limit: limits.max_bytes_per_document,
                },
            );
            continue;
        }

        if aggregate_bytes + buffer.len() > limits.max_aggregate_bytes.get() {
            diagnostics.insert(
                entry.path.clone(),
                DocumentDiagnostic::ExceedsAggregateByteLimit {
                    limit: limits.max_aggregate_bytes,
                },
            );
            continue;
        }

        match LosslessJsonDocument::parse(&buffer) {
            Ok(document) => {
                documents.insert(entry.path.clone(), document);
                aggregate_bytes += buffer.len();
                document_count += 1;
            }
            Err(source) => {
                diagnostics.insert(entry.path.clone(), DocumentDiagnostic::Parse { source });
            }
        }
    }

    Ok(ProjectSnapshot {
        inventory,
        documents,
        diagnostics,
    })
}

fn is_candidate(path: &Path, classification: &InventoryClassification) -> bool {
    if path == Path::new("package.json") {
        return true;
    }

    match classification {
        InventoryClassification::Known { family } => match family {
            KnownEntryFamily::StandardDataFile | KnownEntryFamily::MapDataFile => true,
            KnownEntryFamily::StandardRootEntry => false,
        },
        InventoryClassification::ExtensionCandidate { family } => match family {
            ExtensionCandidateFamily::DataJson => true,
        },
        InventoryClassification::Unknown => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cap_std::ambient_authority;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn open_root(path: &Path) -> Dir {
        Dir::open_ambient_dir(path, ambient_authority()).expect("open synthetic test root")
    }

    fn default_limits() -> SnapshotLimits {
        SnapshotLimits {
            max_documents: NonZeroUsize::new(100).unwrap(),
            max_bytes_per_document: NonZeroUsize::new(1024 * 1024).unwrap(),
            max_aggregate_bytes: NonZeroUsize::new(10 * 1024 * 1024).unwrap(),
        }
    }

    #[test]
    fn empty_project_and_non_json_files() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("notes.txt")).unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        File::create(temp.path().join("data/Actors.csv")).unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Complete);
        assert!(snapshot.documents.is_empty());
        assert!(snapshot.diagnostics.is_empty());
    }

    #[test]
    fn valid_package_and_standard_data() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), b"{\"name\":\"test\"}").unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Actors.json"), b"[]").unwrap();
        fs::write(temp.path().join("data/Map001.json"), b"{}").unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Complete);
        assert_eq!(snapshot.documents.len(), 3);
        assert!(snapshot.documents.contains_key(Path::new("package.json")));
        assert!(
            snapshot
                .documents
                .contains_key(Path::new("data/Actors.json"))
        );
        assert!(
            snapshot
                .documents
                .contains_key(Path::new("data/Map001.json"))
        );
    }

    #[test]
    fn valid_extension_candidate() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/PluginData.json"), b"{}").unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Complete);
        assert_eq!(snapshot.documents.len(), 1);
        assert!(
            snapshot
                .documents
                .contains_key(Path::new("data/PluginData.json"))
        );
    }

    #[test]
    fn unknown_nested_json_is_not_loaded() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("js/plugins")).unwrap();
        fs::write(temp.path().join("js/plugins/config.json"), b"{}").unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Complete);
        assert!(snapshot.documents.is_empty());
    }

    #[test]
    fn deterministic_native_path_order() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Zeta.json"), b"{}").unwrap();
        fs::write(temp.path().join("data/Alpha.json"), b"{}").unwrap();
        fs::write(temp.path().join("package.json"), b"{}").unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        let paths: Vec<_> = snapshot.documents.keys().collect();
        assert_eq!(
            paths,
            vec![
                &PathBuf::from("data/Alpha.json"),
                &PathBuf::from("data/Zeta.json"),
                &PathBuf::from("package.json"),
            ]
        );
    }

    #[test]
    fn parse_failures_become_diagnostics_while_others_succeed() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Actors.json"), b"[]").unwrap();
        fs::write(temp.path().join("data/Map001.json"), b"\xff").unwrap(); // Invalid UTF-8
        fs::write(temp.path().join("data/Map002.json"), b"\xef\xbb\xbf{}").unwrap(); // BOM
        fs::write(temp.path().join("data/Map003.json"), b"{").unwrap(); // Invalid syntax
        fs::write(temp.path().join("data/Map004.json"), b"").unwrap(); // Empty
        fs::write(temp.path().join("data/Map005.json"), b"{}// comment").unwrap(); // Comment
        fs::write(temp.path().join("data/Map006.json"), b"{\"a\":1,}").unwrap(); // Trailing comma

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Partial);
        assert_eq!(snapshot.documents.len(), 1);
        assert!(
            snapshot
                .documents
                .contains_key(Path::new("data/Actors.json"))
        );
        assert_eq!(snapshot.diagnostics.len(), 6);
        for i in 1..=6 {
            let path = PathBuf::from(format!("data/Map00{i}.json"));
            assert!(matches!(
                snapshot.diagnostics.get(&path).unwrap(),
                DocumentDiagnostic::Parse { .. }
            ));
        }
    }

    #[test]
    fn byte_identity_preserved() {
        let temp = TempDir::new().unwrap();
        let content = b"{\"a\":1,\"a\":2, \"b\": 1e+02, \"c\": \"\\n\"}";
        fs::write(temp.path().join("package.json"), content).unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        let doc = snapshot.documents.get(Path::new("package.json")).unwrap();
        assert_eq!(doc.source_bytes(), content);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn symlinks_are_not_opened() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        let target = temp.path().join("target.json");
        fs::write(&target, b"{}").unwrap();
        let link = temp.path().join("data/Actors.json");

        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &link).unwrap();

        #[cfg(windows)]
        if let Err(error) = std::os::windows::fs::symlink_file(&target, &link) {
            if error.kind() == io::ErrorKind::PermissionDenied {
                return;
            }
            panic!("create symlink: {error}");
        }

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Partial);
        assert!(snapshot.documents.is_empty());
        assert!(matches!(
            snapshot
                .diagnostics
                .get(Path::new("data/Actors.json"))
                .unwrap(),
            DocumentDiagnostic::UnsupportedEntryKind {
                kind: InventoryEntryKind::Symlink
            }
        ));
    }

    #[test]
    fn resource_limits() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Map001.json"), b"{}").unwrap(); // 2 bytes
        fs::write(temp.path().join("data/Map002.json"), b"{\"a\":1}").unwrap(); // 7 bytes
        fs::write(temp.path().join("data/Map003.json"), b"{}").unwrap(); // 2 bytes
        fs::write(temp.path().join("data/Map004.json"), b"{}").unwrap(); // 2 bytes

        let limits = SnapshotLimits {
            max_documents: NonZeroUsize::new(3).unwrap(),
            max_bytes_per_document: NonZeroUsize::new(5).unwrap(),
            max_aggregate_bytes: NonZeroUsize::new(4).unwrap(),
        };

        let snapshot = load_snapshot(&open_root(temp.path()), limits).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Partial);

        // Map001: 2 bytes, ok. Aggregate = 2. Count = 1.
        assert!(
            snapshot
                .documents
                .contains_key(Path::new("data/Map001.json"))
        );

        // Map002: 7 bytes > 5. ExceedsDocumentByteLimit.
        assert!(matches!(
            snapshot
                .diagnostics
                .get(Path::new("data/Map002.json"))
                .unwrap(),
            DocumentDiagnostic::ExceedsDocumentByteLimit { .. }
        ));

        // Map003: 2 bytes. Aggregate would be 4, which is <= 4. Ok. Aggregate = 4. Count = 2.
        assert!(
            snapshot
                .documents
                .contains_key(Path::new("data/Map003.json"))
        );

        // Map004: 2 bytes. Aggregate would be 6 > 4. ExceedsAggregateByteLimit.
        assert!(matches!(
            snapshot
                .diagnostics
                .get(Path::new("data/Map004.json"))
                .unwrap(),
            DocumentDiagnostic::ExceedsAggregateByteLimit { .. }
        ));
    }

    #[test]
    fn document_count_limit() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Map001.json"), b"{}").unwrap();
        fs::write(temp.path().join("data/Map002.json"), b"{}").unwrap();
        fs::write(temp.path().join("data/Map003.json"), b"{}").unwrap();

        let limits = SnapshotLimits {
            max_documents: NonZeroUsize::new(2).unwrap(),
            max_bytes_per_document: NonZeroUsize::new(100).unwrap(),
            max_aggregate_bytes: NonZeroUsize::new(100).unwrap(),
        };

        let snapshot = load_snapshot(&open_root(temp.path()), limits).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Partial);
        assert!(
            snapshot
                .documents
                .contains_key(Path::new("data/Map001.json"))
        );
        assert!(
            snapshot
                .documents
                .contains_key(Path::new("data/Map002.json"))
        );
        assert!(matches!(
            snapshot
                .diagnostics
                .get(Path::new("data/Map003.json"))
                .unwrap(),
            DocumentDiagnostic::ExceedsDocumentCountLimit { .. }
        ));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn non_utf8_paths() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        let name = OsString::from_vec(b"data/Map\xff.json".to_vec());
        fs::write(temp.path().join(&name), b"{}").unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        // It's classified as ExtensionCandidateFamily::DataJson because it ends in .json
        // Wait, classify_path checks extension.
        // Let's see if it's loaded.
        // Actually, classify_path might not classify it as DataJson if it's not valid UTF-8?
        // Path::new(second).extension() == Some(OsStr::new("json"))
        // Yes, it will be classified as DataJson.
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Complete);
        assert!(snapshot.documents.contains_key(&PathBuf::from(name)));
    }

    #[cfg(unix)]
    #[test]
    fn fatal_inventory_failure() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::new().unwrap();
        let root = open_root(temp.path());

        let mut perms = fs::metadata(temp.path()).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(temp.path(), perms).unwrap();

        let result = load_snapshot(&root, default_limits());
        assert!(matches!(result, Err(SnapshotError::Inventory { .. })));
    }
}
