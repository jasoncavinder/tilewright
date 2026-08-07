// SPDX-License-Identifier: MPL-2.0

//! Experimental, read-only summary of one selected RPG Maker MZ map.

use crate::rpg_maker_mz::map_catalog::{
    JsonValueKind, MapCatalogError, MapId, evidenced_map_document_path, map_catalog,
};
use crate::rpg_maker_mz::snapshot::ProjectSnapshot;
use jsonc_parser::cst::{CstNode, CstObject};
use std::fmt;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

/// A read-only summary of one catalog-selected map.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MapSummary {
    id: MapId,
    catalog_name: String,
    document_path: PathBuf,
    display_name: String,
    width: NonZeroU32,
    height: NonZeroU32,
    tileset_id: NonZeroU32,
    event_count: usize,
}

impl MapSummary {
    /// Returns the selected map identifier.
    pub fn id(&self) -> MapId {
        self.id
    }

    /// Returns the decoded editor-facing name from `MapInfos.json`.
    pub fn catalog_name(&self) -> &str {
        &self.catalog_name
    }

    /// Returns the exact evidenced project-relative map-document path.
    pub fn document_path(&self) -> &Path {
        &self.document_path
    }

    /// Returns the decoded `displayName` from the map document.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Returns the positive map width scalar.
    pub fn width(&self) -> u32 {
        self.width.get()
    }

    /// Returns the positive map height scalar.
    pub fn height(&self) -> u32 {
        self.height.get()
    }

    /// Returns the positive tileset ID scalar.
    ///
    /// This is not yet a stable project-wide resource identifier, and this
    /// summary does not validate that the referenced tileset exists.
    pub fn tileset_id(&self) -> u32 {
        self.tileset_id.get()
    }

    /// Returns the number of non-null object entries in the `events` array.
    ///
    /// Event objects remain opaque; their IDs, pages, commands, and other
    /// fields are not interpreted by this summary.
    pub fn event_count(&self) -> usize {
        self.event_count
    }
}

/// A required field in the experimental selected-map summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapSummaryField {
    /// The map-facing display name.
    DisplayName,
    /// The positive map width scalar.
    Width,
    /// The positive map height scalar.
    Height,
    /// The positive tileset ID scalar.
    TilesetId,
    /// The array of opaque event entries and null holes.
    Events,
}

impl MapSummaryField {
    fn name(self) -> &'static str {
        match self {
            Self::DisplayName => "displayName",
            Self::Width => "width",
            Self::Height => "height",
            Self::TilesetId => "tilesetId",
            Self::Events => "events",
        }
    }
}

impl fmt::Display for MapSummaryField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Why a selected map could not be summarized coherently.
#[derive(Debug)]
#[non_exhaustive]
pub enum MapSummaryError {
    /// The project could not produce a structurally coherent map catalog.
    #[non_exhaustive]
    Catalog { source: MapCatalogError },
    /// The requested ID has no record in the coherent map catalog.
    #[non_exhaustive]
    UnknownMapId { map_id: MapId },
    /// The ID is outside the evidenced three-digit map-document family.
    #[non_exhaustive]
    UnevidencedDocumentPath { map_id: MapId },
    /// No inventory entry or loaded document exists at the expected path.
    #[non_exhaustive]
    MissingDocument { map_id: MapId, path: PathBuf },
    /// Snapshot diagnostics exist instead of a loaded document at the path.
    #[non_exhaustive]
    UnavailableDocument { map_id: MapId, path: PathBuf },
    /// The selected map document root is not an object.
    #[non_exhaustive]
    UnexpectedRootKind {
        map_id: MapId,
        path: PathBuf,
        actual: JsonValueKind,
    },
    /// A required decoded field is absent.
    #[non_exhaustive]
    MissingField {
        map_id: MapId,
        path: PathBuf,
        field: MapSummaryField,
    },
    /// A required decoded field name occurs more than once.
    #[non_exhaustive]
    DuplicateField {
        map_id: MapId,
        path: PathBuf,
        field: MapSummaryField,
    },
    /// A required field has the wrong JSON value kind.
    #[non_exhaustive]
    UnexpectedFieldKind {
        map_id: MapId,
        path: PathBuf,
        field: MapSummaryField,
        actual: JsonValueKind,
    },
    /// A required number is not a supported positive unsigned integer.
    #[non_exhaustive]
    UnsupportedInteger {
        map_id: MapId,
        path: PathBuf,
        field: MapSummaryField,
    },
    /// The display-name string could not be decoded.
    #[non_exhaustive]
    InvalidString {
        map_id: MapId,
        path: PathBuf,
        field: MapSummaryField,
    },
    /// A non-null event-array entry is not an opaque object.
    #[non_exhaustive]
    UnexpectedEventEntryKind {
        map_id: MapId,
        path: PathBuf,
        index: usize,
        actual: JsonValueKind,
    },
}

