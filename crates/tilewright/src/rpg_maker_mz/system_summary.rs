// SPDX-License-Identifier: MPL-2.0

//! Experimental, read-only summary of selected RPG Maker MZ system settings.

use crate::rpg_maker_mz::map_catalog::JsonValueKind;
use crate::rpg_maker_mz::snapshot::ProjectSnapshot;
use jsonc_parser::cst::{CstNode, CstObject};
use std::fmt;
use std::path::{Path, PathBuf};

const SYSTEM_DOCUMENT_PATH: &str = "data/System.json";

/// A read-only summary of selected project-level system settings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SystemSummary {
    document_path: PathBuf,
    game_title: String,
    currency_unit: String,
    locale: String,
    edit_map_id: u32,
    start_map_id: u32,
    start_x: u32,
    start_y: u32,
}

impl SystemSummary {
    /// Returns the exact project-relative system-document path.
    pub fn document_path(&self) -> &Path {
        &self.document_path
    }

    /// Returns the decoded game title without normalization.
    pub fn game_title(&self) -> &str {
        &self.game_title
    }

    /// Returns the decoded currency unit without normalization.
    pub fn currency_unit(&self) -> &str {
        &self.currency_unit
    }

    /// Returns the decoded locale string without normalization.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Returns the stored nonnegative editor-map ID scalar.
    ///
    /// This is not a catalog-scoped map identifier. In particular, zero and
    /// dangling values are reported without semantic interpretation.
    pub fn edit_map_id(&self) -> u32 {
        self.edit_map_id
    }

    /// Returns the stored nonnegative player-start map ID scalar.
    ///
    /// This is not a catalog-scoped map identifier. The summary does not
    /// validate that a corresponding map exists.
    pub fn start_map_id(&self) -> u32 {
        self.start_map_id
    }

    /// Returns the stored nonnegative player-start X-coordinate scalar.
    pub fn start_x(&self) -> u32 {
        self.start_x
    }

    /// Returns the stored nonnegative player-start Y-coordinate scalar.
    pub fn start_y(&self) -> u32 {
        self.start_y
    }
}

/// A required field in the experimental system summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SystemSummaryField {
    /// The game title.
    GameTitle,
    /// The in-game currency unit.
    CurrencyUnit,
    /// The locale string.
    Locale,
    /// The editor's stored current-map scalar.
    EditMapId,
    /// The player's stored starting-map scalar.
    StartMapId,
    /// The player's stored starting X-coordinate scalar.
    StartX,
    /// The player's stored starting Y-coordinate scalar.
    StartY,
}

impl SystemSummaryField {
    fn name(self) -> &'static str {
        match self {
            Self::GameTitle => "gameTitle",
            Self::CurrencyUnit => "currencyUnit",
            Self::Locale => "locale",
            Self::EditMapId => "editMapId",
            Self::StartMapId => "startMapId",
            Self::StartX => "startX",
            Self::StartY => "startY",
        }
    }
}

impl fmt::Display for SystemSummaryField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Why the system document could not be summarized coherently.
#[derive(Debug)]
#[non_exhaustive]
pub enum SystemSummaryError {
    /// No inventory entry or loaded document exists at the expected path.
    #[non_exhaustive]
    MissingDocument { path: PathBuf },
    /// Snapshot diagnostics exist instead of a loaded document at the path.
    #[non_exhaustive]
    UnavailableDocument { path: PathBuf },
    /// The system document root is not an object.
    #[non_exhaustive]
    UnexpectedRootKind {
        path: PathBuf,
        actual: JsonValueKind,
    },
    /// A required decoded field is absent.
    #[non_exhaustive]
    MissingField {
        path: PathBuf,
        field: SystemSummaryField,
    },
    /// A required decoded field name occurs more than once.
    #[non_exhaustive]
    DuplicateField {
        path: PathBuf,
        field: SystemSummaryField,
    },
    /// A required field has the wrong JSON value kind.
    #[non_exhaustive]
    UnexpectedFieldKind {
        path: PathBuf,
        field: SystemSummaryField,
        actual: JsonValueKind,
    },
    /// A required number is not a supported nonnegative unsigned integer.
    #[non_exhaustive]
    UnsupportedInteger {
        path: PathBuf,
        field: SystemSummaryField,
    },
    /// A required string could not be decoded.
    #[non_exhaustive]
    InvalidString {
        path: PathBuf,
        field: SystemSummaryField,
    },
}

