// SPDX-License-Identifier: MPL-2.0

//! Experimental, read-only event catalog for one selected RPG Maker MZ map.

use crate::rpg_maker_mz::map_catalog::{
    JsonValueKind, MapCatalogError, MapId, evidenced_map_document_path, map_catalog,
};
use crate::rpg_maker_mz::snapshot::ProjectSnapshot;
use jsonc_parser::cst::{CstNode, CstObject};
use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

/// A positive map-event identifier scoped to one selected map.
///
/// This experimental identifier is not a stable project-wide resource ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MapEventId(NonZeroU32);

impl MapEventId {
    /// Creates an event identifier, returning `None` for the reserved zero value.
    pub fn new(value: u32) -> Option<Self> {
        NonZeroU32::new(value).map(Self)
    }

    /// Returns the positive numeric identifier.
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Display for MapEventId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

/// One bounded event record from a selected map.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MapEventRecord {
    id: MapEventId,
    name: String,
    x: u32,
    y: u32,
    page_count: usize,
}

impl MapEventRecord {
    /// Returns the event ID, scoped to the containing map.
    pub fn id(&self) -> MapEventId {
        self.id
    }

    /// Returns the decoded editor-facing event name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the nonnegative stored horizontal coordinate.
    pub fn x(&self) -> u32 {
        self.x
    }

    /// Returns the nonnegative stored vertical coordinate.
    pub fn y(&self) -> u32 {
        self.y
    }

    /// Returns the number of opaque entries in the event's page array.
    ///
    /// Page bodies and commands are not interpreted by this projection.
    pub fn page_count(&self) -> usize {
        self.page_count
    }
}

/// A structurally coherent event catalog for one selected map.
#[derive(Debug)]
#[non_exhaustive]
pub struct MapEventCatalog {
    map_id: MapId,
    catalog_name: String,
    document_path: PathBuf,
    width: NonZeroU32,
    height: NonZeroU32,
    records: BTreeMap<MapEventId, MapEventRecord>,
    findings: Vec<MapEventFinding>,
}

impl MapEventCatalog {
    /// Returns the selected map ID.
    pub fn map_id(&self) -> MapId {
        self.map_id
    }

    /// Returns the selected map's decoded editor-facing catalog name.
    pub fn catalog_name(&self) -> &str {
        &self.catalog_name
    }

    /// Returns the exact evidenced project-relative map-document path.
    pub fn document_path(&self) -> &Path {
        &self.document_path
    }

    /// Returns the positive map width used for coordinate findings.
    pub fn width(&self) -> u32 {
        self.width.get()
    }

    /// Returns the positive map height used for coordinate findings.
    pub fn height(&self) -> u32 {
        self.height.get()
    }

    /// Returns event records in ascending event-ID order.
    pub fn records(&self) -> &BTreeMap<MapEventId, MapEventRecord> {
        &self.records
    }

    /// Returns one event by its map-scoped ID.
    pub fn get(&self, id: MapEventId) -> Option<&MapEventRecord> {
        self.records.get(&id)
    }

    /// Returns deterministic contextual coordinate findings.
    ///
    /// Findings do not claim that RPG Maker MZ rejects the project.
    pub fn findings(&self) -> &[MapEventFinding] {
        &self.findings
    }
}

/// A contextual finding produced after event structure is coherent.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapEventFinding {
    /// The stored event coordinate is outside the selected map dimensions.
    #[non_exhaustive]
    CoordinatesOutsideMap {
        event_id: MapEventId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

/// A required selected-map field used by the event catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapEventMapField {
    /// The positive map width scalar.
    Width,
    /// The positive map height scalar.
    Height,
    /// The array of event objects and null holes.
    Events,
}

impl MapEventMapField {
    fn name(self) -> &'static str {
        match self {
            Self::Width => "width",
            Self::Height => "height",
            Self::Events => "events",
        }
    }
}

impl fmt::Display for MapEventMapField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// A required field in one map-event object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapEventField {
    /// The positive map-scoped event ID.
    Id,
    /// The decoded editor-facing event name.
    Name,
    /// The nonnegative horizontal coordinate.
    X,
    /// The nonnegative vertical coordinate.
    Y,
    /// The array of opaque event pages.
    Pages,
}

