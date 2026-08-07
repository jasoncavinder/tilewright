// SPDX-License-Identifier: MPL-2.0

//! Experimental, read-only typed projection of RPG Maker MZ map information.

use crate::rpg_maker_mz::inventory::{
    InventoryClassification, InventoryEntryKind, KnownEntryFamily,
};
use crate::rpg_maker_mz::snapshot::ProjectSnapshot;
use jsonc_parser::cst::{CstNode, CstObject};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

const MAP_INFOS_PATH: &str = "data/MapInfos.json";

/// A positive RPG Maker MZ map identifier.
///
/// This identifier is experimental and scoped to the map-catalog projection.
/// It is not yet a stable project-wide resource identity contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MapId(NonZeroU32);

impl MapId {
    /// Creates a map identifier, returning `None` for the reserved zero value.
    pub fn new(value: u32) -> Option<Self> {
        NonZeroU32::new(value).map(Self)
    }

    /// Returns the positive numeric identifier.
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Display for MapId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

/// One structurally understood `MapInfos.json` record.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MapRecord {
    id: MapId,
    name: String,
    order: NonZeroU32,
    parent_id: Option<MapId>,
}

impl MapRecord {
    /// Returns the record's positive map identifier.
    pub fn id(&self) -> MapId {
        self.id
    }

    /// Returns the decoded editor-facing map name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the positive editor display-order value.
    pub fn order(&self) -> u32 {
        self.order.get()
    }

    /// Returns the parent map, or `None` for a top-level map.
    pub fn parent_id(&self) -> Option<MapId> {
        self.parent_id
    }
}

/// A structurally coherent, read-only map catalog projection.
#[derive(Debug)]
#[non_exhaustive]
pub struct MapCatalog {
    records: BTreeMap<MapId, MapRecord>,
    display_order: Vec<MapId>,
    findings: Vec<MapCatalogFinding>,
}

impl MapCatalog {
    /// Returns records in ascending map-ID order.
    pub fn records(&self) -> &BTreeMap<MapId, MapRecord> {
        &self.records
    }

    /// Returns one record by map ID.
    pub fn get(&self, id: MapId) -> Option<&MapRecord> {
        self.records.get(&id)
    }

    /// Iterates records in display-order value, then map-ID order.
    pub fn records_in_display_order(&self) -> impl Iterator<Item = &MapRecord> {
        self.display_order
            .iter()
            .filter_map(|id| self.records.get(id))
    }

    /// Returns deterministic contextual findings.
    ///
    /// Findings describe project relationships and Tilewright's evidence
    /// boundary. They do not claim that RPG Maker MZ rejects the project.
    pub fn findings(&self) -> &[MapCatalogFinding] {
        &self.findings
    }
}

/// A contextual map-catalog finding after structural projection succeeds.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapCatalogFinding {
    /// A record names a parent that is absent from the catalog.
    #[non_exhaustive]
    MissingParent { map_id: MapId, parent_id: MapId },
    /// One or more parent relationships form a cycle.
    #[non_exhaustive]
    ParentCycle { map_ids: Vec<MapId> },
    /// Multiple maps use the same positive display-order value.
    #[non_exhaustive]
    DuplicateOrder { order: u32, map_ids: Vec<MapId> },
    /// No inventory entry exists at the evidenced three-digit map path.
    #[non_exhaustive]
    MissingMapDocument {
        map_id: MapId,
        expected_path: PathBuf,
    },
    /// An evidenced three-digit map path has no matching catalog record.
    #[non_exhaustive]
    OrphanMapDocument { map_id: MapId, path: PathBuf },
    /// A classified map-document path does not encode a positive map ID.
    #[non_exhaustive]
    UnrecognizedMapDocumentIdentity { path: PathBuf },
    /// The map ID is outside the evidenced three-digit filename relationship.
    #[non_exhaustive]
    UnevidencedMapDocumentPath { map_id: MapId },
}

/// A required field in the first map-catalog projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapInfoField {
    /// The positive map identifier.
    Id,
    /// The decoded map name.
    Name,
    /// The positive display-order value.
    Order,
    /// Zero for a top-level map or a positive parent map identifier.
    ParentId,
}

