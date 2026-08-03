// SPDX-License-Identifier: MPL-2.0

//! Read-only discovery of RPG Maker MZ project candidates.
//!
//! This module implements the explicit-root candidate discovery contract. It does
//! not validate project contents, parse JSON, or guarantee compatibility with any
//! specific editor version.

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Errors that can occur during candidate discovery.
#[derive(Debug)]
pub enum DiscoveryError {
    /// Failed to inspect the root directory's metadata.
    InspectRoot {
        /// The root path being inspected.
        root: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// The supplied root path is a symlink. This is a best-effort check of the
    /// final supplied path component at inspection time.
    RootIsSymlink {
        /// The root path that was rejected.
        root: PathBuf,
    },
    /// The supplied root path is not a directory.
    RootIsNotDirectory {
        /// The root path that was rejected.
        root: PathBuf,
    },
    /// Failed to read the contents of the root directory.
    ReadRoot {
        /// The root directory being read.
        root: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// Failed to read a directory entry.
    ReadEntry {
        /// The root directory being read.
        root: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// Failed to inspect the metadata of a potential marker file.
    InspectMarker {
        /// The path of the marker file being inspected.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InspectRoot { root, .. } => {
                write!(f, "failed to inspect root path '{}'", root.display())
            }
            Self::RootIsSymlink { root } => {
                write!(f, "root path '{}' is a symlink", root.display())
            }
            Self::RootIsNotDirectory { root } => {
                write!(f, "root path '{}' is not a directory", root.display())
            }
            Self::ReadRoot { root, .. } => {
                write!(f, "failed to read root directory '{}'", root.display())
            }
            Self::ReadEntry { root, .. } => {
                write!(f, "failed to read entry in directory '{}'", root.display())
            }
            Self::InspectMarker { path, .. } => {
                write!(f, "failed to inspect marker file '{}'", path.display())
            }
        }
    }
}

impl Error for DiscoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InspectRoot { source, .. }
            | Self::ReadRoot { source, .. }
            | Self::ReadEntry { source, .. }
            | Self::InspectMarker { source, .. } => Some(source),
            Self::RootIsSymlink { .. } | Self::RootIsNotDirectory { .. } => None,
        }
    }
}

/// The kind of filesystem entry observed for a marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkerEntryKind {
    /// A regular file.
    RegularFile,
    /// A symbolic link.
    Symlink,
    /// A directory.
    Directory,
    /// Another non-regular file type (e.g., socket, device).
    OtherNonRegular,
}

/// An observation of a potential marker entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerObservation {
    /// The exact path to the observed marker entry.
    pub path: PathBuf,
    /// The kind of filesystem entry observed.
    pub kind: MarkerEntryKind,
}

/// The result of inspecting a directory for an RPG Maker MZ project candidate.
///
/// This API identifies candidates based on the presence of a marker file. It does
/// not validate the project, parse its contents, or guarantee compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateDiscovery {
    /// The directory contains exactly one regular file matching the expected
    /// lowercase marker name (`game.rmmzproject`).
    Candidate {
        /// The observed marker entry.
        marker: MarkerObservation,
    },
    /// The directory contains exactly one regular file matching the marker name
    /// case-insensitively, but not exactly (e.g., `Game.rmmzproject`).
    /// This is a platform-limited case variant.
    CaseVariantCandidate {
        /// The observed marker entry.
        marker: MarkerObservation,
    },
    /// The directory does not contain any file matching the marker name.
    NoMarker,
    /// The directory contains multiple files matching the marker name
    /// case-insensitively.
    AmbiguousMarkers {
        /// The observed marker entries, deterministically ordered by path.
        markers: Vec<MarkerObservation>,
    },
    /// The marker file exists but is a symbolic link.
    SymlinkMarker {
        /// The observed marker entry.
        marker: MarkerObservation,
    },
    /// The marker file exists but is not a regular file or symlink (e.g., a directory).
    NonRegularMarker {
        /// The observed marker entry.
        marker: MarkerObservation,
    },
}

