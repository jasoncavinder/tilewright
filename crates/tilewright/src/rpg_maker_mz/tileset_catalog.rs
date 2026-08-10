// SPDX-License-Identifier: MPL-2.0

//! Experimental, read-only typed projection of RPG Maker MZ tileset identity.

use crate::rpg_maker_mz::map_catalog::JsonValueKind;
use crate::rpg_maker_mz::snapshot::ProjectSnapshot;
use jsonc_parser::cst::{CstNode, CstObject};
use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU32;

const TILESETS_PATH: &str = "data/Tilesets.json";

/// A positive identifier scoped to the experimental tileset catalog.
///
/// This is not yet a stable project-wide resource identity contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TilesetId(NonZeroU32);

impl TilesetId {
    /// Creates a tileset identifier, returning `None` for the reserved zero value.
    pub fn new(value: u32) -> Option<Self> {
        NonZeroU32::new(value).map(Self)
    }

    /// Returns the positive numeric identifier.
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Display for TilesetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

/// One structurally understood tileset record.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TilesetRecord {
    id: TilesetId,
    name: String,
}

impl TilesetRecord {
    /// Returns the record's positive tileset identifier.
    pub fn id(&self) -> TilesetId {
        self.id
    }

    /// Returns the decoded editor-facing tileset name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A structurally coherent, read-only tileset catalog projection.
#[derive(Debug)]
#[non_exhaustive]
pub struct TilesetCatalog {
    records: BTreeMap<TilesetId, TilesetRecord>,
}

impl TilesetCatalog {
    /// Returns records in ascending tileset-ID order.
    pub fn records(&self) -> &BTreeMap<TilesetId, TilesetRecord> {
        &self.records
    }

    /// Returns one record by tileset ID.
    pub fn get(&self, id: TilesetId) -> Option<&TilesetRecord> {
        self.records.get(&id)
    }
}

/// A required field in the experimental tileset catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TilesetField {
    /// The positive tileset identifier.
    Id,
    /// The decoded editor-facing name.
    Name,
}

impl TilesetField {
    fn name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
        }
    }
}

impl fmt::Display for TilesetField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Why a raw snapshot could not produce a coherent tileset catalog.
#[derive(Debug)]
#[non_exhaustive]
pub enum TilesetCatalogError {
    /// The snapshot inventory does not contain `data/Tilesets.json`.
    MissingDocument,
    /// Snapshot diagnostics exist instead of a loaded tileset document.
    UnavailableDocument,
    /// The tileset document root is not an array.
    #[non_exhaustive]
    UnexpectedRootKind { actual: JsonValueKind },
    /// A non-null array entry is not an object.
    #[non_exhaustive]
    UnexpectedEntryKind { index: usize, actual: JsonValueKind },
    /// A required field is absent.
    #[non_exhaustive]
    MissingField { index: usize, field: TilesetField },
    /// A required decoded field name occurs more than once.
    #[non_exhaustive]
    DuplicateField { index: usize, field: TilesetField },
    /// A required field has the wrong JSON value kind.
    #[non_exhaustive]
    UnexpectedFieldKind {
        index: usize,
        field: TilesetField,
        actual: JsonValueKind,
    },
    /// A required number is not a supported positive unsigned integer.
    #[non_exhaustive]
    UnsupportedInteger { index: usize, field: TilesetField },
    /// A string field could not be decoded.
    #[non_exhaustive]
    InvalidString { index: usize, field: TilesetField },
    /// An array index cannot be represented by the identifier type.
    #[non_exhaustive]
    IndexOutOfRange { index: usize },
    /// A tileset object occupies reserved array index zero.
    ReservedIndex,
    /// A decoded positive ID does not equal its array index.
    #[non_exhaustive]
    IdIndexMismatch { index: usize, decoded_id: TilesetId },
}