impl MapInfoField {
    fn name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::Order => "order",
            Self::ParentId => "parentId",
        }
    }
}

impl fmt::Display for MapInfoField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// A backend-neutral JSON value kind used by typed projection errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JsonValueKind {
    /// An object value.
    Object,
    /// An array value.
    Array,
    /// A string value.
    String,
    /// A number value.
    Number,
    /// A boolean value.
    Boolean,
    /// A null value.
    Null,
    /// A value kind not recognized by this experimental adapter.
    Unrecognized,
}

impl fmt::Display for JsonValueKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Object => "object",
            Self::Array => "array",
            Self::String => "string",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Null => "null",
            Self::Unrecognized => "unrecognized",
        };
        formatter.write_str(name)
    }
}

/// Why a raw snapshot could not be projected into a coherent map catalog.
#[derive(Debug)]
#[non_exhaustive]
pub enum MapCatalogError {
    /// The snapshot inventory does not contain `data/MapInfos.json`.
    MissingDocument,
    /// The snapshot has a diagnostic instead of a loaded map-info document.
    UnavailableDocument,
    /// The map-info root is not an array.
    #[non_exhaustive]
    UnexpectedRootKind { actual: JsonValueKind },
    /// A non-null array entry is not an object.
    #[non_exhaustive]
    UnexpectedEntryKind { index: usize, actual: JsonValueKind },
    /// A required field is absent.
    #[non_exhaustive]
    MissingField { index: usize, field: MapInfoField },
    /// A required decoded field name occurs more than once.
    #[non_exhaustive]
    DuplicateField { index: usize, field: MapInfoField },
    /// A required field has the wrong JSON value kind.
    #[non_exhaustive]
    UnexpectedFieldKind {
        index: usize,
        field: MapInfoField,
        actual: JsonValueKind,
    },
    /// A required number does not use the supported unsigned integer form.
    #[non_exhaustive]
    UnsupportedInteger { index: usize, field: MapInfoField },
    /// A string field could not be decoded.
    #[non_exhaustive]
    InvalidString { index: usize, field: MapInfoField },
    /// The array index cannot be represented by the experimental identifier.
    #[non_exhaustive]
    IndexOutOfRange { index: usize },
    /// The decoded positive ID does not equal its array index.
    #[non_exhaustive]
    IdIndexMismatch { index: usize, decoded_id: MapId },
}

impl fmt::Display for MapCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDocument => formatter.write_str("data/MapInfos.json is missing"),
            Self::UnavailableDocument => formatter
                .write_str("data/MapInfos.json was not loaded; inspect snapshot diagnostics"),
            Self::UnexpectedRootKind { actual, .. } => {
                write!(
                    formatter,
                    "data/MapInfos.json root is {actual}, expected array"
                )
            }
            Self::UnexpectedEntryKind { index, actual, .. } => {
                write!(
                    formatter,
                    "map-info entry {index} is {actual}, expected object or null"
                )
            }
            Self::MissingField { index, field, .. } => {
                write!(
                    formatter,
                    "map-info entry {index} is missing required field {field}"
                )
            }
            Self::DuplicateField { index, field, .. } => write!(
                formatter,
                "map-info entry {index} has duplicate required field {field}"
            ),
            Self::UnexpectedFieldKind {
                index,
                field,
                actual,
                ..
            } => write!(
                formatter,
                "map-info entry {index} field {field} is {actual}, expected {}",
                expected_kind(*field)
            ),
            Self::UnsupportedInteger { index, field, .. } => write!(
                formatter,
                "map-info entry {index} field {field} is not a supported unsigned integer"
            ),
            Self::InvalidString { index, field, .. } => {
                write!(
                    formatter,
                    "map-info entry {index} field {field} could not be decoded"
                )
            }
            Self::IndexOutOfRange { index, .. } => {
                write!(
                    formatter,
                    "map-info array index {index} exceeds the supported ID range"
                )
            }
            Self::IdIndexMismatch {
                index, decoded_id, ..
            } => write!(
                formatter,
                "map-info entry {index} has ID {decoded_id}, expected its array index"
            ),
        }
    }
}