impl fmt::Display for SystemSummaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDocument { path, .. } => {
                write!(formatter, "system document {} is missing", path.display())
            }
            Self::UnavailableDocument { path, .. } => write!(
                formatter,
                "system document {} was not loaded; inspect snapshot diagnostics",
                path.display()
            ),
            Self::UnexpectedRootKind { path, actual, .. } => write!(
                formatter,
                "system document {} root is {actual}, expected object",
                path.display()
            ),
            Self::MissingField { path, field, .. } => write!(
                formatter,
                "system document {} is missing required field {field}",
                path.display()
            ),
            Self::DuplicateField { path, field, .. } => write!(
                formatter,
                "system document {} has duplicate required field {field}",
                path.display()
            ),
            Self::UnexpectedFieldKind {
                path,
                field,
                actual,
                ..
            } => write!(
                formatter,
                "system document {} field {field} is {actual}, expected {}",
                path.display(),
                expected_kind(*field)
            ),
            Self::UnsupportedInteger { path, field, .. } => write!(
                formatter,
                "system document {} field {field} is not a supported nonnegative unsigned integer",
                path.display()
            ),
            Self::InvalidString { path, field, .. } => write!(
                formatter,
                "system document {} field {field} could not be decoded",
                path.display()
            ),
        }
    }
}

impl std::error::Error for SystemSummaryError {}

/// Projects selected `data/System.json` fields into a bounded read-only summary.
///
/// This function reads only lossless documents and diagnostics already present
/// in `snapshot`. Strings are decoded without normalization, while numeric map
/// and coordinate values remain nonnegative `u32` scalars. Unknown fields and
/// exact source bytes remain untouched in the raw snapshot.
///
/// The result does not require a map catalog, validate map references or
/// coordinate bounds, interpret party members or `versionId`, compare titles
/// across files, perform filesystem I/O, mutate raw data, establish editor
/// compatibility, or persist changes.
///
/// # Errors
///
/// Returns [`SystemSummaryError`] when `data/System.json` is absent or
/// unavailable, its root is not an object, or required summary structure is
/// ambiguous or outside the bounded scalar contract.
pub fn system_summary(snapshot: &ProjectSnapshot) -> Result<SystemSummary, SystemSummaryError> {
    let path = PathBuf::from(SYSTEM_DOCUMENT_PATH);
    let Some(document) = snapshot.documents().get(path.as_path()) else {
        return if snapshot.diagnostics().contains_key(path.as_path()) {
            Err(SystemSummaryError::UnavailableDocument { path })
        } else {
            Err(SystemSummaryError::MissingDocument { path })
        };
    };

    let root =
        document
            .cst_root()
            .value()
            .ok_or_else(|| SystemSummaryError::UnexpectedRootKind {
                path: path.clone(),
                actual: JsonValueKind::Unrecognized,
            })?;
    let object = root
        .as_object()
        .ok_or_else(|| SystemSummaryError::UnexpectedRootKind {
            path: path.clone(),
            actual: json_kind(&root),
        })?;

    let game_title = required_string(&object, &path, SystemSummaryField::GameTitle)?;
    let currency_unit = required_string(&object, &path, SystemSummaryField::CurrencyUnit)?;
    let locale = required_string(&object, &path, SystemSummaryField::Locale)?;
    let edit_map_id = required_integer(&object, &path, SystemSummaryField::EditMapId)?;
    let start_map_id = required_integer(&object, &path, SystemSummaryField::StartMapId)?;
    let start_x = required_integer(&object, &path, SystemSummaryField::StartX)?;
    let start_y = required_integer(&object, &path, SystemSummaryField::StartY)?;

    Ok(SystemSummary {
        document_path: path,
        game_title,
        currency_unit,
        locale,
        edit_map_id,
        start_map_id,
        start_x,
        start_y,
    })
}

fn required_node(
    object: &CstObject,
    path: &Path,
    field: SystemSummaryField,
) -> Result<CstNode, SystemSummaryError> {
    let mut matches = object.properties().into_iter().filter(|property| {
        property
            .name()
            .and_then(|name| name.decoded_value().ok())
            .is_some_and(|name| name == field.name())
    });
    let property = matches
        .next()
        .ok_or_else(|| SystemSummaryError::MissingField {
            path: path.to_owned(),
            field,
        })?;
    if matches.next().is_some() {
        return Err(SystemSummaryError::DuplicateField {
            path: path.to_owned(),
            field,
        });
    }
    property
        .value()
        .ok_or_else(|| SystemSummaryError::MissingField {
            path: path.to_owned(),
            field,
        })
}

