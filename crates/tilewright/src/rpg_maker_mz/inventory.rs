// SPDX-License-Identifier: MPL-2.0

//! Capability-relative, read-only inventory of RPG Maker MZ project entries.
//!
//! Inventory reports paths and filesystem entry kinds without reading file
//! contents. Classifications describe only evidenced pathname families; they do
//! not validate an entry's contents, role, requiredness, or compatibility.

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

const STANDARD_ROOT_ENTRIES: &[&str] = &[
    "audio",
    "css",
    "data",
    "effects",
    "fonts",
    "game.rmmzproject",
    "icon",
    "img",
    "index.html",
    "js",
    "movies",
    "package.json",
];

const STANDARD_DATA_FILES: &[&str] = &[
    "Actors.json",
    "Animations.json",
    "Armors.json",
    "Classes.json",
    "CommonEvents.json",
    "Enemies.json",
    "Items.json",
    "MapInfos.json",
    "Skills.json",
    "States.json",
    "System.json",
    "Tilesets.json",
    "Troops.json",
    "Weapons.json",
];

/// Errors that can occur while inventorying a project capability.
///
/// The enum and its record variants are non-exhaustive so later inventory
/// phases can add context without making downstream matches exhaustive.
#[derive(Debug)]
#[non_exhaustive]
pub enum InventoryError {
    /// Failed to begin reading a directory.
    #[non_exhaustive]
    ReadDirectory {
        /// The exact project-relative path of the directory. The empty path
        /// identifies the supplied project-root capability.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// Failed to retrieve an entry while reading a directory.
    #[non_exhaustive]
    ReadEntry {
        /// The exact project-relative path of the containing directory. The
        /// empty path identifies the supplied project-root capability.
        directory: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// Failed to inspect an entry's filesystem kind.
    #[non_exhaustive]
    InspectEntry {
        /// The exact project-relative path of the entry.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// Failed to open an observed directory without following a symlink.
    #[non_exhaustive]
    OpenDirectory {
        /// The exact project-relative path of the directory.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDirectory { path, .. } if path.as_os_str().is_empty() => {
                f.write_str("failed to read project root")
            }
            Self::ReadDirectory { path, .. } => {
                write!(f, "failed to read project directory '{}'", path.display())
            }
            Self::ReadEntry { directory, .. } if directory.as_os_str().is_empty() => {
                f.write_str("failed to read an entry in the project root")
            }
            Self::ReadEntry { directory, .. } => write!(
                f,
                "failed to read an entry in project directory '{}'",
                directory.display()
            ),
            Self::InspectEntry { path, .. } => {
                write!(f, "failed to inspect project entry '{}'", path.display())
            }
            Self::OpenDirectory { path, .. } => write!(
                f,
                "failed to open project directory '{}' without following symlinks",
                path.display()
            ),
        }
    }
}

impl Error for InventoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadDirectory { source, .. }
            | Self::ReadEntry { source, .. }
            | Self::InspectEntry { source, .. }
            | Self::OpenDirectory { source, .. } => Some(source),
        }
    }
}

/// The observed kind of a filesystem entry.
///
/// This enum is non-exhaustive so additional platform entry kinds can be
/// represented without closing the experimental API prematurely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InventoryEntryKind {
    /// A regular file.
    File,
    /// A directory.
    Directory,
    /// A symbolic link, reported without following its target.
    Symlink,
    /// Another non-regular entry such as a socket or device.
    Other,
}

/// An evidenced standard pathname family.
///
/// A known pathname does not imply that the observed entry kind or contents are
/// valid for that family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum KnownEntryFamily {
    /// One of the exact immediate-root names evidenced in fresh MZ 1.10.0
    /// projects, including the marker and runtime-shaped entries.
    StandardRootEntry,
    /// One of the exact standard filenames immediately beneath `data`.
    StandardDataFile,
    /// An immediate `data/MapNNN.json` pathname with exactly three ASCII digits.
    MapDataFile,
}

/// A documented extension-capable pathname family.
///
/// Membership is only a location-based candidate classification. It does not
/// establish that an entry was created by a plugin or contains valid JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExtensionCandidateFamily {
    /// An otherwise unknown immediate `data/*.json` pathname.
    DataJson,
}