impl std::error::Error for MapCatalogError {}

/// Projects one loaded snapshot into an experimental typed map catalog.
///
/// This function reads only the already loaded lossless `data/MapInfos.json`
/// document and retained inventory. It does not perform filesystem I/O, expose
/// CST nodes, mutate raw data, validate editor compatibility, or persist
/// changes. Unknown fields remain untouched in the snapshot.
///
/// # Errors
///
/// Returns [`MapCatalogError`] when the map-info document is missing or
/// unavailable, or when its required structure is ambiguous or outside the
/// bounded typed contract.
pub fn map_catalog(snapshot: &ProjectSnapshot) -> Result<MapCatalog, MapCatalogError> {
    let map_infos_path = Path::new(MAP_INFOS_PATH);
    let Some(document) = snapshot.documents().get(map_infos_path) else {
        return if snapshot.diagnostics().contains_key(map_infos_path) {
            Err(MapCatalogError::UnavailableDocument)
        } else {
            Err(MapCatalogError::MissingDocument)
        };
    };

    let root = document
        .cst_root()
        .value()
        .ok_or(MapCatalogError::UnexpectedRootKind {
            actual: JsonValueKind::Unrecognized,
        })?;
    let array = root
        .as_array()
        .ok_or_else(|| MapCatalogError::UnexpectedRootKind {
            actual: json_kind(&root),
        })?;

    let mut records = BTreeMap::new();
    for (index, node) in array.elements().into_iter().enumerate() {
        if node.as_null_keyword().is_some() {
            continue;
        }
        let object = node
            .as_object()
            .ok_or_else(|| MapCatalogError::UnexpectedEntryKind {
                index,
                actual: json_kind(&node),
            })?;

        let index_value =
            u32::try_from(index).map_err(|_| MapCatalogError::IndexOutOfRange { index })?;
        let decoded_id = required_positive_integer(&object, index, MapInfoField::Id)?;
        let id = MapId(decoded_id);
        if id.get() != index_value {
            return Err(MapCatalogError::IdIndexMismatch {
                index,
                decoded_id: id,
            });
        }

        let name = required_string(&object, index, MapInfoField::Name)?;
        let order = required_positive_integer(&object, index, MapInfoField::Order)?;
        let parent_value = required_integer(&object, index, MapInfoField::ParentId)?;
        let parent_id = NonZeroU32::new(parent_value).map(MapId);

        records.insert(
            id,
            MapRecord {
                id,
                name,
                order,
                parent_id,
            },
        );
    }

    let mut display_order: Vec<_> = records.keys().copied().collect();
    display_order.sort_by_key(|id| (records[id].order, *id));
    let findings = collect_findings(snapshot, &records);

    Ok(MapCatalog {
        records,
        display_order,
        findings,
    })
}

fn required_node(
    object: &CstObject,
    index: usize,
    field: MapInfoField,
) -> Result<CstNode, MapCatalogError> {
    let mut matches = object.properties().into_iter().filter(|property| {
        property
            .name()
            .and_then(|name| name.decoded_value().ok())
            .is_some_and(|name| name == field.name())
    });
    let property = matches
        .next()
        .ok_or(MapCatalogError::MissingField { index, field })?;
    if matches.next().is_some() {
        return Err(MapCatalogError::DuplicateField { index, field });
    }
    property
        .value()
        .ok_or(MapCatalogError::MissingField { index, field })
}

fn required_integer(
    object: &CstObject,
    index: usize,
    field: MapInfoField,
) -> Result<u32, MapCatalogError> {
    let node = required_node(object, index, field)?;
    let number = node
        .as_number_lit()
        .ok_or_else(|| MapCatalogError::UnexpectedFieldKind {
            index,
            field,
            actual: json_kind(&node),
        })?;
    number
        .to_string()
        .parse::<u32>()
        .map_err(|_| MapCatalogError::UnsupportedInteger { index, field })
}