impl MapEventField {
    fn name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::X => "x",
            Self::Y => "y",
            Self::Pages => "pages",
        }
    }
}

impl fmt::Display for MapEventField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Why a selected map could not produce a coherent event catalog.
#[derive(Debug)]
#[non_exhaustive]
pub enum MapEventCatalogError {
    /// The project could not produce a structurally coherent map catalog.
    #[non_exhaustive]
    Catalog { source: MapCatalogError },
    /// The requested ID has no record in the coherent map catalog.
    #[non_exhaustive]
    UnknownMapId { map_id: MapId },
    /// The ID is outside the evidenced three-digit map-document family.
    #[non_exhaustive]
    UnevidencedDocumentPath { map_id: MapId },
    /// No loaded document exists at the expected path.
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
    /// A required selected-map field is absent.
    #[non_exhaustive]
    MissingMapField {
        map_id: MapId,
        path: PathBuf,
        field: MapEventMapField,
    },
    /// A required selected-map field occurs more than once.
    #[non_exhaustive]
    DuplicateMapField {
        map_id: MapId,
        path: PathBuf,
        field: MapEventMapField,
    },
    /// A required selected-map field has the wrong kind.
    #[non_exhaustive]
    UnexpectedMapFieldKind {
        map_id: MapId,
        path: PathBuf,
        field: MapEventMapField,
        actual: JsonValueKind,
    },
    /// A selected-map dimension is not a supported positive integer.
    #[non_exhaustive]
    UnsupportedMapInteger {
        map_id: MapId,
        path: PathBuf,
        field: MapEventMapField,
    },
    /// A non-null event-array entry is not an object.
    #[non_exhaustive]
    UnexpectedEventEntryKind {
        map_id: MapId,
        path: PathBuf,
        index: usize,
        actual: JsonValueKind,
    },
    /// An event object occupies reserved array index zero.
    #[non_exhaustive]
    ReservedEventIndex { map_id: MapId, path: PathBuf },
    /// An event-array index cannot be represented by the identifier type.
    #[non_exhaustive]
    EventIndexOutOfRange {
        map_id: MapId,
        path: PathBuf,
        index: usize,
    },
    /// A required event-object field is absent.
    #[non_exhaustive]
    MissingEventField {
        map_id: MapId,
        path: PathBuf,
        index: usize,
        field: MapEventField,
    },
    /// A required event-object field occurs more than once.
    #[non_exhaustive]
    DuplicateEventField {
        map_id: MapId,
        path: PathBuf,
        index: usize,
        field: MapEventField,
    },
    /// A required event-object field has the wrong kind.
    #[non_exhaustive]
    UnexpectedEventFieldKind {
        map_id: MapId,
        path: PathBuf,
        index: usize,
        field: MapEventField,
        actual: JsonValueKind,
    },
    /// A required event number is outside the supported unsigned form.
    #[non_exhaustive]
    UnsupportedEventInteger {
        map_id: MapId,
        path: PathBuf,
        index: usize,
        field: MapEventField,
    },
    /// An event name string could not be decoded.
    #[non_exhaustive]
    InvalidEventString {
        map_id: MapId,
        path: PathBuf,
        index: usize,
        field: MapEventField,
    },
    /// The decoded event ID does not equal its array index.
    #[non_exhaustive]
    IdIndexMismatch {
        map_id: MapId,
        path: PathBuf,
        index: usize,
        decoded_id: MapEventId,
    },
}