/// Conservative pathname classification for an inventory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InventoryClassification {
    /// The exact path belongs to an evidenced standard pathname family.
    #[non_exhaustive]
    Known {
        /// The evidenced pathname family.
        family: KnownEntryFamily,
    },
    /// The exact path belongs to a documented extension-capable location.
    #[non_exhaustive]
    ExtensionCandidate {
        /// The documented extension-capable pathname family.
        family: ExtensionCandidateFamily,
    },
    /// No evidenced standard or extension-candidate family matches the exact
    /// path.
    Unknown,
}

/// One exact project-relative inventory observation.
///
/// Classification is based only on the path. Callers must inspect `kind`
/// separately and must not treat a known path as proof of valid contents.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct InventoryEntry {
    /// The exact project-relative path, preserving the operating system's path
    /// spelling and representation.
    pub path: PathBuf,
    /// The observed filesystem entry kind.
    pub kind: InventoryEntryKind,
    /// The conservative pathname classification.
    pub classification: InventoryClassification,
}

/// A deterministically ordered inventory result beneath a project-root capability.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProjectInventory {
    /// All descendant entries ordered by exact project-relative path using the
    /// platform's native `Path` ordering.
    pub entries: Vec<InventoryEntry>,
}

/// Recursively inventories descendants of an authorized project directory.
///
/// The caller supplies the already-authorized [`Dir`]. This function does not
/// acquire ambient authority or attempt to prove how the caller selected that
/// root. All traversal is relative to the supplied capability. Symbolic links
/// are reported and never traversed. File contents are never opened or read.
///
/// Classification is case-sensitive and does not normalize path components.
/// Results include every observed descendant, including unknown and non-UTF-8
/// paths, in deterministic native path order. The root itself is not an entry.
/// Traversal is not an atomic filesystem snapshot; callers must prevent
/// concurrent project mutation if they require a mutually consistent result.
///
/// # Errors
///
/// Returns [`InventoryError`] with project-relative context when a directory
/// cannot be read, an entry cannot be inspected, or a directory cannot be
/// opened without following symlinks. No partial inventory is returned.
pub fn inventory_project(root: &Dir) -> Result<ProjectInventory, InventoryError> {
    let mut entries = Vec::new();
    let mut pending = Vec::new();

    inventory_directory(root, Path::new(""), &mut entries, &mut pending)?;

    while let Some((directory, path)) = pending.pop() {
        inventory_directory(&directory, &path, &mut entries, &mut pending)?;
    }

    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(ProjectInventory { entries })
}

fn inventory_directory(
    directory: &Dir,
    relative_path: &Path,
    inventory: &mut Vec<InventoryEntry>,
    pending: &mut Vec<(Dir, PathBuf)>,
) -> Result<(), InventoryError> {
    let entries = directory
        .entries()
        .map_err(|source| InventoryError::ReadDirectory {
            path: relative_path.to_path_buf(),
            source,
        })?;

    for entry in entries {
        let entry = entry.map_err(|source| InventoryError::ReadEntry {
            directory: relative_path.to_path_buf(),
            source,
        })?;
        let file_name = entry.file_name();
        let path = relative_path.join(&file_name);
        let file_type = entry
            .file_type()
            .map_err(|source| InventoryError::InspectEntry {
                path: path.clone(),
                source,
            })?;

        let kind = if file_type.is_symlink() {
            InventoryEntryKind::Symlink
        } else if file_type.is_dir() {
            InventoryEntryKind::Directory
        } else if file_type.is_file() {
            InventoryEntryKind::File
        } else {
            InventoryEntryKind::Other
        };

        inventory.push(InventoryEntry {
            classification: classify_path(&path),
            path: path.clone(),
            kind,
        });

        if kind == InventoryEntryKind::Directory {
            let child = open_directory_without_following(directory, &file_name, &path)?;
            pending.push((child, path));
        }
    }

    Ok(())
}

fn open_directory_without_following(
    parent: &Dir,
    file_name: &OsStr,
    path: &Path,
) -> Result<Dir, InventoryError> {
    parent
        .open_dir_nofollow(file_name)
        .map_err(|source| InventoryError::OpenDirectory {
            path: path.to_path_buf(),
            source,
        })
}