impl fmt::Display for MapSummaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalog { source, .. } => {
                write!(formatter, "map catalog is unavailable: {source}")
            }
            Self::UnknownMapId { map_id, .. } => {
                write!(formatter, "map catalog has no record for map {map_id}")
            }
            Self::UnevidencedDocumentPath { map_id, .. } => write!(
                formatter,
                "map {map_id} is outside the evidenced three-digit document family"
            ),
            Self::MissingDocument { map_id, path, .. } => write!(
                formatter,
                "map {map_id} document {} is missing",
                path.display()
            ),
            Self::UnavailableDocument { map_id, path, .. } => write!(
                formatter,
                "map {map_id} document {} was not loaded; inspect snapshot diagnostics",
                path.display()
            ),
            Self::UnexpectedRootKind {
                map_id,
                path,
                actual,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} root is {actual}, expected object",
                path.display()
            ),
            Self::MissingField {
                map_id,
                path,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} is missing required field {field}",
                path.display()
            ),
            Self::DuplicateField {
                map_id,
                path,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} has duplicate required field {field}",
                path.display()
            ),
            Self::UnexpectedFieldKind {
                map_id,
                path,
                field,
                actual,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} field {field} is {actual}, expected {}",
                path.display(),
                expected_kind(*field)
            ),
            Self::UnsupportedInteger {
                map_id,
                path,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} field {field} is not a supported positive unsigned integer",
                path.display()
            ),
            Self::InvalidString {
                map_id,
                path,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} field {field} could not be decoded",
                path.display()
            ),
            Self::UnexpectedEventEntryKind {
                map_id,
                path,
                index,
                actual,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} event entry {index} is {actual}, expected object or null",
                path.display()
            ),
        }
    }
}