fn required_positive_integer(
    object: &CstObject,
    index: usize,
    field: MapInfoField,
) -> Result<NonZeroU32, MapCatalogError> {
    let value = required_integer(object, index, field)?;
    NonZeroU32::new(value).ok_or(MapCatalogError::UnsupportedInteger { index, field })
}

fn required_string(
    object: &CstObject,
    index: usize,
    field: MapInfoField,
) -> Result<String, MapCatalogError> {
    let node = required_node(object, index, field)?;
    let string = node
        .as_string_lit()
        .ok_or_else(|| MapCatalogError::UnexpectedFieldKind {
            index,
            field,
            actual: json_kind(&node),
        })?;
    string
        .decoded_value()
        .map_err(|_| MapCatalogError::InvalidString { index, field })
}

fn expected_kind(field: MapInfoField) -> JsonValueKind {
    match field {
        MapInfoField::Name => JsonValueKind::String,
        MapInfoField::Id | MapInfoField::Order | MapInfoField::ParentId => JsonValueKind::Number,
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

fn collect_findings(
    snapshot: &ProjectSnapshot,
    records: &BTreeMap<MapId, MapRecord>,
) -> Vec<MapCatalogFinding> {
    let mut findings = Vec::new();

    for record in records.values() {
        if let Some(parent_id) = record.parent_id
            && !records.contains_key(&parent_id)
        {
            findings.push(MapCatalogFinding::MissingParent {
                map_id: record.id,
                parent_id,
            });
        }
    }

    for cycle in parent_cycles(records) {
        findings.push(MapCatalogFinding::ParentCycle { map_ids: cycle });
    }

    let mut by_order: BTreeMap<u32, Vec<MapId>> = BTreeMap::new();
    for record in records.values() {
        by_order.entry(record.order()).or_default().push(record.id);
    }
    for (order, map_ids) in by_order {
        if map_ids.len() > 1 {
            findings.push(MapCatalogFinding::DuplicateOrder { order, map_ids });
        }
    }

    let inventory_paths: BTreeSet<_> = snapshot
        .inventory()
        .entries
        .iter()
        .map(|entry| entry.path.as_path())
        .collect();
    for id in records.keys().copied() {
        if id.get() > 999 {
            findings.push(MapCatalogFinding::UnevidencedMapDocumentPath { map_id: id });
            continue;
        }
        let expected_path = PathBuf::from("data").join(format!("Map{:03}.json", id.get()));
        if !inventory_paths.contains(expected_path.as_path()) {
            findings.push(MapCatalogFinding::MissingMapDocument {
                map_id: id,
                expected_path,
            });
        }
    }

    for entry in &snapshot.inventory().entries {
        if entry.kind != InventoryEntryKind::File
            || !matches!(
                entry.classification,
                InventoryClassification::Known {
                    family: KnownEntryFamily::MapDataFile,
                    ..
                }
            )
        {
            continue;
        }
        let Some(id) = map_id_from_evidenced_path(&entry.path) else {
            findings.push(MapCatalogFinding::UnrecognizedMapDocumentIdentity {
                path: entry.path.clone(),
            });
            continue;
        };
        if !records.contains_key(&id) {
            findings.push(MapCatalogFinding::OrphanMapDocument {
                map_id: id,
                path: entry.path.clone(),
            });
        }
    }

    findings
}

fn parent_cycles(records: &BTreeMap<MapId, MapRecord>) -> BTreeSet<Vec<MapId>> {
    let mut cycles = BTreeSet::new();
    let mut completed = BTreeSet::new();
    for start in records.keys().copied() {
        if completed.contains(&start) {
            continue;
        }
        let mut path = Vec::new();
        let mut positions = BTreeMap::new();
        let mut current = Some(start);
        while let Some(id) = current {
            if completed.contains(&id) {
                break;
            }
            let Some(record) = records.get(&id) else {
                break;
            };
            if let Some(position) = positions.get(&id).copied() {
                let mut cycle = path[position..].to_vec();
                if let Some((minimum_index, _)) =
                    cycle.iter().enumerate().min_by_key(|(_, map_id)| **map_id)
                {
                    cycle.rotate_left(minimum_index);
                }
                cycles.insert(cycle);
                break;
            }
            positions.insert(id, path.len());
            path.push(id);
            current = record.parent_id;
        }
        completed.extend(path);
    }
    cycles
}

fn map_id_from_evidenced_path(path: &Path) -> Option<MapId> {
    let name = path.file_name()?.to_str()?;
    let digits = name.strip_prefix("Map")?.strip_suffix(".json")?;
    if digits.len() != 3 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    MapId::new(digits.parse().ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpg_maker_mz::snapshot::{SnapshotLimits, load_snapshot};
    use cap_std::fs::Dir;
    use std::fs;
    use std::num::NonZeroUsize;
    use tempfile::TempDir;

    type ErrorPredicate = fn(&MapCatalogError) -> bool;

    fn snapshot(map_infos: &[u8], map_ids: &[u32]) -> (TempDir, ProjectSnapshot) {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join(MAP_INFOS_PATH), map_infos).unwrap();
        for id in map_ids {
            fs::write(temp.path().join(format!("data/Map{id:03}.json")), b"{}").unwrap();
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

    #[test]
    fn projects_top_level_and_child_records_without_changing_raw_bytes() {
        let source = br#"[null,{"id":1,"name":"Root","order":1,"parentId":0,"unknown":1},{"id":2,"name":"Child \u2603","order":2,"parentId":1,"quick":true}]"#;
        let (_temp, snapshot) = snapshot(source, &[1, 2]);

        let catalog = map_catalog(&snapshot).unwrap();

        assert_eq!(catalog.records().len(), 2);
        let root = catalog.get(MapId::new(1).unwrap()).unwrap();
        assert_eq!(root.name(), "Root");
        assert_eq!(root.order(), 1);
        assert_eq!(root.parent_id(), None);
        let child = catalog.get(MapId::new(2).unwrap()).unwrap();
        assert_eq!(child.name(), "Child ☃");
        assert_eq!(child.parent_id(), MapId::new(1));
        assert!(catalog.findings().is_empty());
        assert_eq!(
            snapshot.documents()[Path::new(MAP_INFOS_PATH)].source_bytes(),
            source
        );
    }

    #[test]
    fn accepts_null_holes_and_orders_records_by_order_then_id() {
        let source = br#"[null,{"id":1,"name":"Later","order":2,"parentId":0},null,{"id":3,"name":"Earlier","order":1,"parentId":0}]"#;
        let (_temp, snapshot) = snapshot(source, &[1, 3]);

        let catalog = map_catalog(&snapshot).unwrap();
        let ids: Vec<_> = catalog
            .records_in_display_order()
            .map(MapRecord::id)
            .collect();

        assert_eq!(ids, [MapId::new(3).unwrap(), MapId::new(1).unwrap()]);
    }

    #[test]
    fn distinguishes_missing_and_unavailable_documents() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        let root = Dir::open_ambient_dir(temp.path(), cap_std::ambient_authority()).unwrap();
        let limits = SnapshotLimits {
            max_documents: NonZeroUsize::new(4).unwrap(),
            max_bytes_per_document: NonZeroUsize::new(1024).unwrap(),
            max_aggregate_bytes: NonZeroUsize::new(4096).unwrap(),
        };
        let missing = load_snapshot(&root, limits).unwrap();
        assert!(matches!(
            map_catalog(&missing),
            Err(MapCatalogError::MissingDocument)
        ));

        fs::write(temp.path().join(MAP_INFOS_PATH), b"not json").unwrap();
        let unavailable = load_snapshot(&root, limits).unwrap();
        assert!(matches!(
            map_catalog(&unavailable),
            Err(MapCatalogError::UnavailableDocument)
        ));
    }

    #[test]
    fn refuses_ambiguous_or_malformed_required_structure() {
        let cases: &[(&[u8], ErrorPredicate)] = &[
            (br#"{}"#, |error| {
                matches!(error, MapCatalogError::UnexpectedRootKind { .. })
            }),
            (br#"[null,1]"#, |error| {
                matches!(error, MapCatalogError::UnexpectedEntryKind { index: 1, .. })
            }),
            (br#"[null,{"name":"A","order":1,"parentId":0}]"#, |error| {
                matches!(
                    error,
                    MapCatalogError::MissingField {
                        index: 1,
                        field: MapInfoField::Id,
                        ..
                    }
                )
            }),
            (
                br#"[null,{"id":1,"id":1,"name":"A","order":1,"parentId":0}]"#,
                |error| {
                    matches!(
                        error,
                        MapCatalogError::DuplicateField {
                            index: 1,
                            field: MapInfoField::Id,
                            ..
                        }
                    )
                },
            ),
            (
                br#"[null,{"id":1,"name":1,"order":1,"parentId":0}]"#,
                |error| {
                    matches!(
                        error,
                        MapCatalogError::UnexpectedFieldKind {
                            index: 1,
                            field: MapInfoField::Name,
                            ..
                        }
                    )
                },
            ),
            (
                br#"[null,{"id":1,"name":"A","order":1.0,"parentId":0}]"#,
                |error| {
                    matches!(
                        error,
                        MapCatalogError::UnsupportedInteger {
                            index: 1,
                            field: MapInfoField::Order,
                            ..
                        }
                    )
                },
            ),
            (
                br#"[null,{"id":2,"name":"A","order":1,"parentId":0}]"#,
                |error| matches!(error, MapCatalogError::IdIndexMismatch { index: 1, .. }),
            ),
        ];

        for (source, expected) in cases {
            let (_temp, snapshot) = snapshot(source, &[]);
            let error = map_catalog(&snapshot).unwrap_err();
            assert!(expected(&error), "unexpected error: {error:?}");
        }
    }

    #[test]
    fn reports_relationship_and_map_document_findings_deterministically() {
        let source = br#"[null,{"id":1,"name":"One","order":1,"parentId":2},{"id":2,"name":"Two","order":1,"parentId":1},null]"#;
        let (_temp, snapshot) = snapshot(source, &[0, 2, 3]);

        let catalog = map_catalog(&snapshot).unwrap();

        assert_eq!(
            catalog.findings(),
            &[
                MapCatalogFinding::ParentCycle {
                    map_ids: vec![MapId::new(1).unwrap(), MapId::new(2).unwrap()],
                },
                MapCatalogFinding::DuplicateOrder {
                    order: 1,
                    map_ids: vec![MapId::new(1).unwrap(), MapId::new(2).unwrap()],
                },
                MapCatalogFinding::MissingMapDocument {
                    map_id: MapId::new(1).unwrap(),
                    expected_path: PathBuf::from("data/Map001.json"),
                },
                MapCatalogFinding::UnrecognizedMapDocumentIdentity {
                    path: PathBuf::from("data/Map000.json"),
                },
                MapCatalogFinding::OrphanMapDocument {
                    map_id: MapId::new(3).unwrap(),
                    path: PathBuf::from("data/Map003.json"),
                },
            ]
        );
    }

    #[test]
    fn reports_missing_parent_without_refusing_catalog() {
        let source = br#"[null,{"id":1,"name":"Orphan","order":1,"parentId":2}]"#;
        let (_temp, snapshot) = snapshot(source, &[1]);

        let catalog = map_catalog(&snapshot).unwrap();

        assert_eq!(
            catalog.findings(),
            &[MapCatalogFinding::MissingParent {
                map_id: MapId::new(1).unwrap(),
                parent_id: MapId::new(2).unwrap(),
            }]
        );
    }

    #[test]
    fn does_not_invent_map_document_paths_above_the_evidenced_width() {
        let mut entries = vec!["null".to_string(); 1_001];
        entries[1_000] = r#"{"id":1000,"name":"Wide","order":1,"parentId":0}"#.to_string();
        let source = format!("[{}]", entries.join(","));
        let (_temp, snapshot) = snapshot(source.as_bytes(), &[]);

        let catalog = map_catalog(&snapshot).unwrap();

        assert_eq!(
            catalog.findings(),
            &[MapCatalogFinding::UnevidencedMapDocumentPath {
                map_id: MapId::new(1_000).unwrap(),
            }]
        );
    }
}