fn classify_path(path: &Path) -> InventoryClassification {
    let mut components = path.components();
    let Some(first) = components.next() else {
        return InventoryClassification::Unknown;
    };
    let first = first.as_os_str();
    let second = components.next().map(|component| component.as_os_str());

    if second.is_none()
        && STANDARD_ROOT_ENTRIES
            .iter()
            .any(|known| first == OsStr::new(known))
    {
        return InventoryClassification::Known {
            family: KnownEntryFamily::StandardRootEntry,
        };
    }

    let Some(second) = second else {
        return InventoryClassification::Unknown;
    };

    if components.next().is_some() || first != OsStr::new("data") {
        return InventoryClassification::Unknown;
    }

    if STANDARD_DATA_FILES
        .iter()
        .any(|known| second == OsStr::new(known))
    {
        return InventoryClassification::Known {
            family: KnownEntryFamily::StandardDataFile,
        };
    }

    if is_map_data_file(second) {
        return InventoryClassification::Known {
            family: KnownEntryFamily::MapDataFile,
        };
    }

    if Path::new(second).extension() == Some(OsStr::new("json")) {
        return InventoryClassification::ExtensionCandidate {
            family: ExtensionCandidateFamily::DataJson,
        };
    }

    InventoryClassification::Unknown
}