/// Inspects the given directory to determine if it is an RPG Maker MZ project candidate.
///
/// This function performs a read-only, non-recursive inspection of the immediate
/// children of the provided directory. Immediate marker-entry symlinks are classified
/// without being followed.
///
/// The final component of the provided root path is inspected without following
/// symlinks. If the root itself is a symlink, it is rejected as a best-effort check.
/// Ancestor path resolution follows normal operating-system behavior, and concurrent
/// replacement is not prevented. This does not provide race-free sandbox containment.
///
/// # Errors
///
/// Returns a `DiscoveryError` if the directory cannot be read, if the root is a
/// symlink, or if it is not a directory. Negative findings (like missing markers)
/// are returned as `Ok(CandidateDiscovery::NoMarker)`, not as errors.
pub fn discover_candidate(root: &Path) -> Result<CandidateDiscovery, DiscoveryError> {
    let meta = fs::symlink_metadata(root).map_err(|source| DiscoveryError::InspectRoot {
        root: root.to_path_buf(),
        source,
    })?;

    if meta.is_symlink() {
        return Err(DiscoveryError::RootIsSymlink {
            root: root.to_path_buf(),
        });
    }

    if !meta.is_dir() {
        return Err(DiscoveryError::RootIsNotDirectory {
            root: root.to_path_buf(),
        });
    }

    let entries = fs::read_dir(root).map_err(|source| DiscoveryError::ReadRoot {
        root: root.to_path_buf(),
        source,
    })?;

    let mut observations = Vec::new();

    for entry_res in entries {
        let entry = entry_res.map_err(|source| DiscoveryError::ReadEntry {
            root: root.to_path_buf(),
            source,
        })?;
        let file_name = entry.file_name();
        let file_name_lossy = file_name.to_string_lossy();

        if file_name_lossy.eq_ignore_ascii_case("game.rmmzproject") {
            let file_type = entry
                .file_type()
                .map_err(|source| DiscoveryError::InspectMarker {
                    path: entry.path(),
                    source,
                })?;

            let kind = if file_type.is_symlink() {
                MarkerEntryKind::Symlink
            } else if file_type.is_dir() {
                MarkerEntryKind::Directory
            } else if file_type.is_file() {
                MarkerEntryKind::RegularFile
            } else {
                MarkerEntryKind::OtherNonRegular
            };

            observations.push(MarkerObservation {
                path: entry.path(),
                kind,
            });
        }
    }

    Ok(classify_matches(observations))
}

