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
///
/// These limits bound the processing of selected candidate documents. They do
/// not bound the initial inventory traversal, parser semantic complexity,
/// compatibility, or project validity.
///
/// `max_documents` bounds the number of attempted regular candidate documents,
/// including candidates that fail to open, read, parse, or satisfy byte limits.
/// Unsupported non-file entries are diagnosed without consuming this budget.
///
/// Per-document and aggregate limits permit at most a one-byte probe beyond the
/// limit to detect overflow. Aggregate exhaustion prevents further candidate
/// opens or reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotLimits {
    /// Maximum number of attempted regular candidate documents.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    inventory: ProjectInventory,
    documents: BTreeMap<PathBuf, LosslessJsonDocument>,
    diagnostics: BTreeMap<PathBuf, DocumentDiagnostic>,
}

impl ProjectSnapshot {
    /// Returns the inventory of the project directory.
    pub fn inventory(&self) -> &ProjectInventory {
        &self.inventory
    }

    /// Returns the successfully loaded documents, ordered by path.
    pub fn documents(&self) -> &BTreeMap<PathBuf, LosslessJsonDocument> {
        &self.documents
    }

    /// Returns diagnostics for candidate documents that could not be loaded.
    pub fn diagnostics(&self) -> &BTreeMap<PathBuf, DocumentDiagnostic> {
        &self.diagnostics
    }

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
///
/// The caller supplies an already-authorized [`Dir`]. This function does not
/// acquire ambient authority or attempt to prove how the caller selected that
/// root.
///
/// Candidate documents are selected based on inventory classification and exact
/// paths. Final-component symlinks are not followed. The snapshot is processed
/// deterministically in native path order.
///
/// If a document exceeds the provided resource limits, or if it fails to open,
/// read, or parse, it is recorded as a diagnostic and the snapshot is marked
/// partial. A fatal error is returned only if the project inventory fails.
///
/// Resource limits are enforced by probing at most one byte beyond the limit.
/// Once the aggregate budget is exhausted or its probe has demonstrated overflow,
/// further candidates are not opened or read. Concurrent mutation of the project
/// directory during loading is a documented limitation and may result in an
/// inconsistent snapshot.
///
/// # Errors
///
/// Returns [`SnapshotError`] if the project directory cannot be inventoried.
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

        document_count = document_count.saturating_add(1);