impl fmt::Display for TilesetCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDocument => formatter.write_str("data/Tilesets.json is missing"),
            Self::UnavailableDocument => formatter
                .write_str("data/Tilesets.json was not loaded; inspect snapshot diagnostics"),
            Self::UnexpectedRootKind { actual, .. } => write!(
                formatter,
                "data/Tilesets.json root is {actual}, expected array"
            ),
            Self::UnexpectedEntryKind { index, actual, .. } => write!(
                formatter,
                "data/Tilesets.json entry {index} is {actual}, expected object or null"
            ),
            Self::MissingField { index, field, .. } => write!(
                formatter,
                "data/Tilesets.json entry {index} is missing required field {field}"
            ),
            Self::DuplicateField { index, field, .. } => write!(
                formatter,
                "data/Tilesets.json entry {index} has duplicate required field {field}"
            ),
            Self::UnexpectedFieldKind {
                index,
                field,
                actual,
                ..
            } => write!(
                formatter,
                "data/Tilesets.json entry {index} field {field} is {actual}, expected {}",
                expected_kind(*field)
            ),
            Self::UnsupportedInteger { index, field, .. } => write!(
                formatter,
                "data/Tilesets.json entry {index} field {field} is not a supported positive unsigned integer"
            ),
            Self::InvalidString { index, field, .. } => write!(
                formatter,
                "data/Tilesets.json entry {index} field {field} could not be decoded"
            ),
            Self::IndexOutOfRange { index, .. } => write!(
                formatter,
                "data/Tilesets.json index {index} exceeds the supported identifier range"
            ),
            Self::ReservedIndex => {
                formatter.write_str("data/Tilesets.json has a tileset object at reserved index 0")
            }
            Self::IdIndexMismatch {
                index, decoded_id, ..
            } => write!(
                formatter,
                "data/Tilesets.json entry {index} has decoded ID {decoded_id}"
            ),
        }
    }
}

impl std::error::Error for TilesetCatalogError {}

/// Projects exact `data/Tilesets.json` into bounded tileset IDs and names.
///
/// This operation reads only the lossless document already present in
/// `snapshot`. Null array entries are accepted as holes. Every non-null record
/// must have a positive ID equal to its array index and a decodable name.
///
/// Modes, image-name slots, tile flags, notes, and unknown fields remain
/// untouched in the raw snapshot. This operation performs no filesystem I/O,
/// map-reference or asset validation, mutation, serialization, persistence,
/// runtime execution, or compatibility verification.
///
/// # Errors
///
/// Returns [`TilesetCatalogError`] when the document is absent or unavailable,
/// or when required root, entry, ID, or name structure is ambiguous.
pub fn tileset_catalog(snapshot: &ProjectSnapshot) -> Result<TilesetCatalog, TilesetCatalogError> {
    let path = std::path::Path::new(TILESETS_PATH);
    let Some(document) = snapshot.documents().get(path) else {
        return if snapshot.diagnostics().contains_key(path) {
            Err(TilesetCatalogError::UnavailableDocument)
        } else {
            Err(TilesetCatalogError::MissingDocument)
        };
    };

    let root = document
        .cst_root()
        .value()
        .ok_or(TilesetCatalogError::UnexpectedRootKind {
            actual: JsonValueKind::Unrecognized,
        })?;
    let array = root
        .as_array()
        .ok_or_else(|| TilesetCatalogError::UnexpectedRootKind {
            actual: json_kind(&root),
        })?;

    let mut records = BTreeMap::new();
    for (index, node) in array.elements().into_iter().enumerate() {
        if node.as_null_keyword().is_some() {
            continue;
        }
        let object = node
            .as_object()
            .ok_or_else(|| TilesetCatalogError::UnexpectedEntryKind {
                index,
                actual: json_kind(&node),
            })?;
        let index_id = tileset_id_from_index(index)?;
        let decoded_id = required_positive_integer(&object, index, TilesetField::Id)?;
        if decoded_id != index_id {
            return Err(TilesetCatalogError::IdIndexMismatch { index, decoded_id });
        }
        let name = required_string(&object, index, TilesetField::Name)?;
        records.insert(index_id, TilesetRecord { id: index_id, name });
    }

    Ok(TilesetCatalog { records })
}