/// Classifies a set of marker observations into a `CandidateDiscovery` result.
fn classify_matches(mut observations: Vec<MarkerObservation>) -> CandidateDiscovery {
    if observations.is_empty() {
        return CandidateDiscovery::NoMarker;
    }

    if observations.len() > 1 {
        observations.sort_by(|a, b| a.path.cmp(&b.path));
        return CandidateDiscovery::AmbiguousMarkers {
            markers: observations,
        };
    }

    let marker = observations.pop().unwrap();

    match marker.kind {
        MarkerEntryKind::RegularFile => {
            let file_name = marker.path.file_name().unwrap().to_string_lossy();
            if file_name == "game.rmmzproject" {
                CandidateDiscovery::Candidate { marker }
            } else {
                CandidateDiscovery::CaseVariantCandidate { marker }
            }
        }
        MarkerEntryKind::Symlink => CandidateDiscovery::SymlinkMarker { marker },
        _ => CandidateDiscovery::NonRegularMarker { marker },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    #[test]
    fn test_classify_exact_match() {
        let marker = MarkerObservation {
            path: PathBuf::from("game.rmmzproject"),
            kind: MarkerEntryKind::RegularFile,
        };
        let result = classify_matches(vec![marker.clone()]);
        assert_eq!(result, CandidateDiscovery::Candidate { marker });
    }

    #[test]
    fn test_classify_case_variant() {
        let marker = MarkerObservation {
            path: PathBuf::from("Game.rmmzproject"),
            kind: MarkerEntryKind::RegularFile,
        };
        let result = classify_matches(vec![marker.clone()]);
        assert_eq!(result, CandidateDiscovery::CaseVariantCandidate { marker });
    }

    #[test]
    fn test_classify_no_marker() {
        let result = classify_matches(vec![]);
        assert_eq!(result, CandidateDiscovery::NoMarker);
    }

    #[test]
    fn test_classify_ambiguous() {
        let marker1 = MarkerObservation {
            path: PathBuf::from("Game.rmmzproject"),
            kind: MarkerEntryKind::RegularFile,
        };
        let marker2 = MarkerObservation {
            path: PathBuf::from("game.rmmzproject"),
            kind: MarkerEntryKind::RegularFile,
        };
        // Pass in reverse order to test sorting
        let result = classify_matches(vec![marker2.clone(), marker1.clone()]);
        assert_eq!(
            result,
            CandidateDiscovery::AmbiguousMarkers {
                markers: vec![marker1, marker2]
            }
        );
    }

    #[test]
    fn test_classify_symlink() {
        let marker = MarkerObservation {
            path: PathBuf::from("game.rmmzproject"),
            kind: MarkerEntryKind::Symlink,
        };
        let result = classify_matches(vec![marker.clone()]);
        assert_eq!(result, CandidateDiscovery::SymlinkMarker { marker });
    }

    #[test]
    fn test_classify_non_regular() {
        let marker = MarkerObservation {
            path: PathBuf::from("game.rmmzproject"),
            kind: MarkerEntryKind::Directory,
        };
        let result = classify_matches(vec![marker.clone()]);
        assert_eq!(result, CandidateDiscovery::NonRegularMarker { marker });
    }

    #[test]
    fn test_discover_exact_match() {
        let temp = TempDir::new().unwrap();
        let marker_path = temp.path().join("game.rmmzproject");
        File::create(&marker_path).unwrap();

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(
            result,
            CandidateDiscovery::Candidate {
                marker: MarkerObservation {
                    path: marker_path,
                    kind: MarkerEntryKind::RegularFile,
                }
            }
        );
    }

    #[test]
    fn test_discover_no_marker() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("other.txt")).unwrap();

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(result, CandidateDiscovery::NoMarker);
    }

    #[test]
    fn test_discover_wrong_extension() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("game.rmmvproject")).unwrap();

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(result, CandidateDiscovery::NoMarker);
    }

    #[test]
    fn test_discover_marker_in_child_directory() {
        let temp = TempDir::new().unwrap();
        let child = temp.path().join("child");
        fs::create_dir(&child).unwrap();
        File::create(child.join("game.rmmzproject")).unwrap();

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(result, CandidateDiscovery::NoMarker);
    }

    #[test]
    fn test_discover_deployment_shaped_tree() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::create_dir(temp.path().join("img")).unwrap();
        File::create(temp.path().join("index.html")).unwrap();
        File::create(temp.path().join("package.json")).unwrap();

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(result, CandidateDiscovery::NoMarker);
    }

    #[test]
    fn test_discover_unknown_entries_alongside_marker() {
        let temp = TempDir::new().unwrap();
        let marker_path = temp.path().join("game.rmmzproject");
        File::create(&marker_path).unwrap();
        File::create(temp.path().join("unknown.txt")).unwrap();
        fs::create_dir(temp.path().join("unknown_dir")).unwrap();

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(
            result,
            CandidateDiscovery::Candidate {
                marker: MarkerObservation {
                    path: marker_path,
                    kind: MarkerEntryKind::RegularFile,
                }
            }
        );
    }

    #[test]
    fn test_discover_non_regular_marker_directory() {
        let temp = TempDir::new().unwrap();
        let marker_path = temp.path().join("game.rmmzproject");
        fs::create_dir(&marker_path).unwrap();

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(
            result,
            CandidateDiscovery::NonRegularMarker {
                marker: MarkerObservation {
                    path: marker_path,
                    kind: MarkerEntryKind::Directory,
                }
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_discover_non_regular_marker_symlink() {
        let temp = TempDir::new().unwrap();
        let target_path = temp.path().join("target.rmmzproject");
        File::create(&target_path).unwrap();

        let marker_path = temp.path().join("game.rmmzproject");
        symlink(&target_path, &marker_path).unwrap();

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(
            result,
            CandidateDiscovery::SymlinkMarker {
                marker: MarkerObservation {
                    path: marker_path,
                    kind: MarkerEntryKind::Symlink,
                }
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_discover_root_is_symlink() {
        let temp = TempDir::new().unwrap();
        let real_dir = temp.path().join("real_dir");
        fs::create_dir(&real_dir).unwrap();
        File::create(real_dir.join("game.rmmzproject")).unwrap();

        let symlink_dir = temp.path().join("symlink_dir");
        symlink(&real_dir, &symlink_dir).unwrap();

        let result = discover_candidate(&symlink_dir);
        assert!(matches!(result, Err(DiscoveryError::RootIsSymlink { .. })));
    }

    #[cfg(windows)]
    #[test]
    fn test_discover_non_regular_marker_symlink_windows() {
        use std::os::windows::fs::symlink_file;
        let temp = TempDir::new().unwrap();
        let target_path = temp.path().join("target.rmmzproject");
        File::create(&target_path).unwrap();

        let marker_path = temp.path().join("game.rmmzproject");
        if let Err(e) = symlink_file(&target_path, &marker_path) {
            if e.kind() == io::ErrorKind::PermissionDenied {
                // Windows requires Developer Mode or Administrator privileges to create symlinks.
                // Skip the test if we don't have permission, but emit a clear diagnostic.
                eprintln!(
                    "Skipping test_discover_non_regular_marker_symlink_windows: PermissionDenied creating symlink."
                );
                return;
            }
            panic!("Failed to create symlink: {e}");
        }

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(
            result,
            CandidateDiscovery::SymlinkMarker {
                marker: MarkerObservation {
                    path: marker_path,
                    kind: MarkerEntryKind::Symlink,
                }
            }
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_discover_root_is_symlink_windows() {
        use std::os::windows::fs::symlink_dir;
        let temp = TempDir::new().unwrap();
        let real_dir = temp.path().join("real_dir");
        fs::create_dir(&real_dir).unwrap();
        File::create(real_dir.join("game.rmmzproject")).unwrap();

        let symlink_path = temp.path().join("symlink_dir");
        if let Err(e) = symlink_dir(&real_dir, &symlink_path) {
            // Windows requires Developer Mode or Administrator privileges to create symlinks.
            // If we lack permission, skip the test rather than failing it.
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                eprintln!(
                    "Skipping test_discover_root_is_symlink_windows: PermissionDenied creating symlink."
                );
                return;
            }
            panic!("Failed to create symlink: {e}");
        }

        let result = discover_candidate(&symlink_path);
        assert!(matches!(result, Err(DiscoveryError::RootIsSymlink { .. })));
    }

    #[test]
    fn test_discover_case_variant() {
        let temp = TempDir::new().unwrap();
        let marker_path = temp.path().join("Game.rmmzproject");
        File::create(&marker_path).unwrap();

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(
            result,
            CandidateDiscovery::CaseVariantCandidate {
                marker: MarkerObservation {
                    path: marker_path,
                    kind: MarkerEntryKind::RegularFile,
                }
            }
        );
    }

    #[test]
    fn test_discover_ambiguous_markers() {
        let temp = TempDir::new().unwrap();
        let marker1_path = temp.path().join("Game.rmmzproject");
        let marker2_path = temp.path().join("game.rmmzproject");

        File::options()
            .write(true)
            .create_new(true)
            .open(&marker1_path)
            .unwrap();

        let second_create = File::options()
            .write(true)
            .create_new(true)
            .open(&marker2_path);

        match second_create {
            Ok(_) => {
                // Both files were created successfully (case-sensitive filesystem).
                let result = discover_candidate(temp.path()).unwrap();
                assert_eq!(
                    result,
                    CandidateDiscovery::AmbiguousMarkers {
                        markers: vec![
                            MarkerObservation {
                                path: marker1_path,
                                kind: MarkerEntryKind::RegularFile,
                            },
                            MarkerObservation {
                                path: marker2_path,
                                kind: MarkerEntryKind::RegularFile,
                            },
                        ]
                    }
                );
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                // The second creation failed because the filesystem is case-insensitive.
                let result = discover_candidate(temp.path()).unwrap();
                assert_eq!(
                    result,
                    CandidateDiscovery::CaseVariantCandidate {
                        marker: MarkerObservation {
                            path: marker1_path,
                            kind: MarkerEntryKind::RegularFile,
                        }
                    }
                );
            }
            Err(e) => panic!("Unexpected error creating second marker: {e}"),
        }
    }

    #[test]
    fn test_discover_marker_content_neutrality() {
        let temp = TempDir::new().unwrap();
        let marker_path = temp.path().join("game.rmmzproject");

        // Empty marker
        File::create(&marker_path).unwrap();
        let result = discover_candidate(temp.path()).unwrap();
        assert!(matches!(result, CandidateDiscovery::Candidate { .. }));

        // Altered/version-like text
        fs::write(&marker_path, "RPGMZ 1.10.1").unwrap();
        let result = discover_candidate(temp.path()).unwrap();
        assert!(matches!(result, CandidateDiscovery::Candidate { .. }));
    }

    #[test]
    fn test_discover_missing_root() {
        let temp = TempDir::new().unwrap();
        let missing_path = temp.path().join("missing");
        let result = discover_candidate(&missing_path);
        assert!(matches!(result, Err(DiscoveryError::InspectRoot { .. })));
    }

    #[test]
    fn test_discover_non_directory_root() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("file.txt");
        File::create(&file_path).unwrap();
        let result = discover_candidate(&file_path);
        assert!(matches!(
            result,
            Err(DiscoveryError::RootIsNotDirectory { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn test_discover_other_non_regular_marker() {
        let temp = TempDir::new().unwrap();
        let marker_path = temp.path().join("game.rmmzproject");

        // Create a Unix domain socket
        let _listener = std::os::unix::net::UnixListener::bind(&marker_path).unwrap();

        let result = discover_candidate(temp.path()).unwrap();
        assert_eq!(
            result,
            CandidateDiscovery::NonRegularMarker {
                marker: MarkerObservation {
                    path: marker_path,
                    kind: MarkerEntryKind::OtherNonRegular,
                }
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_discover_root_symlink_with_dot_is_not_rejected_known_limitation() {
        let temp = TempDir::new().unwrap();
        let real_dir = temp.path().join("real_dir");
        fs::create_dir(&real_dir).unwrap();
        File::create(real_dir.join("game.rmmzproject")).unwrap();

        let symlink_dir = temp.path().join("symlink_dir");
        symlink(&real_dir, &symlink_dir).unwrap();

        // A path ending in "." resolves through the symlink to the target directory.
        // symlink_metadata on "symlink_dir/." sees a directory, not a symlink.
        // This test documents the known limitation that the current std::fs implementation
        // does not provide complete root symlink rejection or race-free containment.
        let dot_path = symlink_dir.join(".");
        let result = discover_candidate(&dot_path).unwrap();
        assert!(matches!(result, CandidateDiscovery::Candidate { .. }));
    }
}
