// SPDX-License-Identifier: MPL-2.0

//! Experimental contextual validation of map-to-tileset references.

use crate::rpg_maker_mz::map_catalog::{MapCatalogError, MapId, map_catalog};
use crate::rpg_maker_mz::map_summary::{MapSummaryError, map_summary_from_catalog};
use crate::rpg_maker_mz::snapshot::ProjectSnapshot;
use crate::rpg_maker_mz::tileset_catalog::{TilesetCatalogError, TilesetId, tileset_catalog};
use std::fmt;

/// A read-only report for all cataloged map-to-tileset references.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MapTilesetValidation {
    map_count: usize,
    findings: Vec<MapTilesetFinding>,
}

impl MapTilesetValidation {
    /// Returns the number of cataloged maps whose summaries were checked.
    pub fn map_count(&self) -> usize {
        self.map_count
    }

    /// Returns findings in ascending map-ID order.
    ///
    /// Findings describe relationships established by Tilewright's bounded
    /// evidence. They do not claim that RPG Maker MZ accepts or rejects the
    /// project.
    pub fn findings(&self) -> &[MapTilesetFinding] {
        &self.findings
    }

    /// Returns `true` when every checked map references a cataloged tileset.
    pub fn is_finding_free(&self) -> bool {
        self.findings.is_empty()
    }
}

/// A contextual finding about one map-to-tileset reference.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapTilesetFinding {
    /// The map's positive tileset ID has no record in the coherent catalog.
    #[non_exhaustive]
    MissingTileset {
        /// The map containing the unresolved reference.
        map_id: MapId,
        /// The referenced tileset ID.
        tileset_id: TilesetId,
    },
}

impl fmt::Display for MapTilesetFinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTileset {
                map_id, tileset_id, ..
            } => write!(
                formatter,
                "map {map_id} refers to tileset {tileset_id}, which has no tileset-catalog record"
            ),
        }
    }
}

/// Why map-to-tileset references could not be validated coherently.
#[derive(Debug)]
#[non_exhaustive]
pub enum MapTilesetValidationError {
    /// The map catalog could not be projected.
    #[non_exhaustive]
    MapCatalog { source: MapCatalogError },
    /// The tileset catalog could not be projected.
    #[non_exhaustive]
    TilesetCatalog { source: TilesetCatalogError },
    /// One catalog-selected map could not be summarized.
    #[non_exhaustive]
    Map {
        map_id: MapId,
        source: MapSummaryError,
    },
}

impl fmt::Display for MapTilesetValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MapCatalog { source, .. } => {
                write!(formatter, "map catalog is unavailable: {source}")
            }
            Self::TilesetCatalog { source, .. } => {
                write!(formatter, "tileset catalog is unavailable: {source}")
            }
            Self::Map { map_id, source, .. } => {
                write!(formatter, "map {map_id} summary is unavailable: {source}")
            }
        }
    }
}

impl std::error::Error for MapTilesetValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MapCatalog { source, .. } => Some(source),
            Self::TilesetCatalog { source, .. } => Some(source),
            Self::Map { source, .. } => Some(source),
        }
    }
}

