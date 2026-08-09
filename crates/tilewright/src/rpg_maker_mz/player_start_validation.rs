// SPDX-License-Identifier: MPL-2.0

//! Experimental contextual validation of the RPG Maker MZ player start.

use crate::rpg_maker_mz::map_catalog::{MapCatalogError, MapId, map_catalog};
use crate::rpg_maker_mz::map_summary::{MapSummaryError, map_summary};
use crate::rpg_maker_mz::snapshot::ProjectSnapshot;
use crate::rpg_maker_mz::system_summary::{SystemSummaryError, system_summary};
use std::fmt;

/// A read-only report for the stored player starting position.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PlayerStartValidation {
    start_map_id: u32,
    start_x: u32,
    start_y: u32,
    findings: Vec<PlayerStartFinding>,
}

impl PlayerStartValidation {
    /// Returns the stored nonnegative starting-map scalar.
    pub fn start_map_id(&self) -> u32 {
        self.start_map_id
    }

    /// Returns the stored nonnegative starting X-coordinate scalar.
    pub fn start_x(&self) -> u32 {
        self.start_x
    }

    /// Returns the stored nonnegative starting Y-coordinate scalar.
    pub fn start_y(&self) -> u32 {
        self.start_y
    }

    /// Returns deterministic contextual findings.
    ///
    /// Findings describe relationships established by Tilewright's bounded
    /// evidence. Except where official documentation is cited, they do not
    /// claim that RPG Maker MZ accepts or rejects the project.
    pub fn findings(&self) -> &[PlayerStartFinding] {
        &self.findings
    }

    /// Returns `true` when the bounded validation produced no findings.
    pub fn is_finding_free(&self) -> bool {
        self.findings.is_empty()
    }
}

/// A contextual finding about the stored player starting position.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlayerStartFinding {
    /// The evidenced unset triplet `0, 0, 0` is stored.
    ///
    /// RPG Maker MZ 1.10.0 was observed to present and preserve this triplet as
    /// `None`; official help states that a game cannot start without a player
    /// starting position.
    MissingPlayerStart,
    /// The map ID is zero while one or both stored coordinates are nonzero.
    ///
    /// This state is reported without interpreting it as set or unset because
    /// it has not been observed in the editor.
    #[non_exhaustive]
    ZeroMapIdWithCoordinates { start_x: u32, start_y: u32 },
    /// The positive starting-map ID has no record in the coherent map catalog.
    #[non_exhaustive]
    MissingMapRecord { map_id: MapId },
    /// The stored coordinates fall outside the selected map's dimensions.
    #[non_exhaustive]
    OutOfBounds {
        map_id: MapId,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
    },
}

impl fmt::Display for PlayerStartFinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPlayerStart => formatter.write_str(
                "player starting position is unset; RPG Maker MZ cannot start the game without it",
            ),
            Self::ZeroMapIdWithCoordinates {
                start_x, start_y, ..
            } => write!(
                formatter,
                "player start has map ID 0 with unevidenced coordinates ({start_x}, {start_y})"
            ),
            Self::MissingMapRecord { map_id, .. } => write!(
                formatter,
                "player start refers to map {map_id}, which has no map-catalog record"
            ),
            Self::OutOfBounds {
                map_id,
                start_x,
                start_y,
                width,
                height,
                ..
            } => write!(
                formatter,
                "player start ({start_x}, {start_y}) is outside map {map_id} dimensions {width} x {height}"
            ),
        }
    }
}

/// Why the player starting position could not be validated coherently.
#[derive(Debug)]
#[non_exhaustive]
pub enum PlayerStartValidationError {
    /// The selected system settings could not be projected.
    #[non_exhaustive]
    System { source: SystemSummaryError },
    /// The map catalog required by a positive map ID could not be projected.
    #[non_exhaustive]
    Catalog { source: MapCatalogError },
    /// The catalog-selected starting map could not be summarized.
    #[non_exhaustive]
    Map { source: MapSummaryError },
}