impl fmt::Display for MapEventCatalogError {
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
            Self::MissingDocument { map_id, path, .. } => {
                write!(
                    formatter,
                    "map {map_id} document {} is missing",
                    path.display()
                )
            }
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
            Self::MissingMapField {
                map_id,
                path,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} is missing required field {field}",
                path.display()
            ),
            Self::DuplicateMapField {
                map_id,
                path,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} has duplicate required field {field}",
                path.display()
            ),
            Self::UnexpectedMapFieldKind {
                map_id,
                path,
                field,
                actual,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} field {field} is {actual}, expected {}",
                path.display(),
                expected_map_kind(*field)
            ),
            Self::UnsupportedMapInteger {
                map_id,
                path,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} field {field} is not a supported positive unsigned integer",
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
            Self::ReservedEventIndex { map_id, path, .. } => write!(
                formatter,
                "map {map_id} document {} has an event object at reserved index 0",
                path.display()
            ),
            Self::EventIndexOutOfRange {
                map_id,
                path,
                index,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} event index {index} exceeds the supported identifier range",
                path.display()
            ),
            Self::MissingEventField {
                map_id,
                path,
                index,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} event entry {index} is missing required field {field}",
                path.display()
            ),
            Self::DuplicateEventField {
                map_id,
                path,
                index,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} event entry {index} has duplicate required field {field}",
                path.display()
            ),
            Self::UnexpectedEventFieldKind {
                map_id,
                path,
                index,
                field,
                actual,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} event entry {index} field {field} is {actual}, expected {}",
                path.display(),
                expected_event_kind(*field)
            ),
            Self::UnsupportedEventInteger {
                map_id,
                path,
                index,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} event entry {index} field {field} is not a supported unsigned integer",
                path.display()
            ),
            Self::InvalidEventString {
                map_id,
                path,
                index,
                field,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} event entry {index} field {field} could not be decoded",
                path.display()
            ),
            Self::IdIndexMismatch {
                map_id,
                path,
                index,
                decoded_id,
                ..
            } => write!(
                formatter,
                "map {map_id} document {} event entry {index} has decoded ID {decoded_id}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for MapEventCatalogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Catalog { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Projects one catalog-selected map into a bounded event catalog.
///
/// The operation reads only lossless documents already present in `snapshot`.
/// It exposes map-scoped event IDs, names, coordinates, and opaque page counts.
/// Event notes, page bodies, commands, and unknown fields remain untouched in
/// the raw snapshot.
///
/// Coordinate findings compare stored nonnegative coordinates with the map's
/// positive dimensions; they do not claim editor invalidity. This operation
/// performs no filesystem I/O, mutation, serialization, persistence, page or
/// command interpretation, runtime execution, or compatibility verification.
///
/// # Errors
///
/// Returns [`MapEventCatalogError`] when map selection or required map/event
/// structure is absent, ambiguous, or outside the bounded numeric contract.
pub fn map_event_catalog(
    snapshot: &ProjectSnapshot,
    map_id: MapId,
) -> Result<MapEventCatalog, MapEventCatalogError> {
    let catalog =
        map_catalog(snapshot).map_err(|source| MapEventCatalogError::Catalog { source })?;
    let map_record = catalog
        .get(map_id)
        .ok_or(MapEventCatalogError::UnknownMapId { map_id })?;
    let path = evidenced_map_document_path(map_id)
        .ok_or(MapEventCatalogError::UnevidencedDocumentPath { map_id })?;
    let Some(document) = snapshot.documents().get(path.as_path()) else {
        return if snapshot.diagnostics().contains_key(path.as_path()) {
            Err(MapEventCatalogError::UnavailableDocument { map_id, path })
        } else {
            Err(MapEventCatalogError::MissingDocument { map_id, path })
        };
    };

    let root =
        document
            .cst_root()
            .value()
            .ok_or_else(|| MapEventCatalogError::UnexpectedRootKind {
                map_id,
                path: path.clone(),
                actual: JsonValueKind::Unrecognized,
            })?;
    let object = root
        .as_object()
        .ok_or_else(|| MapEventCatalogError::UnexpectedRootKind {
            map_id,
            path: path.clone(),
            actual: json_kind(&root),
        })?;
    let width = required_map_positive_integer(&object, map_id, &path, MapEventMapField::Width)?;
    let height = required_map_positive_integer(&object, map_id, &path, MapEventMapField::Height)?;
    let events_node = required_map_node(&object, map_id, &path, MapEventMapField::Events)?;
    let events =
        events_node
            .as_array()
            .ok_or_else(|| MapEventCatalogError::UnexpectedMapFieldKind {
                map_id,
                path: path.clone(),
                field: MapEventMapField::Events,
                actual: json_kind(&events_node),
            })?;

    let mut records = BTreeMap::new();
    let mut findings = Vec::new();
    for (index, event_node) in events.elements().into_iter().enumerate() {
        if event_node.as_null_keyword().is_some() {
            continue;
        }
        let event = event_node.as_object().ok_or_else(|| {
            MapEventCatalogError::UnexpectedEventEntryKind {
                map_id,
                path: path.clone(),
                index,
                actual: json_kind(&event_node),
            }
        })?;
        let event_id = event_id_from_index(map_id, &path, index)?;
        let decoded_id =
            required_event_positive_integer(&event, map_id, &path, index, MapEventField::Id)?;
        if decoded_id != event_id {
            return Err(MapEventCatalogError::IdIndexMismatch {
                map_id,
                path,
                index,
                decoded_id,
            });
        }
        let name = required_event_string(&event, map_id, &path, index, MapEventField::Name)?;
        let x = required_event_integer(&event, map_id, &path, index, MapEventField::X)?;
        let y = required_event_integer(&event, map_id, &path, index, MapEventField::Y)?;
        let pages_node = required_event_node(&event, map_id, &path, index, MapEventField::Pages)?;
        let pages = pages_node.as_array().ok_or_else(|| {
            MapEventCatalogError::UnexpectedEventFieldKind {
                map_id,
                path: path.clone(),
                index,
                field: MapEventField::Pages,
                actual: json_kind(&pages_node),
            }
        })?;

        if x >= width.get() || y >= height.get() {
            findings.push(MapEventFinding::CoordinatesOutsideMap {
                event_id,
                x,
                y,
                width: width.get(),
                height: height.get(),
            });
        }
        records.insert(
            event_id,
            MapEventRecord {
                id: event_id,
                name,
                x,
                y,
                page_count: pages.elements().len(),
            },
        );
    }

    Ok(MapEventCatalog {
        map_id,
        catalog_name: map_record.name().to_owned(),
        document_path: path,
        width,
        height,
        records,
        findings,
    })
}

fn event_id_from_index(
    map_id: MapId,
    path: &Path,
    index: usize,
) -> Result<MapEventId, MapEventCatalogError> {
    let value = u32::try_from(index).map_err(|_| MapEventCatalogError::EventIndexOutOfRange {
        map_id,
        path: path.to_owned(),
        index,
    })?;
    MapEventId::new(value).ok_or_else(|| MapEventCatalogError::ReservedEventIndex {
        map_id,
        path: path.to_owned(),
    })
}

fn required_map_node(
    object: &CstObject,
    map_id: MapId,
    path: &Path,
    field: MapEventMapField,
) -> Result<CstNode, MapEventCatalogError> {
    let mut matches = matching_properties(object, field.name());
    let property = matches
        .next()
        .ok_or_else(|| MapEventCatalogError::MissingMapField {
            map_id,
            path: path.to_owned(),
            field,
        })?;
    if matches.next().is_some() {
        return Err(MapEventCatalogError::DuplicateMapField {
            map_id,
            path: path.to_owned(),
            field,
        });
    }
    property
        .value()
        .ok_or_else(|| MapEventCatalogError::MissingMapField {
            map_id,
            path: path.to_owned(),
            field,
        })
}

fn required_map_positive_integer(
    object: &CstObject,
    map_id: MapId,
    path: &Path,
    field: MapEventMapField,
) -> Result<NonZeroU32, MapEventCatalogError> {
    let node = required_map_node(object, map_id, path, field)?;
    let number =
        node.as_number_lit()
            .ok_or_else(|| MapEventCatalogError::UnexpectedMapFieldKind {
                map_id,
                path: path.to_owned(),
                field,
                actual: json_kind(&node),
            })?;
    let value = number
        .to_string()
        .parse::<u32>()
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or_else(|| MapEventCatalogError::UnsupportedMapInteger {
            map_id,
            path: path.to_owned(),
            field,
        })?;
    Ok(value)
}

fn required_event_node(
    object: &CstObject,
    map_id: MapId,
    path: &Path,
    index: usize,
    field: MapEventField,
) -> Result<CstNode, MapEventCatalogError> {
    let mut matches = matching_properties(object, field.name());
    let property = matches
        .next()
        .ok_or_else(|| MapEventCatalogError::MissingEventField {
            map_id,
            path: path.to_owned(),
            index,
            field,
        })?;
    if matches.next().is_some() {
        return Err(MapEventCatalogError::DuplicateEventField {
            map_id,
            path: path.to_owned(),
            index,
            field,
        });
    }
    property
        .value()
        .ok_or_else(|| MapEventCatalogError::MissingEventField {
            map_id,
            path: path.to_owned(),
            index,
            field,
        })
}

fn required_event_integer(
    object: &CstObject,
    map_id: MapId,
    path: &Path,
    index: usize,
    field: MapEventField,
) -> Result<u32, MapEventCatalogError> {
    let node = required_event_node(object, map_id, path, index, field)?;
    let number =
        node.as_number_lit()
            .ok_or_else(|| MapEventCatalogError::UnexpectedEventFieldKind {
                map_id,
                path: path.to_owned(),
                index,
                field,
                actual: json_kind(&node),
            })?;
    number
        .to_string()
        .parse::<u32>()
        .map_err(|_| MapEventCatalogError::UnsupportedEventInteger {
            map_id,
            path: path.to_owned(),
            index,
            field,
        })
}

fn required_event_positive_integer(
    object: &CstObject,
    map_id: MapId,
    path: &Path,
    index: usize,
    field: MapEventField,
) -> Result<MapEventId, MapEventCatalogError> {
    let value = required_event_integer(object, map_id, path, index, field)?;
    MapEventId::new(value).ok_or_else(|| MapEventCatalogError::UnsupportedEventInteger {
        map_id,
        path: path.to_owned(),
        index,
        field,
    })
}

fn required_event_string(
    object: &CstObject,
    map_id: MapId,
    path: &Path,
    index: usize,
    field: MapEventField,
) -> Result<String, MapEventCatalogError> {
    let node = required_event_node(object, map_id, path, index, field)?;
    let string =
        node.as_string_lit()
            .ok_or_else(|| MapEventCatalogError::UnexpectedEventFieldKind {
                map_id,
                path: path.to_owned(),
                index,
                field,
                actual: json_kind(&node),
            })?;
    string
        .decoded_value()
        .map_err(|_| MapEventCatalogError::InvalidEventString {
            map_id,
            path: path.to_owned(),
            index,
            field,
        })
}

fn matching_properties<'a>(
    object: &'a CstObject,
    name: &'a str,
) -> impl Iterator<Item = jsonc_parser::cst::CstObjectProp> + 'a {
    object.properties().into_iter().filter(move |property| {
        property
            .name()
            .and_then(|value| value.decoded_value().ok())
            .is_some_and(|value| value == name)
    })
}