fn required_integer(
    object: &CstObject,
    path: &Path,
    field: SystemSummaryField,
) -> Result<u32, SystemSummaryError> {
    let node = required_node(object, path, field)?;
    let number = node
        .as_number_lit()
        .ok_or_else(|| SystemSummaryError::UnexpectedFieldKind {
            path: path.to_owned(),
            field,
            actual: json_kind(&node),
        })?;
    number
        .to_string()
        .parse::<u32>()
        .map_err(|_| SystemSummaryError::UnsupportedInteger {
            path: path.to_owned(),
            field,
        })
}

fn required_string(
    object: &CstObject,
    path: &Path,
    field: SystemSummaryField,
) -> Result<String, SystemSummaryError> {
    let node = required_node(object, path, field)?;
    let string = node
        .as_string_lit()
        .ok_or_else(|| SystemSummaryError::UnexpectedFieldKind {
            path: path.to_owned(),
            field,
            actual: json_kind(&node),
        })?;
    string
        .decoded_value()
        .map_err(|_| SystemSummaryError::InvalidString {
            path: path.to_owned(),
            field,
        })
}

fn expected_kind(field: SystemSummaryField) -> JsonValueKind {
    match field {
        SystemSummaryField::GameTitle
        | SystemSummaryField::CurrencyUnit
        | SystemSummaryField::Locale => JsonValueKind::String,
        SystemSummaryField::EditMapId
        | SystemSummaryField::StartMapId
        | SystemSummaryField::StartX
        | SystemSummaryField::StartY => JsonValueKind::Number,
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

    fn load_test_snapshot(source: Option<&[u8]>) -> (TempDir, ProjectSnapshot) {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        if let Some(source) = source {
            fs::write(temp.path().join(SYSTEM_DOCUMENT_PATH), source).unwrap();
        }
        let root = Dir::open_ambient_dir(temp.path(), cap_std::ambient_authority()).unwrap();
        let loaded = load_snapshot(
            &root,
            SnapshotLimits {
                max_documents: NonZeroUsize::new(4).unwrap(),
                max_bytes_per_document: NonZeroUsize::new(65_536).unwrap(),
                max_aggregate_bytes: NonZeroUsize::new(65_536).unwrap(),
            },
        )
        .unwrap();
        (temp, loaded)
    }

    fn complete_source(overrides: &[(&str, &str)]) -> Vec<u8> {
        let fields = [
            ("gameTitle", r#""Game""#),
            ("currencyUnit", r#""G""#),
            ("locale", r#""en_US""#),
            ("editMapId", "1"),
            ("startMapId", "2"),
            ("startX", "3"),
            ("startY", "4"),
        ];
        let entries = fields.map(|(name, default)| {
            let value = overrides
                .iter()
                .find_map(|(candidate, value)| (*candidate == name).then_some(*value))
                .unwrap_or(default);
            format!(r#""{name}":{value}"#)
        });
        format!("{{{}}}", entries.join(",")).into_bytes()
    }

    #[test]
    fn summarizes_only_bounded_fields_without_changing_raw_bytes() {
        let source = br#"{"gameTitle":"Snowman \u2603","currencyUnit":"Coins","locale":"en_US","editMapId":7,"startMapId":9,"startX":11,"startY":13,"partyMembers":[1,2],"versionId":999,"unknown":{"nested":true}}"#;
        let (_temp, snapshot) = load_test_snapshot(Some(source));

        let summary = system_summary(&snapshot).unwrap();

        assert_eq!(summary.document_path(), Path::new(SYSTEM_DOCUMENT_PATH));
        assert_eq!(summary.game_title(), "Snowman ☃");
        assert_eq!(summary.currency_unit(), "Coins");
        assert_eq!(summary.locale(), "en_US");
        assert_eq!(summary.edit_map_id(), 7);
        assert_eq!(summary.start_map_id(), 9);
        assert_eq!(summary.start_x(), 11);
        assert_eq!(summary.start_y(), 13);
        assert_eq!(
            snapshot.documents()[Path::new(SYSTEM_DOCUMENT_PATH)].source_bytes(),
            source
        );
    }

    #[test]
    fn accepts_empty_escaped_unicode_and_unsigned_boundaries() {
        let source = complete_source(&[
            ("gameTitle", "\"\""),
            ("currencyUnit", r#""line\nunit""#),
            ("locale", r#""日本語""#),
            ("editMapId", "0"),
            ("startMapId", &u32::MAX.to_string()),
            ("startX", "0"),
            ("startY", &u32::MAX.to_string()),
        ]);
        let (_temp, snapshot) = load_test_snapshot(Some(&source));

        let summary = system_summary(&snapshot).unwrap();

        assert_eq!(summary.game_title(), "");
        assert_eq!(summary.currency_unit(), "line\nunit");
        assert_eq!(summary.locale(), "日本語");
        assert_eq!(summary.edit_map_id(), 0);
        assert_eq!(summary.start_map_id(), u32::MAX);
        assert_eq!(summary.start_x(), 0);
        assert_eq!(summary.start_y(), u32::MAX);
    }

    #[test]
    fn distinguishes_missing_unavailable_and_non_file_documents() {
        let (_temp, missing) = load_test_snapshot(None);
        assert!(matches!(
            system_summary(&missing),
            Err(SystemSummaryError::MissingDocument { .. })
        ));

        let (_temp, unavailable) = load_test_snapshot(Some(b"not json"));
        assert!(matches!(
            system_summary(&unavailable),
            Err(SystemSummaryError::UnavailableDocument { .. })
        ));

        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(SYSTEM_DOCUMENT_PATH)).unwrap();
        let root = Dir::open_ambient_dir(temp.path(), cap_std::ambient_authority()).unwrap();
        let loaded = load_snapshot(
            &root,
            SnapshotLimits {
                max_documents: NonZeroUsize::new(4).unwrap(),
                max_bytes_per_document: NonZeroUsize::new(1024).unwrap(),
                max_aggregate_bytes: NonZeroUsize::new(4096).unwrap(),
            },
        )
        .unwrap();
        assert!(matches!(
            system_summary(&loaded),
            Err(SystemSummaryError::UnavailableDocument { .. })
        ));
    }

    #[test]
    fn refuses_non_object_root() {
        let (_temp, snapshot) = load_test_snapshot(Some(br#"[]"#));
        assert!(matches!(
            system_summary(&snapshot),
            Err(SystemSummaryError::UnexpectedRootKind {
                actual: JsonValueKind::Array,
                ..
            })
        ));
    }

    #[test]
    fn refuses_missing_duplicate_and_wrong_kind_fields() {
        let missing = br#"{"currencyUnit":"G","locale":"en_US","editMapId":1,"startMapId":1,"startX":1,"startY":1}"#;
        let (_temp, snapshot) = load_test_snapshot(Some(missing));
        assert!(matches!(
            system_summary(&snapshot),
            Err(SystemSummaryError::MissingField {
                field: SystemSummaryField::GameTitle,
                ..
            })
        ));

        let duplicate = br#"{"gameTitle":"A","game\u0054itle":"B","currencyUnit":"G","locale":"en_US","editMapId":1,"startMapId":1,"startX":1,"startY":1}"#;
        let (_temp, snapshot) = load_test_snapshot(Some(duplicate));
        assert!(matches!(
            system_summary(&snapshot),
            Err(SystemSummaryError::DuplicateField {
                field: SystemSummaryField::GameTitle,
                ..
            })
        ));

        let wrong_string = complete_source(&[("currencyUnit", "false")]);
        let (_temp, snapshot) = load_test_snapshot(Some(&wrong_string));
        assert!(matches!(
            system_summary(&snapshot),
            Err(SystemSummaryError::UnexpectedFieldKind {
                field: SystemSummaryField::CurrencyUnit,
                actual: JsonValueKind::Boolean,
                ..
            })
        ));

        let wrong_number = complete_source(&[("startX", r#""1""#)]);
        let (_temp, snapshot) = load_test_snapshot(Some(&wrong_number));
        assert!(matches!(
            system_summary(&snapshot),
            Err(SystemSummaryError::UnexpectedFieldKind {
                field: SystemSummaryField::StartX,
                actual: JsonValueKind::String,
                ..
            })
        ));
    }

    #[test]
    fn refuses_unsupported_integer_forms_for_each_numeric_role() {
        let cases = [
            ("editMapId", "-1", SystemSummaryField::EditMapId),
            ("startMapId", "1.0", SystemSummaryField::StartMapId),
            ("startX", "1e0", SystemSummaryField::StartX),
            ("startY", "4294967296", SystemSummaryField::StartY),
        ];

        for (name, value, expected_field) in cases {
            let source = complete_source(&[(name, value)]);
            let (_temp, snapshot) = load_test_snapshot(Some(&source));
            assert!(matches!(
                system_summary(&snapshot),
                Err(SystemSummaryError::UnsupportedInteger { field, .. })
                    if field == expected_field
            ));
        }
    }
}