impl fmt::Display for PlayerStartValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System { source, .. } => {
                write!(formatter, "system summary is unavailable: {source}")
            }
            Self::Catalog { source, .. } => {
                write!(formatter, "map catalog is unavailable: {source}")
            }
            Self::Map { source, .. } => {
                write!(formatter, "starting-map summary is unavailable: {source}")
            }
        }
    }
}

impl std::error::Error for PlayerStartValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::System { source, .. } => Some(source),
            Self::Catalog { source, .. } => Some(source),
            Self::Map { source, .. } => Some(source),
        }
    }
}

/// Validates the stored player start against the loaded map catalog and map.
///
/// This pure operation composes [`system_summary`], [`map_catalog`], and
/// [`map_summary`] over `snapshot`. It recognizes the directly observed
/// `startMapId = 0`, `startX = 0`, `startY = 0` unset triplet, reports an
/// absent positive map-catalog reference, and checks zero-based coordinates
/// against the selected map's positive width and height.
///
/// It does not perform filesystem I/O, mutate the snapshot, establish that MZ
/// accepts every finding-free state, validate passability or event placement,
/// inspect vehicle starts, assign severities, or persist changes. A zero map ID
/// with nonzero coordinates is reported as unevidenced rather than interpreted.
///
/// # Errors
///
/// Returns [`PlayerStartValidationError`] when a structural summary required by
/// the applicable validation path cannot be produced. The map catalog and map
/// document are not required for the evidenced unset triplet or a zero map ID
/// with nonzero coordinates.
pub fn validate_player_start(
    snapshot: &ProjectSnapshot,
) -> Result<PlayerStartValidation, PlayerStartValidationError> {
    let system =
        system_summary(snapshot).map_err(|source| PlayerStartValidationError::System { source })?;
    let start_map_id = system.start_map_id();
    let start_x = system.start_x();
    let start_y = system.start_y();
    let mut findings = Vec::new();

    let Some(map_id) = MapId::new(start_map_id) else {
        if start_x == 0 && start_y == 0 {
            findings.push(PlayerStartFinding::MissingPlayerStart);
        } else {
            findings.push(PlayerStartFinding::ZeroMapIdWithCoordinates { start_x, start_y });
        }
        return Ok(PlayerStartValidation {
            start_map_id,
            start_x,
            start_y,
            findings,
        });
    };

    let catalog =
        map_catalog(snapshot).map_err(|source| PlayerStartValidationError::Catalog { source })?;
    if catalog.get(map_id).is_none() {
        findings.push(PlayerStartFinding::MissingMapRecord { map_id });
        return Ok(PlayerStartValidation {
            start_map_id,
            start_x,
            start_y,
            findings,
        });
    }

    let map = map_summary(snapshot, map_id)
        .map_err(|source| PlayerStartValidationError::Map { source })?;
    if start_x >= map.width() || start_y >= map.height() {
        findings.push(PlayerStartFinding::OutOfBounds {
            map_id,
            start_x,
            start_y,
            width: map.width(),
            height: map.height(),
        });
    }

    Ok(PlayerStartValidation {
        start_map_id,
        start_x,
        start_y,
        findings,
    })
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

    fn snapshot(
        system: &[u8],
        map_infos: Option<&[u8]>,
        map: Option<&[u8]>,
    ) -> (TempDir, ProjectSnapshot) {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/System.json"), system).unwrap();
        if let Some(map_infos) = map_infos {
            fs::write(temp.path().join("data/MapInfos.json"), map_infos).unwrap();
        }
        if let Some(map) = map {
            fs::write(temp.path().join("data/Map001.json"), map).unwrap();
        }
        let root = Dir::open_ambient_dir(temp.path(), cap_std::ambient_authority()).unwrap();
        let snapshot = load_snapshot(
            &root,
            SnapshotLimits {
                max_documents: NonZeroUsize::new(8).unwrap(),
                max_bytes_per_document: NonZeroUsize::new(65_536).unwrap(),
                max_aggregate_bytes: NonZeroUsize::new(131_072).unwrap(),
            },
        )
        .unwrap();
        (temp, snapshot)
    }

    fn system(start_map_id: u32, start_x: u32, start_y: u32) -> Vec<u8> {
        format!(
            r#"{{"gameTitle":"Game","currencyUnit":"G","locale":"en_US","editMapId":1,"startMapId":{start_map_id},"startX":{start_x},"startY":{start_y},"unknown":true}}"#
        )
        .into_bytes()
    }

    const MAP_INFOS: &[u8] = br#"[null,{"id":1,"name":"One","order":1,"parentId":0}]"#;
    const MAP: &[u8] =
        br#"{"displayName":"One","width":10,"height":8,"tilesetId":1,"events":[],"unknown":true}"#;

    #[test]
    fn accepts_in_bounds_coordinates_without_changing_raw_documents() {
        let system = system(1, 9, 7);
        let (_temp, snapshot) = snapshot(&system, Some(MAP_INFOS), Some(MAP));

        let report = validate_player_start(&snapshot).unwrap();

        assert!(report.is_finding_free());
        assert_eq!(report.start_map_id(), 1);
        assert_eq!(report.start_x(), 9);
        assert_eq!(report.start_y(), 7);
        assert_eq!(
            snapshot.documents()[Path::new("data/System.json")].source_bytes(),
            system
        );
        assert_eq!(
            snapshot.documents()[Path::new("data/Map001.json")].source_bytes(),
            MAP
        );
    }

    #[test]
    fn reports_evidenced_unset_triplet_without_requiring_map_data() {
        let system = system(0, 0, 0);
        let (_temp, snapshot) = snapshot(&system, None, None);

        let report = validate_player_start(&snapshot).unwrap();

        assert_eq!(report.findings(), &[PlayerStartFinding::MissingPlayerStart]);
    }

    #[test]
    fn reports_zero_map_with_nonzero_coordinates_without_interpreting_it() {
        let system = system(0, 2, 3);
        let (_temp, snapshot) = snapshot(&system, None, None);

        let report = validate_player_start(&snapshot).unwrap();

        assert_eq!(
            report.findings(),
            &[PlayerStartFinding::ZeroMapIdWithCoordinates {
                start_x: 2,
                start_y: 3,
            }]
        );
    }

    #[test]
    fn reports_missing_positive_map_record() {
        let system = system(2, 0, 0);
        let (_temp, snapshot) = snapshot(&system, Some(MAP_INFOS), Some(MAP));

        let report = validate_player_start(&snapshot).unwrap();

        assert_eq!(
            report.findings(),
            &[PlayerStartFinding::MissingMapRecord {
                map_id: MapId::new(2).unwrap(),
            }]
        );
    }

    #[test]
    fn reports_each_coordinate_boundary_as_out_of_bounds() {
        for (start_x, start_y) in [(10, 0), (0, 8), (u32::MAX, u32::MAX)] {
            let system = system(1, start_x, start_y);
            let (_temp, snapshot) = snapshot(&system, Some(MAP_INFOS), Some(MAP));

            let report = validate_player_start(&snapshot).unwrap();

            assert_eq!(
                report.findings(),
                &[PlayerStartFinding::OutOfBounds {
                    map_id: MapId::new(1).unwrap(),
                    start_x,
                    start_y,
                    width: 10,
                    height: 8,
                }]
            );
        }
    }

    #[test]
    fn distinguishes_system_catalog_and_selected_map_failures() {
        let (_temp, malformed_system) = snapshot(b"{}", Some(MAP_INFOS), Some(MAP));
        assert!(matches!(
            validate_player_start(&malformed_system),
            Err(PlayerStartValidationError::System { .. })
        ));

        let system = system(1, 0, 0);
        let (_temp, malformed_catalog) = snapshot(&system, Some(b"{}"), Some(MAP));
        assert!(matches!(
            validate_player_start(&malformed_catalog),
            Err(PlayerStartValidationError::Catalog { .. })
        ));

        let (_temp, missing_map) = snapshot(&system, Some(MAP_INFOS), None);
        assert!(matches!(
            validate_player_start(&missing_map),
            Err(PlayerStartValidationError::Map { .. })
        ));
    }

    #[test]
    fn error_sources_remain_available() {
        let (_temp, snapshot) = snapshot(b"{}", None, None);
        let error = validate_player_start(&snapshot).unwrap_err();

        assert!(std::error::Error::source(&error).is_some());
    }
}