fn tileset_id_from_index(index: usize) -> Result<TilesetId, TilesetCatalogError> {
    let value = u32::try_from(index).map_err(|_| TilesetCatalogError::IndexOutOfRange { index })?;
    TilesetId::new(value).ok_or(TilesetCatalogError::ReservedIndex)
}

fn required_node(
    object: &CstObject,
    index: usize,
    field: TilesetField,
) -> Result<CstNode, TilesetCatalogError> {
    let mut matches = object.properties().into_iter().filter(|property| {
        property
            .name()
            .and_then(|name| name.decoded_value().ok())
            .is_some_and(|name| name == field.name())
    });
    let property = matches
        .next()
        .ok_or(TilesetCatalogError::MissingField { index, field })?;
    if matches.next().is_some() {
        return Err(TilesetCatalogError::DuplicateField { index, field });
    }
    property
        .value()
        .ok_or(TilesetCatalogError::MissingField { index, field })
}

fn required_positive_integer(
    object: &CstObject,
    index: usize,
    field: TilesetField,
) -> Result<TilesetId, TilesetCatalogError> {
    let node = required_node(object, index, field)?;
    let number = node
        .as_number_lit()
        .ok_or_else(|| TilesetCatalogError::UnexpectedFieldKind {
            index,
            field,
            actual: json_kind(&node),
        })?;
    let value = number
        .to_string()
        .parse::<u32>()
        .ok()
        .and_then(TilesetId::new)
        .ok_or(TilesetCatalogError::UnsupportedInteger { index, field })?;
    Ok(value)
}

fn required_string(
    object: &CstObject,
    index: usize,
    field: TilesetField,
) -> Result<String, TilesetCatalogError> {
    let node = required_node(object, index, field)?;
    let string = node
        .as_string_lit()
        .ok_or_else(|| TilesetCatalogError::UnexpectedFieldKind {
            index,
            field,
            actual: json_kind(&node),
        })?;
    string
        .decoded_value()
        .map_err(|_| TilesetCatalogError::InvalidString { index, field })
}