        if aggregate_bytes >= limits.max_aggregate_bytes.get() {
            diagnostics.insert(
                entry.path.clone(),
                DocumentDiagnostic::ExceedsAggregateByteLimit {
                    limit: limits.max_aggregate_bytes,
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

        let max_read_doc = limits.max_bytes_per_document.get();
        let remaining_aggregate = limits
            .max_aggregate_bytes
            .get()
            .saturating_sub(aggregate_bytes);
        let limit = std::cmp::min(max_read_doc, remaining_aggregate);
        let read_limit = limit.saturating_add(1);

        let mut buffer = Vec::new();
        let mut handle = (&mut file).take(read_limit as u64);

        let read_result = handle.read_to_end(&mut buffer);
        aggregate_bytes = aggregate_bytes.saturating_add(buffer.len());

        if let Err(source) = read_result {
            diagnostics.insert(entry.path.clone(), DocumentDiagnostic::Read { source });
            continue;
        }

        if buffer.len() > limit {
            if buffer.len() > max_read_doc {
                diagnostics.insert(
                    entry.path.clone(),
                    DocumentDiagnostic::ExceedsDocumentByteLimit {
                        limit: limits.max_bytes_per_document,
                    },
                );
            } else {
                diagnostics.insert(
                    entry.path.clone(),
                    DocumentDiagnostic::ExceedsAggregateByteLimit {
                        limit: limits.max_aggregate_bytes,
                    },
                );
            }
            continue;
        }

        match LosslessJsonDocument::parse(&buffer) {
            Ok(document) => {
                documents.insert(entry.path.clone(), document);
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
    fn snapshot_exposes_inventory() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Actors.json"), b"[]").unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Complete);
        assert!(!snapshot.inventory().entries.is_empty());
    }

    #[test]
    fn candidate_pathname_with_non_file_kind() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        // Create a directory with a candidate pathname
        fs::create_dir(temp.path().join("data/Actors.json")).unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Partial);
        assert!(snapshot.documents().is_empty());
        assert!(matches!(
            snapshot
                .diagnostics()
                .get(Path::new("data/Actors.json"))
                .unwrap(),
            DocumentDiagnostic::UnsupportedEntryKind {
                kind: InventoryEntryKind::Directory
            }
        ));
    }

    #[test]
    fn empty_project_and_non_json_files() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("notes.txt")).unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        File::create(temp.path().join("data/Actors.csv")).unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Complete);
        assert!(snapshot.documents().is_empty());
        assert!(snapshot.diagnostics().is_empty());
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
        assert_eq!(snapshot.documents().len(), 3);
        assert!(snapshot.documents().contains_key(Path::new("package.json")));
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
        assert_eq!(snapshot.documents().len(), 1);
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
        assert!(snapshot.documents().is_empty());
    }

    #[test]
    fn deterministic_native_path_order() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Zeta.json"), b"{}").unwrap();
        fs::write(temp.path().join("data/Alpha.json"), b"{}").unwrap();
        fs::write(temp.path().join("package.json"), b"{}").unwrap();

        let snapshot = load_snapshot(&open_root(temp.path()), default_limits()).unwrap();
        let paths: Vec<_> = snapshot.documents().keys().collect();
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
        assert_eq!(snapshot.documents().len(), 1);
        assert!(
            snapshot
                .documents
                .contains_key(Path::new("data/Actors.json"))
        );
        assert_eq!(snapshot.diagnostics().len(), 6);
        for i in 1..=6 {
            let path = PathBuf::from(format!("data/Map00{i}.json"));
            assert!(matches!(
                snapshot.diagnostics().get(&path).unwrap(),
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
        let doc = snapshot.documents().get(Path::new("package.json")).unwrap();
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
        assert!(snapshot.documents().is_empty());
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
    fn parse_failure_consumes_attempted_document_budget() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Map001.json"), b"invalid").unwrap();
        fs::write(temp.path().join("data/Map002.json"), b"{}").unwrap();

        let limits = SnapshotLimits {
            max_documents: NonZeroUsize::new(1).unwrap(),
            max_bytes_per_document: NonZeroUsize::new(100).unwrap(),
            max_aggregate_bytes: NonZeroUsize::new(100).unwrap(),
        };

        let snapshot = load_snapshot(&open_root(temp.path()), limits).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Partial);
        assert!(snapshot.documents().is_empty());

        assert!(matches!(
            snapshot
                .diagnostics()
                .get(Path::new("data/Map001.json"))
                .unwrap(),
            DocumentDiagnostic::Parse { .. }
        ));

        assert!(matches!(
            snapshot
                .diagnostics()
                .get(Path::new("data/Map002.json"))
                .unwrap(),
            DocumentDiagnostic::ExceedsDocumentCountLimit { .. }
        ));
    }

    #[test]
    fn parse_failure_consumes_aggregate_examined_bytes() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Map001.json"), b"invalid").unwrap(); // 7 bytes
        fs::write(temp.path().join("data/Map002.json"), b"{}").unwrap(); // 2 bytes

        let limits = SnapshotLimits {
            max_documents: NonZeroUsize::new(2).unwrap(),
            max_bytes_per_document: NonZeroUsize::new(100).unwrap(),
            max_aggregate_bytes: NonZeroUsize::new(8).unwrap(),
        };

        let snapshot = load_snapshot(&open_root(temp.path()), limits).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Partial);
        assert!(snapshot.documents().is_empty());

        assert!(matches!(
            snapshot
                .diagnostics()
                .get(Path::new("data/Map001.json"))
                .unwrap(),
            DocumentDiagnostic::Parse { .. }
        ));

        // Map001 consumed 7 bytes. Remaining aggregate is 1.
        // Map002 is 2 bytes. It will read 2 bytes (limit is 1, read_limit is 2).
        // 2 > 1, so it exceeds aggregate byte limit.
        assert!(matches!(
            snapshot
                .diagnostics()
                .get(Path::new("data/Map002.json"))
                .unwrap(),
            DocumentDiagnostic::ExceedsAggregateByteLimit { .. }
        ));
    }

    #[test]
    fn aggregate_exhaustion_does_not_repeatedly_read_later_candidates() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Map001.json"), b"12345").unwrap(); // 5 bytes

        // A later readable candidate would load if exhaustion did not short-circuit.
        // On Unix, remove its read permissions as an additional signal that it was
        // not opened.
        let map2 = temp.path().join("data/Map002.json");
        fs::write(&map2, b"{}").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&map2).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&map2, perms).unwrap();
        }

        let limits = SnapshotLimits {
            max_documents: NonZeroUsize::new(2).unwrap(),
            max_bytes_per_document: NonZeroUsize::new(10).unwrap(),
            max_aggregate_bytes: NonZeroUsize::new(5).unwrap(),
        };

        let snapshot = load_snapshot(&open_root(temp.path()), limits).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Partial);

        // Map001 exhausts the aggregate budget. Map002 must be skipped before any
        // open or read and receive the aggregate-limit diagnostic.
        assert!(matches!(
            snapshot
                .diagnostics()
                .get(Path::new("data/Map002.json"))
                .unwrap(),
            DocumentDiagnostic::ExceedsAggregateByteLimit { .. }
        ));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&map2).unwrap().permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&map2, perms).unwrap();
        }
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
            max_aggregate_bytes: NonZeroUsize::new(6).unwrap(),
        };

        let snapshot = load_snapshot(&open_root(temp.path()), limits).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Partial);

        // Map001: 2 bytes, ok. Aggregate = 2. Count = 1.
        assert!(
            snapshot
                .documents()
                .contains_key(Path::new("data/Map001.json"))
        );

        // Map002: 7 bytes. max_read_doc = 5, remaining_aggregate = 4. limit = 4.
        // We read 5 bytes. 5 > 4. ExceedsAggregateByteLimit.
        // Aggregate becomes 2 + 5 = 7. Count = 2.
        assert!(matches!(
            snapshot
                .diagnostics()
                .get(Path::new("data/Map002.json"))
                .unwrap(),
            DocumentDiagnostic::ExceedsAggregateByteLimit { .. }
        ));

        // Map003: remaining_aggregate = 0. limit = 0. read 1 byte.
        // 1 > 0. ExceedsAggregateByteLimit.
        // Aggregate becomes 7 + 1 = 8. Count = 3.
        assert!(matches!(
            snapshot
                .diagnostics()
                .get(Path::new("data/Map003.json"))
                .unwrap(),
            DocumentDiagnostic::ExceedsAggregateByteLimit { .. }
        ));

        // Map004: Count = 3 >= max_documents (3). ExceedsDocumentCountLimit.
        assert!(matches!(
            snapshot
                .diagnostics()
                .get(Path::new("data/Map004.json"))
                .unwrap(),
            DocumentDiagnostic::ExceedsDocumentCountLimit { .. }
        ));
    }

    #[test]
    fn resource_limits_document_byte_limit_precedence() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Map001.json"), b"{\"a\":1}").unwrap(); // 7 bytes

        let limits = SnapshotLimits {
            max_documents: NonZeroUsize::new(3).unwrap(),
            max_bytes_per_document: NonZeroUsize::new(5).unwrap(),
            max_aggregate_bytes: NonZeroUsize::new(10).unwrap(),
        };

        let snapshot = load_snapshot(&open_root(temp.path()), limits).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Partial);

        // Map001: 7 bytes. max_read_doc = 5, remaining_aggregate = 10. limit = 5.
        // We read 6 bytes. 6 > 5. 6 > max_read_doc (5). ExceedsDocumentByteLimit.
        assert!(matches!(
            snapshot
                .diagnostics()
                .get(Path::new("data/Map001.json"))
                .unwrap(),
            DocumentDiagnostic::ExceedsDocumentByteLimit { .. }
        ));
    }

    #[test]
    fn resource_limits_exact_boundary_acceptance() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Map001.json"), b"{}").unwrap(); // 2 bytes

        let limits = SnapshotLimits {
            max_documents: NonZeroUsize::new(1).unwrap(),
            max_bytes_per_document: NonZeroUsize::new(2).unwrap(),
            max_aggregate_bytes: NonZeroUsize::new(2).unwrap(),
        };

        let snapshot = load_snapshot(&open_root(temp.path()), limits).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Complete);
        assert!(
            snapshot
                .documents()
                .contains_key(Path::new("data/Map001.json"))
        );
    }

    #[test]
    fn resource_limits_nonzero_usize_max_does_not_panic() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/Map001.json"), b"{}").unwrap();

        let limits = SnapshotLimits {
            max_documents: NonZeroUsize::MAX,
            max_bytes_per_document: NonZeroUsize::MAX,
            max_aggregate_bytes: NonZeroUsize::MAX,
        };

        let snapshot = load_snapshot(&open_root(temp.path()), limits).unwrap();
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Complete);
        assert!(
            snapshot
                .documents()
                .contains_key(Path::new("data/Map001.json"))
        );
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
        // The path is classified as ExtensionCandidateFamily::DataJson because it ends in .json,
        // even though it contains non-UTF-8 bytes. It should be successfully loaded.
        assert_eq!(snapshot.completeness(), SnapshotCompleteness::Complete);
        assert!(snapshot.documents().contains_key(&PathBuf::from(name)));
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