fn is_map_data_file(file_name: &OsStr) -> bool {
    let Some(file_name) = file_name.to_str() else {
        return false;
    };
    let Some(digits) = file_name
        .strip_prefix("Map")
        .and_then(|name| name.strip_suffix(".json"))
    else {
        return false;
    };

    digits.len() == 3 && digits.bytes().all(|byte| byte.is_ascii_digit())
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

    fn entry(
        path: impl Into<PathBuf>,
        kind: InventoryEntryKind,
        classification: InventoryClassification,
    ) -> InventoryEntry {
        InventoryEntry {
            path: path.into(),
            kind,
            classification,
        }
    }

    #[test]
    fn inventories_standard_root_and_data_families_without_reading_contents() {
        let temp = TempDir::new().expect("create temporary directory");
        fs::create_dir(temp.path().join("data")).expect("create data directory");
        File::create(temp.path().join("game.rmmzproject")).expect("create marker");
        fs::write(temp.path().join("data/Actors.json"), [0xff, 0xfe])
            .expect("create intentionally non-JSON contents");
        File::create(temp.path().join("data/Map007.json")).expect("create map path");

        let inventory = inventory_project(&open_root(temp.path())).expect("inventory succeeds");

        assert_eq!(
            inventory.entries,
            vec![
                entry(
                    "data",
                    InventoryEntryKind::Directory,
                    InventoryClassification::Known {
                        family: KnownEntryFamily::StandardRootEntry,
                    },
                ),
                entry(
                    "data/Actors.json",
                    InventoryEntryKind::File,
                    InventoryClassification::Known {
                        family: KnownEntryFamily::StandardDataFile,
                    },
                ),
                entry(
                    "data/Map007.json",
                    InventoryEntryKind::File,
                    InventoryClassification::Known {
                        family: KnownEntryFamily::MapDataFile,
                    },
                ),
                entry(
                    "game.rmmzproject",
                    InventoryEntryKind::File,
                    InventoryClassification::Known {
                        family: KnownEntryFamily::StandardRootEntry,
                    },
                ),
            ]
        );
    }

    #[test]
    fn every_evidenced_standard_pathname_is_known() {
        for root_name in STANDARD_ROOT_ENTRIES {
            assert_eq!(
                classify_path(Path::new(root_name)),
                InventoryClassification::Known {
                    family: KnownEntryFamily::StandardRootEntry,
                },
                "root path {root_name}"
            );
        }

        for file_name in STANDARD_DATA_FILES {
            let path = Path::new("data").join(file_name);
            assert_eq!(
                classify_path(&path),
                InventoryClassification::Known {
                    family: KnownEntryFamily::StandardDataFile,
                },
                "data path {}",
                path.display()
            );
        }
    }

    #[test]
    fn empty_project_has_an_empty_inventory() {
        let temp = TempDir::new().expect("create temporary directory");

        assert!(
            inventory_project(&open_root(temp.path()))
                .expect("inventory succeeds")
                .entries
                .is_empty()
        );
    }

    #[test]
    fn classification_boundaries_are_exact_and_case_sensitive() {
        let cases = [
            (
                "data/Map000.json",
                InventoryClassification::Known {
                    family: KnownEntryFamily::MapDataFile,
                },
            ),
            (
                "data/Map999.json",
                InventoryClassification::Known {
                    family: KnownEntryFamily::MapDataFile,
                },
            ),
            (
                "data/Map01.json",
                InventoryClassification::ExtensionCandidate {
                    family: ExtensionCandidateFamily::DataJson,
                },
            ),
            (
                "data/Map0001.json",
                InventoryClassification::ExtensionCandidate {
                    family: ExtensionCandidateFamily::DataJson,
                },
            ),
            (
                "data/map001.json",
                InventoryClassification::ExtensionCandidate {
                    family: ExtensionCandidateFamily::DataJson,
                },
            ),
            ("Data/Actors.json", InventoryClassification::Unknown),
            (
                "data/actors.json",
                InventoryClassification::ExtensionCandidate {
                    family: ExtensionCandidateFamily::DataJson,
                },
            ),
            ("GAME.RMMZPROJECT", InventoryClassification::Unknown),
            ("nested/data/Actors.json", InventoryClassification::Unknown),
        ];

        for (path, expected) in cases {
            assert_eq!(classify_path(Path::new(path)), expected, "path {path}");
        }
    }

    #[test]
    fn classifies_only_immediate_unknown_data_json_as_extension_candidates() {
        let cases = [
            (
                "data/PluginData.json",
                InventoryClassification::ExtensionCandidate {
                    family: ExtensionCandidateFamily::DataJson,
                },
            ),
            ("data/PluginData.JSON", InventoryClassification::Unknown),
            ("data/plugin.dat", InventoryClassification::Unknown),
            (
                "data/nested/PluginData.json",
                InventoryClassification::Unknown,
            ),
            ("plugins/PluginData.json", InventoryClassification::Unknown),
        ];

        for (path, expected) in cases {
            assert_eq!(classify_path(Path::new(path)), expected, "path {path}");
        }
    }

    #[test]
    fn recursively_reports_nested_and_unknown_entries() {
        let temp = TempDir::new().expect("create temporary directory");
        fs::create_dir_all(temp.path().join("img/pictures/custom/deep"))
            .expect("create nested directories");
        File::create(temp.path().join("img/pictures/custom/deep/picture.png"))
            .expect("create nested file");
        File::create(temp.path().join("notes.txt")).expect("create unknown root file");

        let inventory = inventory_project(&open_root(temp.path())).expect("inventory succeeds");
        let paths: Vec<_> = inventory
            .entries
            .iter()
            .map(|entry| entry.path.as_path())
            .collect();

        assert_eq!(
            paths,
            [
                Path::new("img"),
                Path::new("img/pictures"),
                Path::new("img/pictures/custom"),
                Path::new("img/pictures/custom/deep"),
                Path::new("img/pictures/custom/deep/picture.png"),
                Path::new("notes.txt"),
            ]
        );
        assert!(
            inventory.entries[1..]
                .iter()
                .all(|entry| entry.classification == InventoryClassification::Unknown)
        );
    }

    #[test]
    fn results_are_deterministically_ordered_by_relative_path() {
        let temp = TempDir::new().expect("create temporary directory");
        fs::create_dir(temp.path().join("zeta")).expect("create zeta");
        File::create(temp.path().join("zeta/first")).expect("create nested file");
        File::create(temp.path().join("middle")).expect("create middle");
        fs::create_dir(temp.path().join("alpha")).expect("create alpha");
        File::create(temp.path().join("alpha/last")).expect("create nested file");

        let root = open_root(temp.path());
        let first = inventory_project(&root).expect("first inventory succeeds");
        let second = inventory_project(&root).expect("second inventory succeeds");

        assert_eq!(first, second);
        assert_eq!(
            first
                .entries
                .iter()
                .map(|entry| entry.path.as_path())
                .collect::<Vec<_>>(),
            [
                Path::new("alpha"),
                Path::new("alpha/last"),
                Path::new("middle"),
                Path::new("zeta"),
                Path::new("zeta/first"),
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn reports_other_non_regular_entries() {
        let temp = TempDir::new().expect("create temporary directory");
        let _listener = std::os::unix::net::UnixListener::bind(temp.path().join("socket"))
            .expect("create Unix socket");

        let inventory = inventory_project(&open_root(temp.path())).expect("inventory succeeds");

        assert_eq!(
            inventory.entries,
            vec![entry(
                "socket",
                InventoryEntryKind::Other,
                InventoryClassification::Unknown,
            )]
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn preserves_non_utf8_paths() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let temp = TempDir::new().expect("create temporary directory");
        let name = OsString::from_vec(b"unknown-\xff".to_vec());
        File::create(temp.path().join(&name)).expect("create non-UTF-8 path");

        let inventory = inventory_project(&open_root(temp.path())).expect("inventory succeeds");

        assert_eq!(inventory.entries.len(), 1);
        assert_eq!(inventory.entries[0].path, PathBuf::from(name));
        assert_eq!(
            inventory.entries[0].classification,
            InventoryClassification::Unknown
        );
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn reports_symlink_without_following_its_target() {
        let temp = TempDir::new().expect("create temporary directory");
        let project = temp.path().join("project");
        let outside = temp.path().join("outside");
        fs::create_dir(&project).expect("create project directory");
        fs::create_dir(&outside).expect("create outside directory");
        File::create(outside.join("must-not-appear")).expect("create outside file");
        let link = project.join("linked");

        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).expect("create directory symlink");

        #[cfg(windows)]
        if let Err(error) = std::os::windows::fs::symlink_dir(&outside, &link) {
            if error.kind() == io::ErrorKind::PermissionDenied {
                eprintln!("skipping symlink test: permission denied creating symlink");
                return;
            }
            panic!("create directory symlink: {error}");
        }

        let inventory = inventory_project(&open_root(&project)).expect("inventory succeeds");

        assert_eq!(
            inventory.entries,
            vec![entry(
                "linked",
                InventoryEntryKind::Symlink,
                InventoryClassification::Unknown,
            )]
        );
    }

    #[test]
    fn read_error_variants_preserve_relative_context_and_sources() {
        let cases = [
            (
                InventoryError::ReadDirectory {
                    path: PathBuf::new(),
                    source: io::Error::new(io::ErrorKind::PermissionDenied, "read root"),
                },
                "failed to read project root".to_owned(),
                PathBuf::new(),
            ),
            (
                InventoryError::ReadDirectory {
                    path: PathBuf::from("data"),
                    source: io::Error::new(io::ErrorKind::PermissionDenied, "read directory"),
                },
                "failed to read project directory 'data'".to_owned(),
                PathBuf::from("data"),
            ),
            (
                InventoryError::ReadEntry {
                    directory: PathBuf::from("img"),
                    source: io::Error::new(io::ErrorKind::PermissionDenied, "read entry"),
                },
                "failed to read an entry in project directory 'img'".to_owned(),
                PathBuf::from("img"),
            ),
            (
                InventoryError::InspectEntry {
                    path: PathBuf::from("data/System.json"),
                    source: io::Error::new(io::ErrorKind::PermissionDenied, "inspect entry"),
                },
                "failed to inspect project entry 'data/System.json'".to_owned(),
                PathBuf::from("data/System.json"),
            ),
            (
                InventoryError::OpenDirectory {
                    path: PathBuf::from("audio"),
                    source: io::Error::new(io::ErrorKind::PermissionDenied, "open directory"),
                },
                "failed to open project directory 'audio' without following symlinks".to_owned(),
                PathBuf::from("audio"),
            ),
        ];

        for (error, display, expected_path) in cases {
            let actual_path = match &error {
                InventoryError::ReadDirectory { path, .. }
                | InventoryError::InspectEntry { path, .. }
                | InventoryError::OpenDirectory { path, .. } => path,
                InventoryError::ReadEntry { directory, .. } => directory,
            };
            assert_eq!(actual_path, &expected_path);
            assert_eq!(error.to_string(), display);
            assert_eq!(
                Error::source(&error)
                    .and_then(|source| source.downcast_ref::<io::Error>())
                    .map(io::Error::kind),
                Some(io::ErrorKind::PermissionDenied)
            );
        }
    }
}