/// Validates every cataloged map's positive tileset reference.
///
/// This pure operation composes the existing map catalog, selected-map
/// summary, and tileset catalog projections over `snapshot`. It checks maps in
/// ascending map-ID order and reports a finding when a selected map's positive
/// `tilesetId` has no matching tileset record.
///
/// It does not perform filesystem I/O, mutate the snapshot, inspect tileset
/// modes, images, flags, or assets, establish editor acceptance, assign
/// severities, or persist changes. Unrelated raw fields remain untouched.
///
/// # Errors
///
/// Returns [`MapTilesetValidationError`] when either prerequisite catalog or
/// any catalog-selected map summary is structurally unavailable. Snapshot
/// diagnostics unrelated to these required documents remain separate.
pub fn validate_map_tilesets(
    snapshot: &ProjectSnapshot,
) -> Result<MapTilesetValidation, MapTilesetValidationError> {
    let maps =
        map_catalog(snapshot).map_err(|source| MapTilesetValidationError::MapCatalog { source })?;
    let tilesets = tileset_catalog(snapshot)
        .map_err(|source| MapTilesetValidationError::TilesetCatalog { source })?;
    let mut findings = Vec::new();

    for map_id in maps.records().keys().copied() {
        let summary = map_summary_from_catalog(snapshot, &maps, map_id)
            .map_err(|source| MapTilesetValidationError::Map { map_id, source })?;
        let tileset_id = TilesetId::from_nonzero(summary.tileset_id_nonzero());
        if tilesets.get(tileset_id).is_none() {
            findings.push(MapTilesetFinding::MissingTileset { map_id, tileset_id });
        }
    }

    Ok(MapTilesetValidation {
        map_count: maps.records().len(),
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
        map_infos: &[u8],
        maps: &[(u32, &[u8])],
        tilesets: &[u8],
    ) -> (TempDir, ProjectSnapshot) {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("data")).unwrap();
        fs::write(temp.path().join("data/MapInfos.json"), map_infos).unwrap();
        fs::write(temp.path().join("data/Tilesets.json"), tilesets).unwrap();
        for (id, source) in maps {
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

    #[test]
    fn validates_all_maps_without_changing_raw_bytes() {
        let map_one = br#"{"displayName":"One","width":10,"height":8,"tilesetId":1,"events":[],"opaque":{"keep":true}}"#;
        let map_two = br#"{"displayName":"Two","width":20,"height":15,"tilesetId":2,"events":[null],"data":"opaque"}"#;
        let tilesets =
            br#"[null,{"id":1,"name":"Field","flags":[1]},{"id":2,"name":"Area","mode":1}]"#;
        let (_temp, snapshot) = snapshot(
            br#"[null,{"id":1,"name":"One","order":2,"parentId":0},{"id":2,"name":"Two","order":1,"parentId":0}]"#,
            &[(1, map_one), (2, map_two)],
            tilesets,
        );

        let validation = validate_map_tilesets(&snapshot).unwrap();

        assert_eq!(validation.map_count(), 2);
        assert!(validation.is_finding_free());
        assert!(validation.findings().is_empty());
        assert_eq!(
            snapshot.documents()[Path::new("data/Map001.json")].source_bytes(),
            map_one
        );
        assert_eq!(
            snapshot.documents()[Path::new("data/Tilesets.json")].source_bytes(),
            tilesets
        );
    }

    #[test]
    fn reports_missing_tilesets_in_map_id_order() {
        let (_temp, snapshot) = snapshot(
            br#"[null,{"id":1,"name":"One","order":2,"parentId":0},{"id":2,"name":"Two","order":1,"parentId":0}]"#,
            &[
                (1, br#"{"displayName":"One","width":1,"height":1,"tilesetId":3,"events":[]}"#),
                (2, br#"{"displayName":"Two","width":1,"height":1,"tilesetId":2,"events":[]}"#),
            ],
            br#"[null,{"id":1,"name":"Only"}]"#,
        );

        let validation = validate_map_tilesets(&snapshot).unwrap();

        assert_eq!(validation.map_count(), 2);
        assert_eq!(
            validation.findings(),
            &[
                MapTilesetFinding::MissingTileset {
                    map_id: MapId::new(1).unwrap(),
                    tileset_id: TilesetId::new(3).unwrap(),
                },
                MapTilesetFinding::MissingTileset {
                    map_id: MapId::new(2).unwrap(),
                    tileset_id: TilesetId::new(2).unwrap(),
                },
            ]
        );
        assert!(!validation.is_finding_free());
    }

    #[test]
    fn distinguishes_each_structural_prerequisite_error() {
        let (_temp, invalid_maps) = snapshot(br#"{}"#, &[], br#"[null,{"id":1,"name":"One"}]"#);
        let error = validate_map_tilesets(&invalid_maps).unwrap_err();
        assert!(matches!(
            error,
            MapTilesetValidationError::MapCatalog { .. }
        ));
        assert!(std::error::Error::source(&error).is_some());

        let (_temp, invalid_tilesets) = snapshot(
            br#"[null,{"id":1,"name":"One","order":1,"parentId":0}]"#,
            &[(
                1,
                br#"{"displayName":"One","width":1,"height":1,"tilesetId":1,"events":[]}"#,
            )],
            br#"{}"#,
        );
        let error = validate_map_tilesets(&invalid_tilesets).unwrap_err();
        assert!(matches!(
            error,
            MapTilesetValidationError::TilesetCatalog { .. }
        ));
        assert!(std::error::Error::source(&error).is_some());

        let (_temp, invalid_map) = snapshot(
            br#"[null,{"id":1,"name":"One","order":1,"parentId":0}]"#,
            &[(1, br#"{"displayName":"One"}"#)],
            br#"[null,{"id":1,"name":"One"}]"#,
        );
        let error = validate_map_tilesets(&invalid_map).unwrap_err();
        assert!(matches!(
            error,
            MapTilesetValidationError::Map {
                map_id,
                source: MapSummaryError::MissingField { .. },
            } if map_id == MapId::new(1).unwrap()
        ));
        assert!(std::error::Error::source(&error).is_some());
    }
}