fn expected_kind(field: TilesetField) -> JsonValueKind {
    match field {
        TilesetField::Id => JsonValueKind::Number,
        TilesetField::Name => JsonValueKind::String,
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
    use std::path::Path;
    use tempfile::TempDir;

    type ErrorPredicate = fn(&TilesetCatalogError) -> bool;

    fn snapshot(source: Option<&[u8]>, max_bytes: usize) -> (TempDir, ProjectSnapshot) {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        if let Some(source) = source {
            fs::write(temp.path().join(TILESETS_PATH), source).unwrap();
        }
        let root = Dir::open_ambient_dir(temp.path(), cap_std::ambient_authority()).unwrap();
        let loaded = load_snapshot(
            &root,
            SnapshotLimits {
                max_documents: NonZeroUsize::new(8).unwrap(),
                max_bytes_per_document: NonZeroUsize::new(max_bytes).unwrap(),
                max_aggregate_bytes: NonZeroUsize::new(131_072).unwrap(),
            },
        )
        .unwrap();
        (temp, loaded)
    }

    #[test]
    fn projects_holes_and_decoded_names_without_changing_raw_bytes() {
        let source = br#"[null,{"id":1,"name":"Field \u2603","mode":1,"flags":[1],"tilesetNames":["secret"],"note":"memo","plugin":{"value":1}},null,{"id":3,"name":"Area"}]"#;
        let (_temp, snapshot) = snapshot(Some(source), 65_536);

        let catalog = tileset_catalog(&snapshot).unwrap();

        let ids: Vec<_> = catalog.records().keys().map(|id| id.get()).collect();
        assert_eq!(ids, [1, 3]);
        assert_eq!(
            catalog.get(TilesetId::new(1).unwrap()).unwrap().name(),
            "Field ☃"
        );
        assert_eq!(
            snapshot.documents()[Path::new(TILESETS_PATH)].source_bytes(),
            source
        );
    }

    #[test]
    fn distinguishes_missing_and_unavailable_documents() {
        let (_temp, missing) = snapshot(None, 65_536);
        assert!(matches!(
            tileset_catalog(&missing),
            Err(TilesetCatalogError::MissingDocument)
        ));

        let source = format!(r#"[null,{{"id":1,"name":"{}"}}]"#, "x".repeat(256));
        let (_temp, unavailable) = snapshot(Some(source.as_bytes()), 64);
        assert!(matches!(
            tileset_catalog(&unavailable),
            Err(TilesetCatalogError::UnavailableDocument)
        ));
    }

    #[test]
    fn refuses_ambiguous_or_malformed_required_structure() {
        let cases: &[(&[u8], ErrorPredicate)] = &[
            (br#"{}"#, |error| {
                matches!(error, TilesetCatalogError::UnexpectedRootKind { .. })
            }),
            (br#"[null,true]"#, |error| {
                matches!(
                    error,
                    TilesetCatalogError::UnexpectedEntryKind {
                        index: 1,
                        actual: JsonValueKind::Boolean,
                        ..
                    }
                )
            }),
            (br#"[null,{"name":"A"}]"#, |error| {
                matches!(
                    error,
                    TilesetCatalogError::MissingField {
                        index: 1,
                        field: TilesetField::Id,
                        ..
                    }
                )
            }),
            (br#"[null,{"id":1,"name":"A","n\u0061me":"B"}]"#, |error| {
                matches!(
                    error,
                    TilesetCatalogError::DuplicateField {
                        index: 1,
                        field: TilesetField::Name,
                        ..
                    }
                )
            }),
            (br#"[null,{"id":1,"name":false}]"#, |error| {
                matches!(
                    error,
                    TilesetCatalogError::UnexpectedFieldKind {
                        index: 1,
                        field: TilesetField::Name,
                        actual: JsonValueKind::Boolean,
                        ..
                    }
                )
            }),
        ];

        for (source, predicate) in cases {
            let (_temp, snapshot) = snapshot(Some(source), 65_536);
            let error = tileset_catalog(&snapshot).unwrap_err();
            assert!(predicate(&error), "unexpected error: {error}");
        }
    }

    #[test]
    fn refuses_unsupported_integer_forms_and_id_mismatch() {
        for source in [
            br#"[null,{"id":0,"name":"Zero"}]"#.as_slice(),
            br#"[null,{"id":1.0,"name":"Decimal"}]"#.as_slice(),
            br#"[null,{"id":4294967296,"name":"Large"}]"#.as_slice(),
        ] {
            let (_temp, snapshot) = snapshot(Some(source), 65_536);
            assert!(matches!(
                tileset_catalog(&snapshot),
                Err(TilesetCatalogError::UnsupportedInteger { .. })
            ));
        }

        let (_temp, reserved) = snapshot(Some(br#"[{"id":1,"name":"Reserved"}]"#), 65_536);
        assert!(matches!(
            tileset_catalog(&reserved),
            Err(TilesetCatalogError::ReservedIndex)
        ));

        let (_temp, mismatch) = snapshot(Some(br#"[null,{"id":2,"name":"Wrong"}]"#), 65_536);
        assert!(matches!(
            tileset_catalog(&mismatch),
            Err(TilesetCatalogError::IdIndexMismatch {
                index: 1,
                decoded_id,
                ..
            }) if decoded_id == TilesetId::new(2).unwrap()
        ));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn checks_index_range_without_allocating_a_huge_array() {
        assert!(matches!(
            tileset_id_from_index(usize::MAX),
            Err(TilesetCatalogError::IndexOutOfRange { .. })
        ));
    }
}