impl std::error::Error for MapSummaryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Catalog { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Projects one catalog-selected map into a bounded read-only summary.
///
/// This function reads only lossless documents and diagnostics already present
/// in `snapshot`. It requires a coherent map catalog and a matching catalog
/// record, then reads the evidenced `data/MapNNN.json` document for IDs through
/// 999. Unknown fields, tile data, and event bodies remain untouched in the raw
/// snapshot.
///
/// The result does not validate tileset references, interpret events or tile
/// layers, perform filesystem I/O, mutate raw data, establish editor
/// compatibility, or persist changes.
///
/// # Errors
///
/// Returns [`MapSummaryError`] when the catalog is unavailable, the map ID or
/// document relationship is outside the bounded contract, the document is
/// absent or unavailable, or required map-summary structure is ambiguous.
pub fn map_summary(
    snapshot: &ProjectSnapshot,
    map_id: MapId,
) -> Result<MapSummary, MapSummaryError> {
    let catalog = map_catalog(snapshot).map_err(|source| MapSummaryError::Catalog { source })?;
    let record = catalog
        .get(map_id)
        .ok_or(MapSummaryError::UnknownMapId { map_id })?;
    let path = evidenced_map_document_path(map_id)
        .ok_or(MapSummaryError::UnevidencedDocumentPath { map_id })?;
    let Some(document) = snapshot.documents().get(path.as_path()) else {
        return if snapshot.diagnostics().contains_key(path.as_path()) {
            Err(MapSummaryError::UnavailableDocument { map_id, path })
        } else {
            Err(MapSummaryError::MissingDocument { map_id, path })
        };
    };

    let root = document
        .cst_root()
        .value()
        .ok_or_else(|| MapSummaryError::UnexpectedRootKind {
            map_id,
            path: path.clone(),
            actual: JsonValueKind::Unrecognized,
        })?;
    let object = root
        .as_object()
        .ok_or_else(|| MapSummaryError::UnexpectedRootKind {
            map_id,
            path: path.clone(),
            actual: json_kind(&root),
        })?;

    let display_name = required_string(&object, map_id, &path, MapSummaryField::DisplayName)?;
    let width = required_positive_integer(&object, map_id, &path, MapSummaryField::Width)?;
    let height = required_positive_integer(&object, map_id, &path, MapSummaryField::Height)?;
    let tileset_id = required_positive_integer(&object, map_id, &path, MapSummaryField::TilesetId)?;
    let events = required_node(&object, map_id, &path, MapSummaryField::Events)?;
    let events = events
        .as_array()
        .ok_or_else(|| MapSummaryError::UnexpectedFieldKind {
            map_id,
            path: path.clone(),
            field: MapSummaryField::Events,
            actual: json_kind(&events),
        })?;
    let mut event_count = 0;
    for (index, event) in events.elements().into_iter().enumerate() {
        if event.as_null_keyword().is_some() {
            continue;
        }
        if event.as_object().is_none() {
            return Err(MapSummaryError::UnexpectedEventEntryKind {
                map_id,
                path,
                index,
                actual: json_kind(&event),
            });
        }
        event_count += 1;
    }

    Ok(MapSummary {
        id: map_id,
        catalog_name: record.name().to_owned(),
        document_path: path,
        display_name,
        width,
        height,
        tileset_id,
        event_count,
    })
}

fn required_node(
    object: &CstObject,
    map_id: MapId,
    path: &Path,
    field: MapSummaryField,
) -> Result<CstNode, MapSummaryError> {
    let mut matches = object.properties().into_iter().filter(|property| {
        property
            .name()
            .and_then(|name| name.decoded_value().ok())
            .is_some_and(|name| name == field.name())
    });
    let property = matches
        .next()
        .ok_or_else(|| MapSummaryError::MissingField {
            map_id,
            path: path.to_owned(),
            field,
        })?;
    if matches.next().is_some() {
        return Err(MapSummaryError::DuplicateField {
            map_id,
            path: path.to_owned(),
            field,
        });
    }
    property
        .value()
        .ok_or_else(|| MapSummaryError::MissingField {
            map_id,
            path: path.to_owned(),
            field,
        })
}

fn required_positive_integer(
    object: &CstObject,
    map_id: MapId,
    path: &Path,
    field: MapSummaryField,
) -> Result<NonZeroU32, MapSummaryError> {
    let node = required_node(object, map_id, path, field)?;
    let number = node
        .as_number_lit()
        .ok_or_else(|| MapSummaryError::UnexpectedFieldKind {
            map_id,
            path: path.to_owned(),
            field,
            actual: json_kind(&node),
        })?;
    let value =
        number
            .to_string()
            .parse::<u32>()
            .map_err(|_| MapSummaryError::UnsupportedInteger {
                map_id,
                path: path.to_owned(),
                field,
            })?;
    NonZeroU32::new(value).ok_or_else(|| MapSummaryError::UnsupportedInteger {
        map_id,
        path: path.to_owned(),
        field,
    })
}

fn required_string(
    object: &CstObject,
    map_id: MapId,
    path: &Path,
    field: MapSummaryField,
) -> Result<String, MapSummaryError> {
    let node = required_node(object, map_id, path, field)?;
    let string = node
        .as_string_lit()
        .ok_or_else(|| MapSummaryError::UnexpectedFieldKind {
            map_id,
            path: path.to_owned(),
            field,
            actual: json_kind(&node),
        })?;
    string
        .decoded_value()
        .map_err(|_| MapSummaryError::InvalidString {
            map_id,
            path: path.to_owned(),
            field,
        })
}

fn expected_kind(field: MapSummaryField) -> JsonValueKind {
    match field {
        MapSummaryField::DisplayName => JsonValueKind::String,
        MapSummaryField::Width | MapSummaryField::Height | MapSummaryField::TilesetId => {
            JsonValueKind::Number
        }
        MapSummaryField::Events => JsonValueKind::Array,
    }
}

fn json_kind(node: &CstNode) -> JsonValueKind {
    if node.as_object().is_some() {
        JsonValueKind::Object
    } else if node.as_array().is_some() {
        JsonValueKind::Array
    } else if node.as_string_lit().is_some() {
        JsonValueKind::String
    } else if node.as_number_lit().is_some() {
        JsonValueKind::Number
    } else if node.as_boolean_lit().is_some() {
        JsonValueKind::Boolean
    } else if node.as_null_keyword().is_some() {
        JsonValueKind::Null
    } else {
        JsonValueKind::Unrecognized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpg_maker_mz::snapshot::{SnapshotLimits, load_snapshot};
    use cap_std::fs::Dir;
    use std::fs;
    use std::num::NonZeroUsize;
    use tempfile::TempDir;

    type ErrorPredicate = fn(&MapSummaryError) -> bool;

    fn snapshot(map_infos: &[u8], map_documents: &[(u32, &[u8])]) -> (TempDir, ProjectSnapshot) {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/MapInfos.json"), map_infos).unwrap();
        for (id, source) in map_documents {
            fs::write(temp.path().join(format!("data/Map{id:03}.json")), source).unwrap();
        }
        let root = Dir::open_ambient_dir(temp.path(), cap_std::ambient_authority()).unwrap();
        let loaded = load_snapshot(
            &root,
            SnapshotLimits {
                max_documents: NonZeroUsize::new(32).unwrap(),
                max_bytes_per_document: NonZeroUsize::new(65_536).unwrap(),
                max_aggregate_bytes: NonZeroUsize::new(131_072).unwrap(),
            },
        )
        .unwrap();
        (temp, loaded)
    }

    fn map_infos(ids: &[u32]) -> Vec<u8> {
        let maximum = ids.iter().copied().max().unwrap_or(0) as usize;
        let mut entries = vec!["null".to_string(); maximum + 1];
        for id in ids {
            entries[*id as usize] =
                format!(r#"{{"id":{id},"name":"Catalog {id}","order":{id},"parentId":0}}"#);
        }
        format!("[{}]", entries.join(",")).into_bytes()
    }

    #[test]
    fn summarizes_only_bounded_fields_without_changing_raw_bytes() {
        let source = br#"{"displayName":"Town \u2603","width":20,"height":15,"tilesetId":2,"events":[null,{"id":1,"pages":[{"plugin":true}]},null,{"unknown":1}],"unknown":{"nested":true},"data":"not interpreted"}"#;
        let (_temp, snapshot) = snapshot(&map_infos(&[1]), &[(1, source)]);

        let summary = map_summary(&snapshot, MapId::new(1).unwrap()).unwrap();

        assert_eq!(summary.id(), MapId::new(1).unwrap());
        assert_eq!(summary.catalog_name(), "Catalog 1");
        assert_eq!(summary.document_path(), Path::new("data/Map001.json"));
        assert_eq!(summary.display_name(), "Town ☃");
        assert_eq!(summary.width(), 20);
        assert_eq!(summary.height(), 15);
        assert_eq!(summary.tileset_id(), 2);
        assert_eq!(summary.event_count(), 2);
        assert_eq!(
            snapshot.documents()[Path::new("data/Map001.json")].source_bytes(),
            source
        );
    }

    #[test]
    fn accepts_empty_display_name_and_unrelated_catalog_findings() {
        let infos = br#"[null,{"id":1,"name":"One","order":1,"parentId":0},{"id":2,"name":"Two","order":2,"parentId":9}]"#;
        let source = br#"{"displayName":"","width":1,"height":1,"tilesetId":1,"events":[]}"#;
        let (_temp, snapshot) = snapshot(infos, &[(1, source)]);

        let summary = map_summary(&snapshot, MapId::new(1).unwrap()).unwrap();

        assert_eq!(summary.display_name(), "");
        assert_eq!(summary.event_count(), 0);
    }

    #[test]
    fn distinguishes_catalog_identity_and_document_failures() {
        let (_temp, malformed_catalog) = snapshot(br#"{}"#, &[]);
        assert!(matches!(
            map_summary(&malformed_catalog, MapId::new(1).unwrap()),
            Err(MapSummaryError::Catalog { .. })
        ));

        let (_temp, unknown) = snapshot(&map_infos(&[1]), &[(1, br#"{}"#)]);
        assert!(matches!(
            map_summary(&unknown, MapId::new(2).unwrap()),
            Err(MapSummaryError::UnknownMapId { .. })
        ));

        let (_temp, missing) = snapshot(&map_infos(&[1]), &[]);
        assert!(matches!(
            map_summary(&missing, MapId::new(1).unwrap()),
            Err(MapSummaryError::MissingDocument { .. })
        ));

        let (_temp, unavailable) = snapshot(&map_infos(&[1]), &[(1, b"not json")]);
        assert!(matches!(
            map_summary(&unavailable, MapId::new(1).unwrap()),
            Err(MapSummaryError::UnavailableDocument { .. })
        ));

        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/MapInfos.json"), map_infos(&[1])).unwrap();
        fs::create_dir(temp.path().join("data/Map001.json")).unwrap();
        let root = Dir::open_ambient_dir(temp.path(), cap_std::ambient_authority()).unwrap();
        let non_file = load_snapshot(
            &root,
            SnapshotLimits {
                max_documents: NonZeroUsize::new(4).unwrap(),
                max_bytes_per_document: NonZeroUsize::new(1024).unwrap(),
                max_aggregate_bytes: NonZeroUsize::new(4096).unwrap(),
            },
        )
        .unwrap();
        assert!(matches!(
            map_summary(&non_file, MapId::new(1).unwrap()),
            Err(MapSummaryError::UnavailableDocument { .. })
        ));
    }

    #[test]
    fn refuses_ids_outside_the_evidenced_document_family() {
        let infos = map_infos(&[1_000]);
        let (_temp, snapshot) = snapshot(&infos, &[]);

        assert!(matches!(
            map_summary(&snapshot, MapId::new(1_000).unwrap()),
            Err(MapSummaryError::UnevidencedDocumentPath { .. })
        ));
    }

    #[test]
    fn refuses_ambiguous_or_malformed_required_fields() {
        let cases: &[(&[u8], ErrorPredicate)] = &[
            (br#"[]"#, |error| {
                matches!(error, MapSummaryError::UnexpectedRootKind { .. })
            }),
            (
                br#"{"width":1,"height":1,"tilesetId":1,"events":[]}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::MissingField {
                            field: MapSummaryField::DisplayName,
                            ..
                        }
                    )
                },
            ),
            (
                br#"{"displayName":"A","display\u004eame":"B","width":1,"height":1,"tilesetId":1,"events":[]}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::DuplicateField {
                            field: MapSummaryField::DisplayName,
                            ..
                        }
                    )
                },
            ),
            (
                br#"{"displayName":1,"width":1,"height":1,"tilesetId":1,"events":[]}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::UnexpectedFieldKind {
                            field: MapSummaryField::DisplayName,
                            actual: JsonValueKind::Number,
                            ..
                        }
                    )
                },
            ),
            (
                br#"{"displayName":"A","width":"1","height":1,"tilesetId":1,"events":[]}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::UnexpectedFieldKind {
                            field: MapSummaryField::Width,
                            ..
                        }
                    )
                },
            ),
            (
                br#"{"displayName":"A","width":0,"height":1,"tilesetId":1,"events":[]}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::UnsupportedInteger {
                            field: MapSummaryField::Width,
                            ..
                        }
                    )
                },
            ),
            (
                br#"{"displayName":"A","width":1.0,"height":1,"tilesetId":1,"events":[]}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::UnsupportedInteger {
                            field: MapSummaryField::Width,
                            ..
                        }
                    )
                },
            ),
            (
                br#"{"displayName":"A","width":-1,"height":1,"tilesetId":1,"events":[]}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::UnsupportedInteger {
                            field: MapSummaryField::Width,
                            ..
                        }
                    )
                },
            ),
            (
                br#"{"displayName":"A","width":4294967296,"height":1,"tilesetId":1,"events":[]}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::UnsupportedInteger {
                            field: MapSummaryField::Width,
                            ..
                        }
                    )
                },
            ),
            (
                br#"{"displayName":"A","width":1,"height":0,"tilesetId":1,"events":[]}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::UnsupportedInteger {
                            field: MapSummaryField::Height,
                            ..
                        }
                    )
                },
            ),
            (
                br#"{"displayName":"A","width":1,"height":1,"tilesetId":0,"events":[]}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::UnsupportedInteger {
                            field: MapSummaryField::TilesetId,
                            ..
                        }
                    )
                },
            ),
            (
                br#"{"displayName":"A","width":1,"height":1,"tilesetId":1,"events":{}}"#,
                |error| {
                    matches!(
                        error,
                        MapSummaryError::UnexpectedFieldKind {
                            field: MapSummaryField::Events,
                            ..
                        }
                    )
                },
            ),
        ];

        for (source, expected) in cases {
            let (_temp, snapshot) = snapshot(&map_infos(&[1]), &[(1, source)]);
            let error = map_summary(&snapshot, MapId::new(1).unwrap()).unwrap_err();
            assert!(expected(&error), "unexpected error: {error:?}");
        }
    }

    #[test]
    fn accepts_exact_positive_u32_boundaries() {
        let source = br#"{"displayName":"Boundary","width":4294967295,"height":4294967295,"tilesetId":4294967295,"events":[]}"#;
        let (_temp, snapshot) = snapshot(&map_infos(&[1]), &[(1, source)]);

        let summary = map_summary(&snapshot, MapId::new(1).unwrap()).unwrap();

        assert_eq!(summary.width(), u32::MAX);
        assert_eq!(summary.height(), u32::MAX);
        assert_eq!(summary.tileset_id(), u32::MAX);
    }

    #[test]
    fn catalog_error_remains_available_as_the_error_source() {
        let (_temp, snapshot) = snapshot(br#"{}"#, &[]);

        let error = map_summary(&snapshot, MapId::new(1).unwrap()).unwrap_err();

        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn refuses_non_object_event_entries_with_index_context() {
        let source =
            br#"{"displayName":"A","width":1,"height":1,"tilesetId":1,"events":[null,true]}"#;
        let (_temp, snapshot) = snapshot(&map_infos(&[1]), &[(1, source)]);

        let error = map_summary(&snapshot, MapId::new(1).unwrap()).unwrap_err();

        assert!(matches!(
            error,
            MapSummaryError::UnexpectedEventEntryKind {
                index: 1,
                actual: JsonValueKind::Boolean,
                ..
            }
        ));
    }
}