fn expected_map_kind(field: MapEventMapField) -> JsonValueKind {
    match field {
        MapEventMapField::Width | MapEventMapField::Height => JsonValueKind::Number,
        MapEventMapField::Events => JsonValueKind::Array,
    }
}

fn expected_event_kind(field: MapEventField) -> JsonValueKind {
    match field {
        MapEventField::Id | MapEventField::X | MapEventField::Y => JsonValueKind::Number,
        MapEventField::Name => JsonValueKind::String,
        MapEventField::Pages => JsonValueKind::Array,
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

    type ErrorPredicate = fn(&MapEventCatalogError) -> bool;

    fn snapshot_with(
        map_infos: &[u8],
        source: Option<&[u8]>,
        max_bytes_per_document: usize,
    ) -> (TempDir, ProjectSnapshot) {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/MapInfos.json"), map_infos).unwrap();
        if let Some(source) = source {
            fs::write(temp.path().join("data/Map001.json"), source).unwrap();
        }
        let root = Dir::open_ambient_dir(temp.path(), cap_std::ambient_authority()).unwrap();
        let loaded = load_snapshot(
            &root,
            SnapshotLimits {
                max_documents: NonZeroUsize::new(8).unwrap(),
                max_bytes_per_document: NonZeroUsize::new(max_bytes_per_document).unwrap(),
                max_aggregate_bytes: NonZeroUsize::new(131_072).unwrap(),
            },
        )
        .unwrap();
        (temp, loaded)
    }

    fn snapshot(source: &[u8]) -> (TempDir, ProjectSnapshot) {
        snapshot_with(
            br#"[null,{"id":1,"name":"Catalog Map","order":1,"parentId":0}]"#,
            Some(source),
            65_536,
        )
    }

    #[test]
    fn projects_bounded_records_in_id_order_and_preserves_raw_bytes() {
        let source = br#"{"width":10,"height":8,"events":[null,{"id":1,"name":"Door \u2603","note":"private memo","x":0,"y":7,"pages":[{"unknown":true},{"commands":[1]}],"plugin":{"value":1}},null,{"id":3,"name":"Chest","x":9,"y":0,"pages":[]}],"unknown":true}"#;
        let (_temp, snapshot) = snapshot(source);

        let catalog = map_event_catalog(&snapshot, MapId::new(1).unwrap()).unwrap();

        assert_eq!(catalog.map_id(), MapId::new(1).unwrap());
        assert_eq!(catalog.catalog_name(), "Catalog Map");
        assert_eq!(catalog.document_path(), Path::new("data/Map001.json"));
        assert_eq!((catalog.width(), catalog.height()), (10, 8));
        assert!(catalog.findings().is_empty());
        let ids: Vec<_> = catalog.records().keys().map(|id| id.get()).collect();
        assert_eq!(ids, [1, 3]);
        let first = catalog.get(MapEventId::new(1).unwrap()).unwrap();
        assert_eq!(first.name(), "Door ☃");
        assert_eq!((first.x(), first.y(), first.page_count()), (0, 7, 2));
        assert_eq!(
            catalog
                .get(MapEventId::new(3).unwrap())
                .unwrap()
                .page_count(),
            0
        );
        assert_eq!(
            snapshot.documents()[Path::new("data/Map001.json")].source_bytes(),
            source
        );
    }

    #[test]
    fn reports_out_of_bounds_coordinates_without_losing_records() {
        let source = br#"{"width":10,"height":8,"events":[null,{"id":1,"name":"Edge","x":10,"y":8,"pages":[{}]}]}"#;
        let (_temp, snapshot) = snapshot(source);

        let catalog = map_event_catalog(&snapshot, MapId::new(1).unwrap()).unwrap();

        assert_eq!(catalog.records().len(), 1);
        assert_eq!(
            catalog.findings(),
            [MapEventFinding::CoordinatesOutsideMap {
                event_id: MapEventId::new(1).unwrap(),
                x: 10,
                y: 8,
                width: 10,
                height: 8,
            }]
        );
    }

    #[test]
    fn refuses_reserved_index_and_id_index_mismatch() {
        let (_temp, reserved_snapshot) = snapshot(
            br#"{"width":1,"height":1,"events":[{"id":1,"name":"A","x":0,"y":0,"pages":[]}]}"#,
        );
        assert!(matches!(
            map_event_catalog(&reserved_snapshot, MapId::new(1).unwrap()),
            Err(MapEventCatalogError::ReservedEventIndex { .. })
        ));

        let (_temp, mismatch_snapshot) = snapshot(
            br#"{"width":1,"height":1,"events":[null,{"id":2,"name":"A","x":0,"y":0,"pages":[]}]}"#,
        );
        assert!(matches!(
            map_event_catalog(&mismatch_snapshot, MapId::new(1).unwrap()),
            Err(MapEventCatalogError::IdIndexMismatch {
                index: 1,
                decoded_id,
                ..
            }) if decoded_id == MapEventId::new(2).unwrap()
        ));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn refuses_missing_duplicate_and_wrong_kind_event_fields() {
        let cases: &[(&[u8], ErrorPredicate)] = &[
            (
                br#"{"width":1,"height":1,"events":[null,{"id":1,"x":0,"y":0,"pages":[]}]}"#,
                |error| matches!(error, MapEventCatalogError::MissingEventField { field: MapEventField::Name, .. }),
            ),
            (
                br#"{"width":1,"height":1,"events":[null,{"id":1,"name":"A","n\u0061me":"B","x":0,"y":0,"pages":[]}]}"#,
                |error| matches!(error, MapEventCatalogError::DuplicateEventField { field: MapEventField::Name, .. }),
            ),
            (
                br#"{"width":1,"height":1,"events":[null,{"id":1,"name":false,"x":0,"y":0,"pages":[]}]}"#,
                |error| matches!(error, MapEventCatalogError::UnexpectedEventFieldKind { field: MapEventField::Name, actual: JsonValueKind::Boolean, .. }),
            ),
            (
                br#"{"width":1,"height":1,"events":[null,{"id":1,"name":"A","x":0,"y":0,"pages":{}}]}"#,
                |error| matches!(error, MapEventCatalogError::UnexpectedEventFieldKind { field: MapEventField::Pages, actual: JsonValueKind::Object, .. }),
            ),
        ];

        for (source, predicate) in cases {
            let (_temp, snapshot) = snapshot(source);
            let error = map_event_catalog(&snapshot, MapId::new(1).unwrap()).unwrap_err();
            assert!(predicate(&error), "unexpected error: {error}");
        }
    }

    #[test]
    fn refuses_unsupported_integers_and_non_object_entries() {
        for source in [
            br#"{"width":1,"height":1,"events":[null,{"id":0,"name":"A","x":0,"y":0,"pages":[]}]}"#.as_slice(),
            br#"{"width":1,"height":1,"events":[null,{"id":1,"name":"A","x":-1,"y":0,"pages":[]}]}"#.as_slice(),
            br#"{"width":1,"height":1,"events":[null,{"id":1,"name":"A","x":1.0,"y":0,"pages":[]}]}"#.as_slice(),
            br#"{"width":1,"height":1,"events":[null,{"id":1,"name":"A","x":4294967296,"y":0,"pages":[]}]}"#.as_slice(),
        ] {
            let (_temp, snapshot) = snapshot(source);
            assert!(matches!(
                map_event_catalog(&snapshot, MapId::new(1).unwrap()),
                Err(MapEventCatalogError::UnsupportedEventInteger { .. })
            ));
        }

        let (_temp, snapshot) = snapshot(br#"{"width":1,"height":1,"events":[null,true]}"#);
        assert!(matches!(
            map_event_catalog(&snapshot, MapId::new(1).unwrap()),
            Err(MapEventCatalogError::UnexpectedEventEntryKind {
                index: 1,
                actual: JsonValueKind::Boolean,
                ..
            })
        ));
    }

    #[test]
    fn refuses_ambiguous_map_structure_and_unknown_map() {
        let (_temp, snapshot) = snapshot(br#"{"width":1,"w\u0069dth":2,"height":1,"events":[]}"#);
        assert!(matches!(
            map_event_catalog(&snapshot, MapId::new(1).unwrap()),
            Err(MapEventCatalogError::DuplicateMapField {
                field: MapEventMapField::Width,
                ..
            })
        ));
        assert!(matches!(
            map_event_catalog(&snapshot, MapId::new(2).unwrap()),
            Err(MapEventCatalogError::UnknownMapId { .. })
        ));
    }

    #[test]
    fn distinguishes_catalog_identity_and_document_failures() {
        let (_temp, malformed_catalog) = snapshot_with(br#"{}"#, None, 65_536);
        let catalog_error =
            map_event_catalog(&malformed_catalog, MapId::new(1).unwrap()).unwrap_err();
        assert!(matches!(
            catalog_error,
            MapEventCatalogError::Catalog { .. }
        ));
        assert!(std::error::Error::source(&catalog_error).is_some());

        let (_temp, missing) = snapshot_with(
            br#"[null,{"id":1,"name":"One","order":1,"parentId":0}]"#,
            None,
            65_536,
        );
        assert!(matches!(
            map_event_catalog(&missing, MapId::new(1).unwrap()),
            Err(MapEventCatalogError::MissingDocument { .. })
        ));

        let long_map = format!(
            r#"{{"width":1,"height":1,"events":[],"padding":"{}"}}"#,
            "x".repeat(256)
        );
        let (_temp, unavailable) = snapshot_with(
            br#"[null,{"id":1,"name":"One","order":1,"parentId":0}]"#,
            Some(long_map.as_bytes()),
            96,
        );
        assert!(matches!(
            map_event_catalog(&unavailable, MapId::new(1).unwrap()),
            Err(MapEventCatalogError::UnavailableDocument { .. })
        ));

        let mut entries = vec!["null"; 1_001];
        entries[1_000] = r#"{"id":1000,"name":"Far","order":1,"parentId":0}"#;
        let infos = format!("[{}]", entries.join(","));
        let (_temp, outside_family) = snapshot_with(infos.as_bytes(), None, 65_536);
        assert!(matches!(
            map_event_catalog(&outside_family, MapId::new(1_000).unwrap()),
            Err(MapEventCatalogError::UnevidencedDocumentPath { .. })
        ));
    }

    #[test]
    fn refuses_malformed_required_map_fields() {
        let cases: &[(&[u8], ErrorPredicate)] = &[
            (br#"[]"#, |error| {
                matches!(error, MapEventCatalogError::UnexpectedRootKind { .. })
            }),
            (br#"{"height":1,"events":[]}"#, |error| {
                matches!(
                    error,
                    MapEventCatalogError::MissingMapField {
                        field: MapEventMapField::Width,
                        ..
                    }
                )
            }),
            (br#"{"width":"1","height":1,"events":[]}"#, |error| {
                matches!(
                    error,
                    MapEventCatalogError::UnexpectedMapFieldKind {
                        field: MapEventMapField::Width,
                        actual: JsonValueKind::String,
                        ..
                    }
                )
            }),
            (br#"{"width":0,"height":1,"events":[]}"#, |error| {
                matches!(
                    error,
                    MapEventCatalogError::UnsupportedMapInteger {
                        field: MapEventMapField::Width,
                        ..
                    }
                )
            }),
            (br#"{"width":1,"height":1.0,"events":[]}"#, |error| {
                matches!(
                    error,
                    MapEventCatalogError::UnsupportedMapInteger {
                        field: MapEventMapField::Height,
                        ..
                    }
                )
            }),
            (br#"{"width":1,"height":1,"events":{}}"#, |error| {
                matches!(
                    error,
                    MapEventCatalogError::UnexpectedMapFieldKind {
                        field: MapEventMapField::Events,
                        actual: JsonValueKind::Object,
                        ..
                    }
                )
            }),
        ];

        for (source, predicate) in cases {
            let (_temp, snapshot) = snapshot(source);
            let error = map_event_catalog(&snapshot, MapId::new(1).unwrap()).unwrap_err();
            assert!(predicate(&error), "unexpected error: {error}");
        }
    }

    #[test]
    fn accepts_unsigned_coordinate_boundaries() {
        let source = br#"{"width":4294967295,"height":4294967295,"events":[null,{"id":1,"name":"Boundary","x":4294967294,"y":4294967294,"pages":[]}]}"#;
        let (_temp, snapshot) = snapshot(source);

        let catalog = map_event_catalog(&snapshot, MapId::new(1).unwrap()).unwrap();
        let event = catalog.get(MapEventId::new(1).unwrap()).unwrap();

        assert_eq!(catalog.width(), u32::MAX);
        assert_eq!(catalog.height(), u32::MAX);
        assert_eq!(event.x(), u32::MAX - 1);
        assert_eq!(event.y(), u32::MAX - 1);
        assert!(catalog.findings().is_empty());
    }

    #[test]
    fn checks_index_conversion_without_allocating_a_huge_array() {
        let map_id = MapId::new(1).unwrap();
        assert!(matches!(
            event_id_from_index(map_id, Path::new("data/Map001.json"), usize::MAX),
            Err(MapEventCatalogError::EventIndexOutOfRange { .. })
        ));
    }
}
