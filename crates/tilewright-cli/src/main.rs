// SPDX-License-Identifier: MPL-2.0

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tilewright::json::{LosslessJsonDocument, LosslessJsonError};
use tilewright::rpg_maker_mz::discovery::{
    CandidateDiscovery, MarkerEntryKind, MarkerObservation, discover_candidate,
};
use tilewright::rpg_maker_mz::inventory::{
    ExtensionCandidateFamily, InventoryClassification, InventoryEntryKind, KnownEntryFamily,
    ProjectInventory, inventory_project,
};
use tilewright::rpg_maker_mz::map_catalog::{
    JsonValueKind, MapCatalog, MapCatalogError, MapCatalogFinding, MapId, MapInfoField, map_catalog,
};
use tilewright::rpg_maker_mz::map_events::{
    MapEventCatalog, MapEventCatalogError, MapEventField, MapEventFinding, MapEventMapField,
    map_event_catalog,
};
use tilewright::rpg_maker_mz::map_summary::{MapSummary, MapSummaryError, map_summary};
use tilewright::rpg_maker_mz::map_tileset_validation::{
    MapTilesetFinding, MapTilesetValidation, MapTilesetValidationError, validate_map_tilesets,
};
use tilewright::rpg_maker_mz::player_start_validation::{
    PlayerStartFinding, PlayerStartValidation, PlayerStartValidationError, validate_player_start,
};
use tilewright::rpg_maker_mz::snapshot::{
    DocumentDiagnostic, ProjectSnapshot, SnapshotCompleteness, SnapshotLimits, load_snapshot,
};
use tilewright::rpg_maker_mz::system_summary::{SystemSummary, SystemSummaryError, system_summary};
use tilewright::rpg_maker_mz::tileset_catalog::{
    TilesetCatalog, TilesetCatalogError, TilesetField, tileset_catalog,
};

const OUTPUT_SCHEMA_VERSION: u8 = 1;

fn parse_max_bytes(s: &str) -> Result<usize, String> {
    let val: usize = s.parse().map_err(|_| "must be a valid positive integer")?;
    if val == 0 {
        return Err("max_bytes must be greater than 0".to_string());
    }
    if val == usize::MAX {
        return Err(format!("max_bytes must be less than {}", usize::MAX));
    }
    Ok(val)
}

fn parse_nonzero_usize(s: &str) -> Result<NonZeroUsize, String> {
    let value: usize = s.parse().map_err(|_| "must be a positive integer")?;
    NonZeroUsize::new(value).ok_or_else(|| "must be greater than zero".to_string())
}

fn parse_map_id(s: &str) -> Result<MapId, String> {
    let value: u32 = s.parse().map_err(|_| "must be a positive integer")?;
    MapId::new(value).ok_or_else(|| "must be greater than zero".to_string())
}

#[derive(Clone, Copy, Debug, Args)]
struct SnapshotLimitArgs {
    /// Maximum number of regular candidate documents to attempt.
    #[arg(
        long,
        default_value = "1024",
        value_parser = parse_nonzero_usize
    )]
    max_documents: NonZeroUsize,
    /// Maximum bytes to examine for one candidate document.
    #[arg(
        long,
        default_value = "16777216",
        value_parser = parse_nonzero_usize
    )]
    max_bytes_per_document: NonZeroUsize,
    /// Maximum aggregate bytes to examine across candidate documents.
    #[arg(
        long,
        default_value = "268435456",
        value_parser = parse_nonzero_usize
    )]
    max_aggregate_bytes: NonZeroUsize,
}

impl SnapshotLimitArgs {
    fn limits(self) -> SnapshotLimits {
        SnapshotLimits {
            max_documents: self.max_documents,
            max_bytes_per_document: self.max_bytes_per_document,
            max_aggregate_bytes: self.max_aggregate_bytes,
        }
    }
}

/// Inspect tile-based RPG project data through the Tilewright library.
#[derive(Debug, Parser)]
#[command(name = "tilewright", version = tilewright::VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect one explicit directory for an RPG Maker MZ project marker.
    Discover {
        /// Directory to inspect without recursion.
        path: PathBuf,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Inventory all entries in an RPG Maker MZ project directory.
    Inventory {
        /// Directory to inventory.
        path: PathBuf,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Load a bounded, read-only raw project snapshot.
    Snapshot {
        /// RPG Maker MZ project directory to load.
        path: PathBuf,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        #[command(flatten)]
        limits: SnapshotLimitArgs,
    },
    /// List the experimental typed RPG Maker MZ map catalog.
    Maps {
        /// RPG Maker MZ project directory to inspect.
        path: PathBuf,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        #[command(flatten)]
        limits: SnapshotLimitArgs,
    },
    /// List the experimental typed RPG Maker MZ tileset catalog.
    Tilesets {
        /// RPG Maker MZ project directory to inspect.
        path: PathBuf,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        #[command(flatten)]
        limits: SnapshotLimitArgs,
    },
    /// Summarize one experimental, catalog-selected RPG Maker MZ map.
    Map {
        /// RPG Maker MZ project directory to inspect.
        path: PathBuf,
        /// Positive map ID from the project's map catalog.
        #[arg(value_parser = parse_map_id)]
        id: MapId,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        #[command(flatten)]
        limits: SnapshotLimitArgs,
    },
    /// List bounded events on one catalog-selected RPG Maker MZ map.
    Events {
        /// RPG Maker MZ project directory to inspect.
        path: PathBuf,
        /// Positive map ID from the project's map catalog.
        #[arg(value_parser = parse_map_id)]
        map_id: MapId,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        #[command(flatten)]
        limits: SnapshotLimitArgs,
    },
    /// Summarize selected experimental RPG Maker MZ system settings.
    System {
        /// RPG Maker MZ project directory to inspect.
        path: PathBuf,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        #[command(flatten)]
        limits: SnapshotLimitArgs,
    },
    /// Run experimental contextual validation of the player start.
    Validate {
        /// RPG Maker MZ project directory to inspect.
        path: PathBuf,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        #[command(flatten)]
        limits: SnapshotLimitArgs,
    },
    /// Validate experimental map-to-tileset references.
    ValidateTilesets {
        /// RPG Maker MZ project directory to inspect.
        path: PathBuf,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        #[command(flatten)]
        limits: SnapshotLimitArgs,
    },
    /// Inspect a file for strict lossless JSON syntax.
    InspectJson {
        /// File to inspect.
        path: PathBuf,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        /// Maximum accepted file size in bytes; one extra byte may be read to detect overflow.
        #[arg(long, default_value_t = 10_485_760, value_parser = parse_max_bytes)]
        max_bytes: usize,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Serialize)]
struct DiscoveryReport {
    schema_version: u8,
    root: PathReport,
    result: DiscoveryResult,
    markers: Vec<MarkerReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum DiscoveryResult {
    Candidate,
    CaseVariantCandidate,
    NoMarker,
    AmbiguousMarkers,
    SymlinkMarker,
    NonRegularMarker,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct MarkerReport {
    path: PathReport,
    kind: MarkerKind,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum MarkerKind {
    RegularFile,
    Symlink,
    Directory,
    OtherNonRegular,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct InventoryReport {
    schema_version: u8,
    root: PathReport,
    entries: Vec<InventoryEntryReport>,
}

#[derive(Debug, Serialize)]
struct InventoryEntryReport {
    path: PathReport,
    kind: EntryKindReport,
    classification: ClassificationReport,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum EntryKindReport {
    File,
    Directory,
    Symlink,
    Other,
    Unrecognized,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum ClassificationReport {
    Known { family: KnownFamilyReport },
    ExtensionCandidate { family: ExtensionFamilyReport },
    Unknown,
    Unrecognized,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum KnownFamilyReport {
    StandardRootEntry,
    StandardDataFile,
    MapDataFile,
    Unrecognized,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExtensionFamilyReport {
    DataJson,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct PathReport {
    /// Exact path when it can be represented as UTF-8.
    utf8: Option<String>,
    /// Lossy display form for diagnostics and human presentation.
    display: String,
}

#[derive(Debug, Serialize)]
struct ErrorReport {
    schema_version: u8,
    root: PathReport,
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    message: String,
    cause: Option<String>,
}

#[derive(Debug, Serialize)]
struct InspectJsonReport {
    schema_version: u8,
    path: PathReport,
    byte_length: usize,
    strict_syntax_accepted: bool,
    byte_identical: bool,
}

#[derive(Debug, Serialize)]
struct InspectJsonErrorReport {
    schema_version: u8,
    path: PathReport,
    error: InspectJsonErrorDetail,
}

#[derive(Debug, Serialize)]
struct InspectJsonErrorDetail {
    category: InspectJsonErrorCategory,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    byte_range: Option<RangeReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum InspectJsonErrorCategory {
    IoError,
    TooLarge,
    InvalidUtf8,
    Utf8Bom,
    InvalidSyntax,
    ByteIdentityMismatch,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct RangeReport {
    start: usize,
    end: usize,
}

#[derive(Debug, Serialize)]
struct SnapshotReport {
    schema_version: u8,
    root: PathReport,
    completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    inventory_entry_count: usize,
    loaded_document_count: usize,
    diagnostic_count: usize,
    documents: Vec<SnapshotDocumentReport>,
    diagnostics: Vec<SnapshotDiagnosticReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum SnapshotCompletenessReport {
    Complete,
    Partial,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct SnapshotLimitsReport {
    max_documents: usize,
    max_bytes_per_document: usize,
    max_aggregate_bytes: usize,
}

#[derive(Debug, Serialize)]
struct SnapshotDocumentReport {
    path: PathReport,
    byte_length: usize,
}

#[derive(Debug, Serialize)]
struct SnapshotDiagnosticReport {
    path: PathReport,
    category: SnapshotDiagnosticCategory,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    byte_range: Option<RangeReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum SnapshotDiagnosticCategory {
    UnsupportedEntryKind,
    OpenError,
    ReadError,
    ExceedsDocumentByteLimit,
    ExceedsAggregateByteLimit,
    ExceedsDocumentCountLimit,
    InvalidUtf8,
    Utf8Bom,
    InvalidSyntax,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct MapsReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    map_count: usize,
    finding_count: usize,
    snapshot_diagnostic_count: usize,
    maps: Vec<MapReport>,
    findings: Vec<MapFindingReport>,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
}

#[derive(Debug, Serialize)]
struct MapReport {
    id: u32,
    name: String,
    order: u32,
    parent_id: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "category")]
enum MapFindingReport {
    MissingParent {
        map_id: u32,
        parent_id: u32,
    },
    ParentCycle {
        map_ids: Vec<u32>,
    },
    DuplicateOrder {
        order: u32,
        map_ids: Vec<u32>,
    },
    MissingMapDocument {
        map_id: u32,
        expected_path: PathReport,
    },
    OrphanMapDocument {
        map_id: u32,
        path: PathReport,
    },
    UnrecognizedMapDocumentIdentity {
        path: PathReport,
    },
    UnevidencedMapDocumentPath {
        map_id: u32,
    },
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct MapsErrorReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
    error: MapCatalogErrorDetail,
}

#[derive(Debug, Serialize)]
struct TilesetsReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    tileset_count: usize,
    snapshot_diagnostic_count: usize,
    tilesets: Vec<TilesetReport>,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
}

#[derive(Debug, Serialize)]
struct TilesetReport {
    id: u32,
    name: String,
}

#[derive(Debug, Serialize)]
struct TilesetsErrorReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
    error: TilesetCatalogErrorDetail,
}

#[derive(Debug, Serialize)]
struct TilesetCatalogErrorDetail {
    category: TilesetCatalogErrorCategory,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decoded_id: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum TilesetCatalogErrorCategory {
    MissingDocument,
    UnavailableDocument,
    UnexpectedRootKind,
    UnexpectedEntryKind,
    MissingField,
    DuplicateField,
    UnexpectedFieldKind,
    UnsupportedInteger,
    InvalidString,
    IndexOutOfRange,
    ReservedIndex,
    IdIndexMismatch,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct MapCatalogErrorDetail {
    category: MapCatalogErrorCategory,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decoded_id: Option<u32>,
}

#[derive(Debug, Serialize)]
struct SelectedMapReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostic_count: usize,
    map: SelectedMapDetail,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
}

#[derive(Debug, Serialize)]
struct SelectedMapDetail {
    id: u32,
    catalog_name: String,
    document_path: PathReport,
    display_name: String,
    width: u32,
    height: u32,
    tileset_id: u32,
    event_count: usize,
}

#[derive(Debug, Serialize)]
struct SelectedMapErrorReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
    error: MapSummaryErrorDetail,
}

#[derive(Debug, Serialize)]
struct MapEventsReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostic_count: usize,
    map: MapEventsMapDetail,
    event_count: usize,
    finding_count: usize,
    events: Vec<MapEventDetail>,
    findings: Vec<MapEventFindingReport>,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
}

#[derive(Debug, Serialize)]
struct MapEventsMapDetail {
    id: u32,
    catalog_name: String,
    document_path: PathReport,
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize)]
struct MapEventDetail {
    id: u32,
    name: String,
    x: u32,
    y: u32,
    page_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "category")]
enum MapEventFindingReport {
    CoordinatesOutsideMap {
        event_id: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct MapEventsErrorReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
    error: MapEventsErrorDetail,
}

#[derive(Debug, Serialize)]
struct MapEventsErrorDetail {
    category: MapEventsErrorCategory,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    map_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<PathReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decoded_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog_error: Option<MapCatalogErrorDetail>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum MapEventsErrorCategory {
    CatalogError,
    UnknownMapId,
    UnevidencedDocumentPath,
    MissingDocument,
    UnavailableDocument,
    UnexpectedRootKind,
    MissingMapField,
    DuplicateMapField,
    UnexpectedMapFieldKind,
    UnsupportedMapInteger,
    UnexpectedEventEntryKind,
    ReservedEventIndex,
    EventIndexOutOfRange,
    MissingEventField,
    DuplicateEventField,
    UnexpectedEventFieldKind,
    UnsupportedEventInteger,
    InvalidEventString,
    IdIndexMismatch,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct MapSummaryErrorDetail {
    category: MapSummaryErrorCategory,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    map_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<PathReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog_error: Option<MapCatalogErrorDetail>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum MapSummaryErrorCategory {
    CatalogError,
    UnknownMapId,
    UnevidencedDocumentPath,
    MissingDocument,
    UnavailableDocument,
    UnexpectedRootKind,
    MissingField,
    DuplicateField,
    UnexpectedFieldKind,
    UnsupportedInteger,
    InvalidString,
    UnexpectedEventEntryKind,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct SystemReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostic_count: usize,
    system: SystemDetail,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
}

#[derive(Debug, Serialize)]
struct SystemDetail {
    document_path: PathReport,
    game_title: String,
    currency_unit: String,
    locale: String,
    edit_map_id: u32,
    start_map_id: u32,
    start_x: u32,
    start_y: u32,
}

#[derive(Debug, Serialize)]
struct SystemErrorReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
    error: SystemSummaryErrorDetail,
}

#[derive(Debug, Serialize)]
struct SystemSummaryErrorDetail {
    category: SystemSummaryErrorCategory,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<PathReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_kind: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum SystemSummaryErrorCategory {
    MissingDocument,
    UnavailableDocument,
    UnexpectedRootKind,
    MissingField,
    DuplicateField,
    UnexpectedFieldKind,
    UnsupportedInteger,
    InvalidString,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct ValidationReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostic_count: usize,
    validation: PlayerStartValidationDetail,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
}

#[derive(Debug, Serialize)]
struct PlayerStartValidationDetail {
    scope: ValidationScope,
    finding_free: bool,
    start_map_id: u32,
    start_x: u32,
    start_y: u32,
    finding_count: usize,
    findings: Vec<PlayerStartFindingReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ValidationScope {
    PlayerStart,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "category")]
enum PlayerStartFindingReport {
    MissingPlayerStart,
    ZeroMapIdWithCoordinates {
        start_x: u32,
        start_y: u32,
    },
    MissingMapRecord {
        map_id: u32,
    },
    OutOfBounds {
        map_id: u32,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
    },
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct ValidationErrorReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
    error: PlayerStartValidationErrorDetail,
}

#[derive(Debug, Serialize)]
struct PlayerStartValidationErrorDetail {
    category: PlayerStartValidationErrorCategory,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_error: Option<SystemSummaryErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog_error: Option<MapCatalogErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    map_error: Option<MapSummaryErrorDetail>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum PlayerStartValidationErrorCategory {
    SystemError,
    CatalogError,
    MapError,
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct MapTilesetValidationReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostic_count: usize,
    validation: MapTilesetValidationDetail,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
}

#[derive(Debug, Serialize)]
struct MapTilesetValidationDetail {
    scope: MapTilesetValidationScope,
    finding_free: bool,
    map_count: usize,
    finding_count: usize,
    findings: Vec<MapTilesetFindingReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum MapTilesetValidationScope {
    MapTilesetReferences,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "category")]
enum MapTilesetFindingReport {
    MissingTileset { map_id: u32, tileset_id: u32 },
    Unrecognized,
}

#[derive(Debug, Serialize)]
struct MapTilesetValidationErrorReport {
    schema_version: u8,
    root: PathReport,
    snapshot_completeness: SnapshotCompletenessReport,
    limits: SnapshotLimitsReport,
    snapshot_diagnostics: Vec<SnapshotDiagnosticReport>,
    error: MapTilesetValidationErrorDetail,
}

#[derive(Debug, Serialize)]
struct MapTilesetValidationErrorDetail {
    category: MapTilesetValidationErrorCategory,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    map_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    map_catalog_error: Option<MapCatalogErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tileset_catalog_error: Option<TilesetCatalogErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    map_error: Option<MapSummaryErrorDetail>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum MapTilesetValidationErrorCategory {
    MapCatalogError,
    TilesetCatalogError,
    MapError,
    Unrecognized,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum MapCatalogErrorCategory {
    MissingDocument,
    UnavailableDocument,
    UnexpectedRootKind,
    UnexpectedEntryKind,
    MissingField,
    DuplicateField,
    UnexpectedFieldKind,
    UnsupportedInteger,
    InvalidString,
    IndexOutOfRange,
    IdIndexMismatch,
    Unrecognized,
}

impl DiscoveryReport {
    fn new(root: &Path, discovery: &CandidateDiscovery) -> Self {
        let (result, markers) = match discovery {
            CandidateDiscovery::Candidate { marker, .. } => {
                (DiscoveryResult::Candidate, vec![MarkerReport::new(marker)])
            }
            CandidateDiscovery::CaseVariantCandidate { marker, .. } => (
                DiscoveryResult::CaseVariantCandidate,
                vec![MarkerReport::new(marker)],
            ),
            CandidateDiscovery::NoMarker => (DiscoveryResult::NoMarker, Vec::new()),
            CandidateDiscovery::AmbiguousMarkers { markers, .. } => (
                DiscoveryResult::AmbiguousMarkers,
                markers.iter().map(MarkerReport::new).collect(),
            ),
            CandidateDiscovery::SymlinkMarker { marker, .. } => (
                DiscoveryResult::SymlinkMarker,
                vec![MarkerReport::new(marker)],
            ),
            CandidateDiscovery::NonRegularMarker { marker, .. } => (
                DiscoveryResult::NonRegularMarker,
                vec![MarkerReport::new(marker)],
            ),
            _ => (DiscoveryResult::Unrecognized, Vec::new()),
        };

        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            result,
            markers,
        }
    }
}

impl MarkerReport {
    fn new(marker: &MarkerObservation) -> Self {
        let kind = match marker.kind {
            MarkerEntryKind::RegularFile => MarkerKind::RegularFile,
            MarkerEntryKind::Symlink => MarkerKind::Symlink,
            MarkerEntryKind::Directory => MarkerKind::Directory,
            MarkerEntryKind::OtherNonRegular => MarkerKind::OtherNonRegular,
            _ => MarkerKind::Unrecognized,
        };

        Self {
            path: PathReport::new(&marker.path),
            kind,
        }
    }
}

impl InventoryReport {
    fn new(root: &Path, inventory: &ProjectInventory) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            entries: inventory
                .entries
                .iter()
                .map(InventoryEntryReport::new)
                .collect(),
        }
    }
}

impl InventoryEntryReport {
    fn new(entry: &tilewright::rpg_maker_mz::inventory::InventoryEntry) -> Self {
        let kind = match entry.kind {
            InventoryEntryKind::File => EntryKindReport::File,
            InventoryEntryKind::Directory => EntryKindReport::Directory,
            InventoryEntryKind::Symlink => EntryKindReport::Symlink,
            InventoryEntryKind::Other => EntryKindReport::Other,
            _ => EntryKindReport::Unrecognized,
        };

        let classification = match entry.classification {
            InventoryClassification::Known { family, .. } => ClassificationReport::Known {
                family: match family {
                    KnownEntryFamily::StandardRootEntry => KnownFamilyReport::StandardRootEntry,
                    KnownEntryFamily::StandardDataFile => KnownFamilyReport::StandardDataFile,
                    KnownEntryFamily::MapDataFile => KnownFamilyReport::MapDataFile,
                    _ => KnownFamilyReport::Unrecognized,
                },
            },
            InventoryClassification::ExtensionCandidate { family, .. } => {
                ClassificationReport::ExtensionCandidate {
                    family: match family {
                        ExtensionCandidateFamily::DataJson => ExtensionFamilyReport::DataJson,
                        _ => ExtensionFamilyReport::Unrecognized,
                    },
                }
            }
            InventoryClassification::Unknown => ClassificationReport::Unknown,
            _ => ClassificationReport::Unrecognized,
        };

        Self {
            path: PathReport::new(&entry.path),
            kind,
            classification,
        }
    }
}

impl PathReport {
    fn new(path: &Path) -> Self {
        Self {
            utf8: path.to_str().map(ToOwned::to_owned),
            display: path.display().to_string(),
        }
    }
}

impl SnapshotReport {
    fn new(root: &Path, snapshot: &ProjectSnapshot, limits: SnapshotLimits) -> Self {
        let documents = snapshot
            .documents()
            .iter()
            .map(|(path, document)| SnapshotDocumentReport {
                path: PathReport::new(path),
                byte_length: document.source_bytes().len(),
            })
            .collect();
        let diagnostics = snapshot
            .diagnostics()
            .iter()
            .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
            .collect();

        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            inventory_entry_count: snapshot.inventory().entries.len(),
            loaded_document_count: snapshot.documents().len(),
            diagnostic_count: snapshot.diagnostics().len(),
            documents,
            diagnostics,
        }
    }
}

impl SnapshotDiagnosticReport {
    fn new(path: &Path, diagnostic: &DocumentDiagnostic) -> Self {
        let (category, cause, byte_range) = match diagnostic {
            DocumentDiagnostic::UnsupportedEntryKind { .. } => {
                (SnapshotDiagnosticCategory::UnsupportedEntryKind, None, None)
            }
            DocumentDiagnostic::Open { source, .. } => (
                SnapshotDiagnosticCategory::OpenError,
                Some(source.to_string()),
                None,
            ),
            DocumentDiagnostic::Read { source, .. } => (
                SnapshotDiagnosticCategory::ReadError,
                Some(source.to_string()),
                None,
            ),
            DocumentDiagnostic::ExceedsDocumentByteLimit { .. } => (
                SnapshotDiagnosticCategory::ExceedsDocumentByteLimit,
                None,
                None,
            ),
            DocumentDiagnostic::ExceedsAggregateByteLimit { .. } => (
                SnapshotDiagnosticCategory::ExceedsAggregateByteLimit,
                None,
                None,
            ),
            DocumentDiagnostic::ExceedsDocumentCountLimit { .. } => (
                SnapshotDiagnosticCategory::ExceedsDocumentCountLimit,
                None,
                None,
            ),
            DocumentDiagnostic::Parse { source, .. } => match source {
                LosslessJsonError::InvalidUtf8 { source, .. } => (
                    SnapshotDiagnosticCategory::InvalidUtf8,
                    Some(source.to_string()),
                    None,
                ),
                LosslessJsonError::Utf8ByteOrderMark => {
                    (SnapshotDiagnosticCategory::Utf8Bom, None, None)
                }
                LosslessJsonError::InvalidSyntax { diagnostic, .. } => (
                    SnapshotDiagnosticCategory::InvalidSyntax,
                    None,
                    Some(RangeReport {
                        start: diagnostic.byte_range().start,
                        end: diagnostic.byte_range().end,
                    }),
                ),
                _ => (SnapshotDiagnosticCategory::Unrecognized, None, None),
            },
            _ => (SnapshotDiagnosticCategory::Unrecognized, None, None),
        };

        Self {
            path: PathReport::new(path),
            category,
            message: diagnostic.to_string(),
            cause,
            byte_range,
        }
    }
}

impl MapsReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        catalog: &MapCatalog,
        limits: SnapshotLimits,
    ) -> Self {
        let maps = catalog
            .records_in_display_order()
            .map(|record| MapReport {
                id: record.id().get(),
                name: record.name().to_owned(),
                order: record.order(),
                parent_id: record.parent_id().map(|id| id.get()),
            })
            .collect();
        let findings = catalog
            .findings()
            .iter()
            .map(MapFindingReport::new)
            .collect();
        let snapshot_diagnostics = snapshot
            .diagnostics()
            .iter()
            .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
            .collect();

        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            map_count: catalog.records().len(),
            finding_count: catalog.findings().len(),
            snapshot_diagnostic_count: snapshot.diagnostics().len(),
            maps,
            findings,
            snapshot_diagnostics,
        }
    }
}

impl TilesetsReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        catalog: &TilesetCatalog,
        limits: SnapshotLimits,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            tileset_count: catalog.records().len(),
            snapshot_diagnostic_count: snapshot.diagnostics().len(),
            tilesets: catalog
                .records()
                .values()
                .map(|record| TilesetReport {
                    id: record.id().get(),
                    name: record.name().to_owned(),
                })
                .collect(),
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
        }
    }
}

impl SelectedMapReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        summary: &MapSummary,
        limits: SnapshotLimits,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostic_count: snapshot.diagnostics().len(),
            map: SelectedMapDetail {
                id: summary.id().get(),
                catalog_name: summary.catalog_name().to_owned(),
                document_path: PathReport::new(summary.document_path()),
                display_name: summary.display_name().to_owned(),
                width: summary.width(),
                height: summary.height(),
                tileset_id: summary.tileset_id(),
                event_count: summary.event_count(),
            },
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
        }
    }
}

impl SelectedMapErrorReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        limits: SnapshotLimits,
        error: &MapSummaryError,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
            error: MapSummaryErrorDetail::new(error),
        }
    }
}

impl MapEventsReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        catalog: &MapEventCatalog,
        limits: SnapshotLimits,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostic_count: snapshot.diagnostics().len(),
            map: MapEventsMapDetail {
                id: catalog.map_id().get(),
                catalog_name: catalog.catalog_name().to_owned(),
                document_path: PathReport::new(catalog.document_path()),
                width: catalog.width(),
                height: catalog.height(),
            },
            event_count: catalog.records().len(),
            finding_count: catalog.findings().len(),
            events: catalog
                .records()
                .values()
                .map(|event| MapEventDetail {
                    id: event.id().get(),
                    name: event.name().to_owned(),
                    x: event.x(),
                    y: event.y(),
                    page_count: event.page_count(),
                })
                .collect(),
            findings: catalog
                .findings()
                .iter()
                .map(MapEventFindingReport::new)
                .collect(),
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
        }
    }
}

impl MapEventFindingReport {
    fn new(finding: &MapEventFinding) -> Self {
        match finding {
            MapEventFinding::CoordinatesOutsideMap {
                event_id,
                x,
                y,
                width,
                height,
                ..
            } => Self::CoordinatesOutsideMap {
                event_id: event_id.get(),
                x: *x,
                y: *y,
                width: *width,
                height: *height,
            },
            _ => Self::Unrecognized,
        }
    }
}

impl MapEventsErrorReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        limits: SnapshotLimits,
        error: &MapEventCatalogError,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
            error: MapEventsErrorDetail::new(error),
        }
    }
}

impl MapEventsErrorDetail {
    fn new(error: &MapEventCatalogError) -> Self {
        let mut detail = Self {
            category: MapEventsErrorCategory::Unrecognized,
            message: error.to_string(),
            map_id: None,
            path: None,
            index: None,
            field: None,
            actual_kind: None,
            decoded_id: None,
            catalog_error: None,
        };

        match error {
            MapEventCatalogError::Catalog { source, .. } => {
                detail.category = MapEventsErrorCategory::CatalogError;
                detail.catalog_error = Some(MapCatalogErrorDetail::new(source));
            }
            MapEventCatalogError::UnknownMapId { map_id, .. } => {
                detail.category = MapEventsErrorCategory::UnknownMapId;
                detail.map_id = Some(map_id.get());
            }
            MapEventCatalogError::UnevidencedDocumentPath { map_id, .. } => {
                detail.category = MapEventsErrorCategory::UnevidencedDocumentPath;
                detail.map_id = Some(map_id.get());
            }
            MapEventCatalogError::MissingDocument { map_id, path, .. } => {
                detail.category = MapEventsErrorCategory::MissingDocument;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
            }
            MapEventCatalogError::UnavailableDocument { map_id, path, .. } => {
                detail.category = MapEventsErrorCategory::UnavailableDocument;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
            }
            MapEventCatalogError::UnexpectedRootKind {
                map_id,
                path,
                actual,
                ..
            } => {
                detail.category = MapEventsErrorCategory::UnexpectedRootKind;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.actual_kind = Some(json_value_kind_name(*actual));
            }
            MapEventCatalogError::MissingMapField {
                map_id,
                path,
                field,
                ..
            } => detail.set_map_field(
                MapEventsErrorCategory::MissingMapField,
                *map_id,
                path,
                *field,
                None,
            ),
            MapEventCatalogError::DuplicateMapField {
                map_id,
                path,
                field,
                ..
            } => detail.set_map_field(
                MapEventsErrorCategory::DuplicateMapField,
                *map_id,
                path,
                *field,
                None,
            ),
            MapEventCatalogError::UnexpectedMapFieldKind {
                map_id,
                path,
                field,
                actual,
                ..
            } => detail.set_map_field(
                MapEventsErrorCategory::UnexpectedMapFieldKind,
                *map_id,
                path,
                *field,
                Some(*actual),
            ),
            MapEventCatalogError::UnsupportedMapInteger {
                map_id,
                path,
                field,
                ..
            } => detail.set_map_field(
                MapEventsErrorCategory::UnsupportedMapInteger,
                *map_id,
                path,
                *field,
                None,
            ),
            MapEventCatalogError::UnexpectedEventEntryKind {
                map_id,
                path,
                index,
                actual,
                ..
            } => {
                detail.category = MapEventsErrorCategory::UnexpectedEventEntryKind;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.index = Some(*index);
                detail.actual_kind = Some(json_value_kind_name(*actual));
            }
            MapEventCatalogError::ReservedEventIndex { map_id, path, .. } => {
                detail.category = MapEventsErrorCategory::ReservedEventIndex;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.index = Some(0);
            }
            MapEventCatalogError::EventIndexOutOfRange {
                map_id,
                path,
                index,
                ..
            } => {
                detail.category = MapEventsErrorCategory::EventIndexOutOfRange;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.index = Some(*index);
            }
            MapEventCatalogError::MissingEventField {
                map_id,
                path,
                index,
                field,
                ..
            } => detail.set_event_field(
                MapEventsErrorCategory::MissingEventField,
                *map_id,
                path,
                *index,
                *field,
                None,
            ),
            MapEventCatalogError::DuplicateEventField {
                map_id,
                path,
                index,
                field,
                ..
            } => detail.set_event_field(
                MapEventsErrorCategory::DuplicateEventField,
                *map_id,
                path,
                *index,
                *field,
                None,
            ),
            MapEventCatalogError::UnexpectedEventFieldKind {
                map_id,
                path,
                index,
                field,
                actual,
                ..
            } => detail.set_event_field(
                MapEventsErrorCategory::UnexpectedEventFieldKind,
                *map_id,
                path,
                *index,
                *field,
                Some(*actual),
            ),
            MapEventCatalogError::UnsupportedEventInteger {
                map_id,
                path,
                index,
                field,
                ..
            } => detail.set_event_field(
                MapEventsErrorCategory::UnsupportedEventInteger,
                *map_id,
                path,
                *index,
                *field,
                None,
            ),
            MapEventCatalogError::InvalidEventString {
                map_id,
                path,
                index,
                field,
                ..
            } => detail.set_event_field(
                MapEventsErrorCategory::InvalidEventString,
                *map_id,
                path,
                *index,
                *field,
                None,
            ),
            MapEventCatalogError::IdIndexMismatch {
                map_id,
                path,
                index,
                decoded_id,
                ..
            } => {
                detail.category = MapEventsErrorCategory::IdIndexMismatch;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.index = Some(*index);
                detail.field = Some(MapEventField::Id.to_string());
                detail.decoded_id = Some(decoded_id.get());
            }
            _ => {}
        }

        detail
    }

    fn set_map_field(
        &mut self,
        category: MapEventsErrorCategory,
        map_id: MapId,
        path: &Path,
        field: MapEventMapField,
        actual: Option<JsonValueKind>,
    ) {
        self.category = category;
        self.map_id = Some(map_id.get());
        self.path = Some(PathReport::new(path));
        self.field = Some(field.to_string());
        self.actual_kind = actual.map(json_value_kind_name);
    }

    fn set_event_field(
        &mut self,
        category: MapEventsErrorCategory,
        map_id: MapId,
        path: &Path,
        index: usize,
        field: MapEventField,
        actual: Option<JsonValueKind>,
    ) {
        self.category = category;
        self.map_id = Some(map_id.get());
        self.path = Some(PathReport::new(path));
        self.index = Some(index);
        self.field = Some(field.to_string());
        self.actual_kind = actual.map(json_value_kind_name);
    }
}

impl SystemReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        summary: &SystemSummary,
        limits: SnapshotLimits,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostic_count: snapshot.diagnostics().len(),
            system: SystemDetail {
                document_path: PathReport::new(summary.document_path()),
                game_title: summary.game_title().to_owned(),
                currency_unit: summary.currency_unit().to_owned(),
                locale: summary.locale().to_owned(),
                edit_map_id: summary.edit_map_id(),
                start_map_id: summary.start_map_id(),
                start_x: summary.start_x(),
                start_y: summary.start_y(),
            },
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
        }
    }
}

impl SystemErrorReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        limits: SnapshotLimits,
        error: &SystemSummaryError,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
            error: SystemSummaryErrorDetail::new(error),
        }
    }
}

impl SystemSummaryErrorDetail {
    fn new(error: &SystemSummaryError) -> Self {
        let mut detail = Self {
            category: SystemSummaryErrorCategory::Unrecognized,
            message: error.to_string(),
            path: None,
            field: None,
            actual_kind: None,
        };

        match error {
            SystemSummaryError::MissingDocument { path, .. } => {
                detail.category = SystemSummaryErrorCategory::MissingDocument;
                detail.path = Some(PathReport::new(path));
            }
            SystemSummaryError::UnavailableDocument { path, .. } => {
                detail.category = SystemSummaryErrorCategory::UnavailableDocument;
                detail.path = Some(PathReport::new(path));
            }
            SystemSummaryError::UnexpectedRootKind { path, actual, .. } => {
                detail.category = SystemSummaryErrorCategory::UnexpectedRootKind;
                detail.path = Some(PathReport::new(path));
                detail.actual_kind = Some(json_value_kind_name(*actual));
            }
            SystemSummaryError::MissingField { path, field, .. } => {
                detail.category = SystemSummaryErrorCategory::MissingField;
                detail.path = Some(PathReport::new(path));
                detail.field = Some(field.to_string());
            }
            SystemSummaryError::DuplicateField { path, field, .. } => {
                detail.category = SystemSummaryErrorCategory::DuplicateField;
                detail.path = Some(PathReport::new(path));
                detail.field = Some(field.to_string());
            }
            SystemSummaryError::UnexpectedFieldKind {
                path,
                field,
                actual,
                ..
            } => {
                detail.category = SystemSummaryErrorCategory::UnexpectedFieldKind;
                detail.path = Some(PathReport::new(path));
                detail.field = Some(field.to_string());
                detail.actual_kind = Some(json_value_kind_name(*actual));
            }
            SystemSummaryError::UnsupportedInteger { path, field, .. } => {
                detail.category = SystemSummaryErrorCategory::UnsupportedInteger;
                detail.path = Some(PathReport::new(path));
                detail.field = Some(field.to_string());
            }
            SystemSummaryError::InvalidString { path, field, .. } => {
                detail.category = SystemSummaryErrorCategory::InvalidString;
                detail.path = Some(PathReport::new(path));
                detail.field = Some(field.to_string());
            }
            _ => {}
        }

        detail
    }
}

impl ValidationReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        validation: &PlayerStartValidation,
        limits: SnapshotLimits,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostic_count: snapshot.diagnostics().len(),
            validation: PlayerStartValidationDetail {
                scope: ValidationScope::PlayerStart,
                finding_free: validation.is_finding_free(),
                start_map_id: validation.start_map_id(),
                start_x: validation.start_x(),
                start_y: validation.start_y(),
                finding_count: validation.findings().len(),
                findings: validation
                    .findings()
                    .iter()
                    .map(PlayerStartFindingReport::new)
                    .collect(),
            },
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
        }
    }
}

impl PlayerStartFindingReport {
    fn new(finding: &PlayerStartFinding) -> Self {
        match finding {
            PlayerStartFinding::MissingPlayerStart => Self::MissingPlayerStart,
            PlayerStartFinding::ZeroMapIdWithCoordinates {
                start_x, start_y, ..
            } => Self::ZeroMapIdWithCoordinates {
                start_x: *start_x,
                start_y: *start_y,
            },
            PlayerStartFinding::MissingMapRecord { map_id, .. } => Self::MissingMapRecord {
                map_id: map_id.get(),
            },
            PlayerStartFinding::OutOfBounds {
                map_id,
                start_x,
                start_y,
                width,
                height,
                ..
            } => Self::OutOfBounds {
                map_id: map_id.get(),
                start_x: *start_x,
                start_y: *start_y,
                width: *width,
                height: *height,
            },
            _ => Self::Unrecognized,
        }
    }
}

impl ValidationErrorReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        limits: SnapshotLimits,
        error: &PlayerStartValidationError,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
            error: PlayerStartValidationErrorDetail::new(error),
        }
    }
}

impl PlayerStartValidationErrorDetail {
    fn new(error: &PlayerStartValidationError) -> Self {
        let mut detail = Self {
            category: PlayerStartValidationErrorCategory::Unrecognized,
            message: error.to_string(),
            system_error: None,
            catalog_error: None,
            map_error: None,
        };

        match error {
            PlayerStartValidationError::System { source, .. } => {
                detail.category = PlayerStartValidationErrorCategory::SystemError;
                detail.system_error = Some(SystemSummaryErrorDetail::new(source));
            }
            PlayerStartValidationError::Catalog { source, .. } => {
                detail.category = PlayerStartValidationErrorCategory::CatalogError;
                detail.catalog_error = Some(MapCatalogErrorDetail::new(source));
            }
            PlayerStartValidationError::Map { source, .. } => {
                detail.category = PlayerStartValidationErrorCategory::MapError;
                detail.map_error = Some(MapSummaryErrorDetail::new(source));
            }
            _ => {}
        }

        detail
    }
}

impl MapTilesetValidationReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        validation: &MapTilesetValidation,
        limits: SnapshotLimits,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostic_count: snapshot.diagnostics().len(),
            validation: MapTilesetValidationDetail {
                scope: MapTilesetValidationScope::MapTilesetReferences,
                finding_free: validation.is_finding_free(),
                map_count: validation.map_count(),
                finding_count: validation.findings().len(),
                findings: validation
                    .findings()
                    .iter()
                    .map(MapTilesetFindingReport::new)
                    .collect(),
            },
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
        }
    }
}

impl MapTilesetFindingReport {
    fn new(finding: &MapTilesetFinding) -> Self {
        match finding {
            MapTilesetFinding::MissingTileset {
                map_id, tileset_id, ..
            } => Self::MissingTileset {
                map_id: map_id.get(),
                tileset_id: tileset_id.get(),
            },
            _ => Self::Unrecognized,
        }
    }
}

impl MapTilesetValidationErrorReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        limits: SnapshotLimits,
        error: &MapTilesetValidationError,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
            error: MapTilesetValidationErrorDetail::new(error),
        }
    }
}

impl MapTilesetValidationErrorDetail {
    fn new(error: &MapTilesetValidationError) -> Self {
        let mut detail = Self {
            category: MapTilesetValidationErrorCategory::Unrecognized,
            message: error.to_string(),
            map_id: None,
            map_catalog_error: None,
            tileset_catalog_error: None,
            map_error: None,
        };

        match error {
            MapTilesetValidationError::MapCatalog { source, .. } => {
                detail.category = MapTilesetValidationErrorCategory::MapCatalogError;
                detail.map_catalog_error = Some(MapCatalogErrorDetail::new(source));
            }
            MapTilesetValidationError::TilesetCatalog { source, .. } => {
                detail.category = MapTilesetValidationErrorCategory::TilesetCatalogError;
                detail.tileset_catalog_error = Some(TilesetCatalogErrorDetail::new(source));
            }
            MapTilesetValidationError::Map { map_id, source, .. } => {
                detail.category = MapTilesetValidationErrorCategory::MapError;
                detail.map_id = Some(map_id.get());
                detail.map_error = Some(MapSummaryErrorDetail::new(source));
            }
            _ => {}
        }

        detail
    }
}

impl MapSummaryErrorDetail {
    fn new(error: &MapSummaryError) -> Self {
        let mut detail = Self {
            category: MapSummaryErrorCategory::Unrecognized,
            message: error.to_string(),
            map_id: None,
            path: None,
            field: None,
            index: None,
            actual_kind: None,
            catalog_error: None,
        };

        match error {
            MapSummaryError::Catalog { source, .. } => {
                detail.category = MapSummaryErrorCategory::CatalogError;
                detail.catalog_error = Some(MapCatalogErrorDetail::new(source));
            }
            MapSummaryError::UnknownMapId { map_id, .. } => {
                detail.category = MapSummaryErrorCategory::UnknownMapId;
                detail.map_id = Some(map_id.get());
            }
            MapSummaryError::UnevidencedDocumentPath { map_id, .. } => {
                detail.category = MapSummaryErrorCategory::UnevidencedDocumentPath;
                detail.map_id = Some(map_id.get());
            }
            MapSummaryError::MissingDocument { map_id, path, .. } => {
                detail.category = MapSummaryErrorCategory::MissingDocument;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
            }
            MapSummaryError::UnavailableDocument { map_id, path, .. } => {
                detail.category = MapSummaryErrorCategory::UnavailableDocument;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
            }
            MapSummaryError::UnexpectedRootKind {
                map_id,
                path,
                actual,
                ..
            } => {
                detail.category = MapSummaryErrorCategory::UnexpectedRootKind;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.actual_kind = Some(json_value_kind_name(*actual));
            }
            MapSummaryError::MissingField {
                map_id,
                path,
                field,
                ..
            } => {
                detail.category = MapSummaryErrorCategory::MissingField;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.field = Some(field.to_string());
            }
            MapSummaryError::DuplicateField {
                map_id,
                path,
                field,
                ..
            } => {
                detail.category = MapSummaryErrorCategory::DuplicateField;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.field = Some(field.to_string());
            }
            MapSummaryError::UnexpectedFieldKind {
                map_id,
                path,
                field,
                actual,
                ..
            } => {
                detail.category = MapSummaryErrorCategory::UnexpectedFieldKind;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.field = Some(field.to_string());
                detail.actual_kind = Some(json_value_kind_name(*actual));
            }
            MapSummaryError::UnsupportedInteger {
                map_id,
                path,
                field,
                ..
            } => {
                detail.category = MapSummaryErrorCategory::UnsupportedInteger;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.field = Some(field.to_string());
            }
            MapSummaryError::InvalidString {
                map_id,
                path,
                field,
                ..
            } => {
                detail.category = MapSummaryErrorCategory::InvalidString;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.field = Some(field.to_string());
            }
            MapSummaryError::UnexpectedEventEntryKind {
                map_id,
                path,
                index,
                actual,
                ..
            } => {
                detail.category = MapSummaryErrorCategory::UnexpectedEventEntryKind;
                detail.map_id = Some(map_id.get());
                detail.path = Some(PathReport::new(path));
                detail.index = Some(*index);
                detail.actual_kind = Some(json_value_kind_name(*actual));
            }
            _ => {}
        }

        detail
    }
}

impl MapFindingReport {
    fn new(finding: &MapCatalogFinding) -> Self {
        match finding {
            MapCatalogFinding::MissingParent {
                map_id, parent_id, ..
            } => Self::MissingParent {
                map_id: map_id.get(),
                parent_id: parent_id.get(),
            },
            MapCatalogFinding::ParentCycle { map_ids, .. } => Self::ParentCycle {
                map_ids: map_ids.iter().map(|id| id.get()).collect(),
            },
            MapCatalogFinding::DuplicateOrder { order, map_ids, .. } => Self::DuplicateOrder {
                order: *order,
                map_ids: map_ids.iter().map(|id| id.get()).collect(),
            },
            MapCatalogFinding::MissingMapDocument {
                map_id,
                expected_path,
                ..
            } => Self::MissingMapDocument {
                map_id: map_id.get(),
                expected_path: PathReport::new(expected_path),
            },
            MapCatalogFinding::OrphanMapDocument { map_id, path, .. } => Self::OrphanMapDocument {
                map_id: map_id.get(),
                path: PathReport::new(path),
            },
            MapCatalogFinding::UnrecognizedMapDocumentIdentity { path, .. } => {
                Self::UnrecognizedMapDocumentIdentity {
                    path: PathReport::new(path),
                }
            }
            MapCatalogFinding::UnevidencedMapDocumentPath { map_id, .. } => {
                Self::UnevidencedMapDocumentPath {
                    map_id: map_id.get(),
                }
            }
            _ => Self::Unrecognized,
        }
    }
}

impl MapsErrorReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        limits: SnapshotLimits,
        error: &MapCatalogError,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
            error: MapCatalogErrorDetail::new(error),
        }
    }
}

impl TilesetsErrorReport {
    fn new(
        root: &Path,
        snapshot: &ProjectSnapshot,
        limits: SnapshotLimits,
        error: &TilesetCatalogError,
    ) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            root: PathReport::new(root),
            snapshot_completeness: SnapshotCompletenessReport::new(snapshot.completeness()),
            limits: SnapshotLimitsReport::new(limits),
            snapshot_diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(|(path, diagnostic)| SnapshotDiagnosticReport::new(path, diagnostic))
                .collect(),
            error: TilesetCatalogErrorDetail::new(error),
        }
    }
}

impl TilesetCatalogErrorDetail {
    fn new(error: &TilesetCatalogError) -> Self {
        let (category, index, field, actual_kind, decoded_id) = match error {
            TilesetCatalogError::MissingDocument => (
                TilesetCatalogErrorCategory::MissingDocument,
                None,
                None,
                None,
                None,
            ),
            TilesetCatalogError::UnavailableDocument => (
                TilesetCatalogErrorCategory::UnavailableDocument,
                None,
                None,
                None,
                None,
            ),
            TilesetCatalogError::UnexpectedRootKind { actual, .. } => (
                TilesetCatalogErrorCategory::UnexpectedRootKind,
                None,
                None,
                Some(json_value_kind_name(*actual)),
                None,
            ),
            TilesetCatalogError::UnexpectedEntryKind { index, actual, .. } => (
                TilesetCatalogErrorCategory::UnexpectedEntryKind,
                Some(*index),
                None,
                Some(json_value_kind_name(*actual)),
                None,
            ),
            TilesetCatalogError::MissingField { index, field, .. } => (
                TilesetCatalogErrorCategory::MissingField,
                Some(*index),
                Some(field.to_string()),
                None,
                None,
            ),
            TilesetCatalogError::DuplicateField { index, field, .. } => (
                TilesetCatalogErrorCategory::DuplicateField,
                Some(*index),
                Some(field.to_string()),
                None,
                None,
            ),
            TilesetCatalogError::UnexpectedFieldKind {
                index,
                field,
                actual,
                ..
            } => (
                TilesetCatalogErrorCategory::UnexpectedFieldKind,
                Some(*index),
                Some(field.to_string()),
                Some(json_value_kind_name(*actual)),
                None,
            ),
            TilesetCatalogError::UnsupportedInteger { index, field, .. } => (
                TilesetCatalogErrorCategory::UnsupportedInteger,
                Some(*index),
                Some(field.to_string()),
                None,
                None,
            ),
            TilesetCatalogError::InvalidString { index, field, .. } => (
                TilesetCatalogErrorCategory::InvalidString,
                Some(*index),
                Some(field.to_string()),
                None,
                None,
            ),
            TilesetCatalogError::IndexOutOfRange { index, .. } => (
                TilesetCatalogErrorCategory::IndexOutOfRange,
                Some(*index),
                None,
                None,
                None,
            ),
            TilesetCatalogError::ReservedIndex => (
                TilesetCatalogErrorCategory::ReservedIndex,
                Some(0),
                None,
                None,
                None,
            ),
            TilesetCatalogError::IdIndexMismatch {
                index, decoded_id, ..
            } => (
                TilesetCatalogErrorCategory::IdIndexMismatch,
                Some(*index),
                Some(TilesetField::Id.to_string()),
                None,
                Some(decoded_id.get()),
            ),
            _ => (
                TilesetCatalogErrorCategory::Unrecognized,
                None,
                None,
                None,
                None,
            ),
        };

        Self {
            category,
            message: error.to_string(),
            index,
            field,
            actual_kind,
            decoded_id,
        }
    }
}

impl MapCatalogErrorDetail {
    fn new(error: &MapCatalogError) -> Self {
        let (category, index, field, actual_kind, decoded_id) = match error {
            MapCatalogError::MissingDocument => (
                MapCatalogErrorCategory::MissingDocument,
                None,
                None,
                None,
                None,
            ),
            MapCatalogError::UnavailableDocument => (
                MapCatalogErrorCategory::UnavailableDocument,
                None,
                None,
                None,
                None,
            ),
            MapCatalogError::UnexpectedRootKind { actual, .. } => (
                MapCatalogErrorCategory::UnexpectedRootKind,
                None,
                None,
                Some(json_value_kind_name(*actual)),
                None,
            ),
            MapCatalogError::UnexpectedEntryKind { index, actual, .. } => (
                MapCatalogErrorCategory::UnexpectedEntryKind,
                Some(*index),
                None,
                Some(json_value_kind_name(*actual)),
                None,
            ),
            MapCatalogError::MissingField { index, field, .. } => (
                MapCatalogErrorCategory::MissingField,
                Some(*index),
                Some(map_info_field_name(*field)),
                None,
                None,
            ),
            MapCatalogError::DuplicateField { index, field, .. } => (
                MapCatalogErrorCategory::DuplicateField,
                Some(*index),
                Some(map_info_field_name(*field)),
                None,
                None,
            ),
            MapCatalogError::UnexpectedFieldKind {
                index,
                field,
                actual,
                ..
            } => (
                MapCatalogErrorCategory::UnexpectedFieldKind,
                Some(*index),
                Some(map_info_field_name(*field)),
                Some(json_value_kind_name(*actual)),
                None,
            ),
            MapCatalogError::UnsupportedInteger { index, field, .. } => (
                MapCatalogErrorCategory::UnsupportedInteger,
                Some(*index),
                Some(map_info_field_name(*field)),
                None,
                None,
            ),
            MapCatalogError::InvalidString { index, field, .. } => (
                MapCatalogErrorCategory::InvalidString,
                Some(*index),
                Some(map_info_field_name(*field)),
                None,
                None,
            ),
            MapCatalogError::IndexOutOfRange { index, .. } => (
                MapCatalogErrorCategory::IndexOutOfRange,
                Some(*index),
                None,
                None,
                None,
            ),
            MapCatalogError::IdIndexMismatch {
                index, decoded_id, ..
            } => (
                MapCatalogErrorCategory::IdIndexMismatch,
                Some(*index),
                Some(map_info_field_name(MapInfoField::Id)),
                None,
                Some(decoded_id.get()),
            ),
            _ => (
                MapCatalogErrorCategory::Unrecognized,
                None,
                None,
                None,
                None,
            ),
        };

        Self {
            category,
            message: error.to_string(),
            index,
            field,
            actual_kind,
            decoded_id,
        }
    }
}

fn map_info_field_name(field: MapInfoField) -> String {
    field.to_string()
}

fn json_value_kind_name(kind: JsonValueKind) -> String {
    kind.to_string()
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Discover { path, format } => run_discover(&path, format),
        Command::Inventory { path, format } => run_inventory(&path, format),
        Command::Snapshot {
            path,
            format,
            limits,
        } => run_snapshot(&path, format, limits.limits()),
        Command::Maps {
            path,
            format,
            limits,
        } => run_maps(&path, format, limits.limits()),
        Command::Tilesets {
            path,
            format,
            limits,
        } => run_tilesets(&path, format, limits.limits()),
        Command::Map {
            path,
            id,
            format,
            limits,
        } => run_map(&path, id, format, limits.limits()),
        Command::Events {
            path,
            map_id,
            format,
            limits,
        } => run_events(&path, map_id, format, limits.limits()),
        Command::System {
            path,
            format,
            limits,
        } => run_system(&path, format, limits.limits()),
        Command::Validate {
            path,
            format,
            limits,
        } => run_validate(&path, format, limits.limits()),
        Command::ValidateTilesets {
            path,
            format,
            limits,
        } => run_validate_tilesets(&path, format, limits.limits()),
        Command::InspectJson {
            path,
            format,
            max_bytes,
        } => run_inspect_json(&path, format, max_bytes),
    }
}

fn run_snapshot(path: &Path, format: OutputFormat, limits: SnapshotLimits) -> ExitCode {
    let root_dir = match cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()) {
        Ok(dir) => dir,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: format!("failed to open project root '{}'", path.display()),
                    cause: Some(error.to_string()),
                },
            };
            return write_error_report(&report, format);
        }
    };

    match load_snapshot(&root_dir, limits) {
        Ok(snapshot) => {
            let report = SnapshotReport::new(path, &snapshot, limits);
            let write_result = match format {
                OutputFormat::Human => write_human_snapshot_report(io::stdout().lock(), &report),
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: error.to_string(),
                    cause: error.source().map(ToString::to_string),
                },
            };
            write_error_report(&report, format)
        }
    }
}

fn run_maps(path: &Path, format: OutputFormat, limits: SnapshotLimits) -> ExitCode {
    let root_dir = match cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()) {
        Ok(dir) => dir,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: format!("failed to open project root '{}'", path.display()),
                    cause: Some(error.to_string()),
                },
            };
            return write_error_report(&report, format);
        }
    };

    let snapshot = match load_snapshot(&root_dir, limits) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: error.to_string(),
                    cause: error.source().map(ToString::to_string),
                },
            };
            return write_error_report(&report, format);
        }
    };

    match map_catalog(&snapshot) {
        Ok(catalog) => {
            let report = MapsReport::new(path, &snapshot, &catalog, limits);
            let write_result = match format {
                OutputFormat::Human => write_human_maps_report(io::stdout().lock(), &report),
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let report = MapsErrorReport::new(path, &snapshot, limits, &error);
            write_maps_error_report(&report, format)
        }
    }
}

fn run_tilesets(path: &Path, format: OutputFormat, limits: SnapshotLimits) -> ExitCode {
    let root_dir = match cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()) {
        Ok(dir) => dir,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: format!("failed to open project root '{}'", path.display()),
                    cause: Some(error.to_string()),
                },
            };
            return write_error_report(&report, format);
        }
    };

    let snapshot = match load_snapshot(&root_dir, limits) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: error.to_string(),
                    cause: error.source().map(ToString::to_string),
                },
            };
            return write_error_report(&report, format);
        }
    };

    match tileset_catalog(&snapshot) {
        Ok(catalog) => {
            let report = TilesetsReport::new(path, &snapshot, &catalog, limits);
            let write_result = match format {
                OutputFormat::Human => write_human_tilesets_report(io::stdout().lock(), &report),
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let report = TilesetsErrorReport::new(path, &snapshot, limits, &error);
            write_tilesets_error_report(&report, format)
        }
    }
}

fn run_map(path: &Path, map_id: MapId, format: OutputFormat, limits: SnapshotLimits) -> ExitCode {
    let root_dir = match cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()) {
        Ok(dir) => dir,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: format!("failed to open project root '{}'", path.display()),
                    cause: Some(error.to_string()),
                },
            };
            return write_error_report(&report, format);
        }
    };

    let snapshot = match load_snapshot(&root_dir, limits) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: error.to_string(),
                    cause: error.source().map(ToString::to_string),
                },
            };
            return write_error_report(&report, format);
        }
    };

    match map_summary(&snapshot, map_id) {
        Ok(summary) => {
            let report = SelectedMapReport::new(path, &snapshot, &summary, limits);
            let write_result = match format {
                OutputFormat::Human => {
                    write_human_selected_map_report(io::stdout().lock(), &report)
                }
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let report = SelectedMapErrorReport::new(path, &snapshot, limits, &error);
            write_selected_map_error_report(&report, format)
        }
    }
}

fn run_events(
    path: &Path,
    map_id: MapId,
    format: OutputFormat,
    limits: SnapshotLimits,
) -> ExitCode {
    let root_dir = match cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()) {
        Ok(dir) => dir,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: format!("failed to open project root '{}'", path.display()),
                    cause: Some(error.to_string()),
                },
            };
            return write_error_report(&report, format);
        }
    };

    let snapshot = match load_snapshot(&root_dir, limits) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: error.to_string(),
                    cause: error.source().map(ToString::to_string),
                },
            };
            return write_error_report(&report, format);
        }
    };

    match map_event_catalog(&snapshot, map_id) {
        Ok(catalog) => {
            let report = MapEventsReport::new(path, &snapshot, &catalog, limits);
            let write_result = match format {
                OutputFormat::Human => write_human_map_events_report(io::stdout().lock(), &report),
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let report = MapEventsErrorReport::new(path, &snapshot, limits, &error);
            write_map_events_error_report(&report, format)
        }
    }
}

fn run_system(path: &Path, format: OutputFormat, limits: SnapshotLimits) -> ExitCode {
    let root_dir = match cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()) {
        Ok(dir) => dir,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: format!("failed to open project root '{}'", path.display()),
                    cause: Some(error.to_string()),
                },
            };
            return write_error_report(&report, format);
        }
    };

    let snapshot = match load_snapshot(&root_dir, limits) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: error.to_string(),
                    cause: error.source().map(ToString::to_string),
                },
            };
            return write_error_report(&report, format);
        }
    };

    match system_summary(&snapshot) {
        Ok(summary) => {
            let report = SystemReport::new(path, &snapshot, &summary, limits);
            let write_result = match format {
                OutputFormat::Human => write_human_system_report(io::stdout().lock(), &report),
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let report = SystemErrorReport::new(path, &snapshot, limits, &error);
            write_system_error_report(&report, format)
        }
    }
}

fn run_validate(path: &Path, format: OutputFormat, limits: SnapshotLimits) -> ExitCode {
    let root_dir = match cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()) {
        Ok(dir) => dir,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: format!("failed to open project root '{}'", path.display()),
                    cause: Some(error.to_string()),
                },
            };
            return write_error_report(&report, format);
        }
    };

    let snapshot = match load_snapshot(&root_dir, limits) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: error.to_string(),
                    cause: error.source().map(ToString::to_string),
                },
            };
            return write_error_report(&report, format);
        }
    };

    match validate_player_start(&snapshot) {
        Ok(validation) => {
            let report = ValidationReport::new(path, &snapshot, &validation, limits);
            let write_result = match format {
                OutputFormat::Human => write_human_validation_report(io::stdout().lock(), &report),
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let report = ValidationErrorReport::new(path, &snapshot, limits, &error);
            write_validation_error_report(&report, format)
        }
    }
}

fn run_validate_tilesets(path: &Path, format: OutputFormat, limits: SnapshotLimits) -> ExitCode {
    let root_dir = match cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()) {
        Ok(dir) => dir,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: format!("failed to open project root '{}'", path.display()),
                    cause: Some(error.to_string()),
                },
            };
            return write_error_report(&report, format);
        }
    };

    let snapshot = match load_snapshot(&root_dir, limits) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: error.to_string(),
                    cause: error.source().map(ToString::to_string),
                },
            };
            return write_error_report(&report, format);
        }
    };

    match validate_map_tilesets(&snapshot) {
        Ok(validation) => {
            let report = MapTilesetValidationReport::new(path, &snapshot, &validation, limits);
            let write_result = match format {
                OutputFormat::Human => {
                    write_human_map_tileset_validation_report(io::stdout().lock(), &report)
                }
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let report = MapTilesetValidationErrorReport::new(path, &snapshot, limits, &error);
            write_map_tileset_validation_error_report(&report, format)
        }
    }
}

fn run_discover(path: &Path, format: OutputFormat) -> ExitCode {
    match discover_candidate(path) {
        Ok(discovery) => {
            let report = DiscoveryReport::new(path, &discovery);
            let write_result = match format {
                OutputFormat::Human => write_human_report(io::stdout().lock(), &report),
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: error.to_string(),
                    cause: error.source().map(ToString::to_string),
                },
            };
            write_error_report(&report, format)
        }
    }
}

fn run_inventory(path: &Path, format: OutputFormat) -> ExitCode {
    let root_dir = match cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()) {
        Ok(dir) => dir,
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: format!("failed to open project root '{}'", path.display()),
                    cause: Some(error.to_string()),
                },
            };
            return write_error_report(&report, format);
        }
    };

    match inventory_project(&root_dir) {
        Ok(inventory) => {
            let report = InventoryReport::new(path, &inventory);

            if matches!(format, OutputFormat::Json)
                && (report.root.utf8.is_none()
                    || report.entries.iter().any(|e| e.path.utf8.is_none()))
            {
                let error_report = ErrorReport {
                    schema_version: OUTPUT_SCHEMA_VERSION,
                    root: PathReport::new(path),
                    error: ErrorDetail {
                        message: "JSON output cannot safely represent non-UTF-8 paths without lossy conversion".to_string(),
                        cause: None,
                    },
                };
                return write_error_report(&error_report, format);
            }

            let write_result = match format {
                OutputFormat::Human => write_human_inventory_report(io::stdout().lock(), &report),
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let report = ErrorReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                root: PathReport::new(path),
                error: ErrorDetail {
                    message: error.to_string(),
                    cause: error.source().map(ToString::to_string),
                },
            };
            write_error_report(&report, format)
        }
    }
}

fn run_inspect_json(path: &Path, format: OutputFormat, max_bytes: usize) -> ExitCode {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            return write_inspect_json_error(
                path,
                format,
                InspectJsonErrorCategory::IoError,
                e.to_string(),
                None,
            );
        }
    };

    // Read at most max_bytes + 1
    let mut buffer = Vec::new();
    let limit = max_bytes + 1;
    if let Err(e) = file.take(limit as u64).read_to_end(&mut buffer) {
        return write_inspect_json_error(
            path,
            format,
            InspectJsonErrorCategory::IoError,
            e.to_string(),
            None,
        );
    }

    if buffer.len() > max_bytes {
        return write_inspect_json_error(
            path,
            format,
            InspectJsonErrorCategory::TooLarge,
            format!("file exceeds maximum size of {} bytes", max_bytes),
            None,
        );
    }

    match LosslessJsonDocument::parse(&buffer) {
        Ok(document) => {
            let byte_identical = document.to_string().as_bytes() == buffer;
            if !byte_identical {
                return write_inspect_json_error(
                    path,
                    format,
                    InspectJsonErrorCategory::ByteIdentityMismatch,
                    "document parsed successfully but failed byte-identity invariant".to_string(),
                    None,
                );
            }

            let report = InspectJsonReport {
                schema_version: OUTPUT_SCHEMA_VERSION,
                path: PathReport::new(path),
                byte_length: buffer.len(),
                strict_syntax_accepted: true,
                byte_identical: true,
            };

            let write_result = match format {
                OutputFormat::Human => {
                    write_human_inspect_json_report(io::stdout().lock(), &report)
                }
                OutputFormat::Json => write_json(io::stdout().lock(), &report),
            };
            finish_write(write_result)
        }
        Err(error) => {
            let (category, message, byte_range) = match error {
                LosslessJsonError::InvalidUtf8 { source, .. } => (
                    InspectJsonErrorCategory::InvalidUtf8,
                    format!("JSON input is not valid UTF-8: {}", source),
                    None,
                ),
                LosslessJsonError::Utf8ByteOrderMark => (
                    InspectJsonErrorCategory::Utf8Bom,
                    "JSON input starts with a UTF-8 byte-order mark".to_string(),
                    None,
                ),
                LosslessJsonError::InvalidSyntax { diagnostic, .. } => (
                    InspectJsonErrorCategory::InvalidSyntax,
                    diagnostic.message().to_string(),
                    Some(RangeReport {
                        start: diagnostic.byte_range().start,
                        end: diagnostic.byte_range().end,
                    }),
                ),
                _ => (
                    InspectJsonErrorCategory::Unrecognized,
                    error.to_string(),
                    None,
                ),
            };

            write_inspect_json_error(path, format, category, message, byte_range)
        }
    }
}

fn write_inspect_json_error(
    path: &Path,
    format: OutputFormat,
    category: InspectJsonErrorCategory,
    message: String,
    byte_range: Option<RangeReport>,
) -> ExitCode {
    let report = InspectJsonErrorReport {
        schema_version: OUTPUT_SCHEMA_VERSION,
        path: PathReport::new(path),
        error: InspectJsonErrorDetail {
            category,
            message,
            byte_range,
        },
    };

    let write_result = match format {
        OutputFormat::Human => write_human_inspect_json_error(io::stderr().lock(), &report),
        OutputFormat::Json => write_json(io::stdout().lock(), &report),
    };

    match write_result {
        Ok(()) => ExitCode::from(1),
        Err(write_error) => report_write_error(write_error),
    }
}

fn write_error_report(report: &ErrorReport, format: OutputFormat) -> ExitCode {
    let write_result = match format {
        OutputFormat::Human => write_human_error(io::stderr().lock(), report),
        OutputFormat::Json => write_json(io::stdout().lock(), report),
    };

    match write_result {
        Ok(()) => ExitCode::from(1),
        Err(write_error) => report_write_error(write_error),
    }
}

fn write_maps_error_report(report: &MapsErrorReport, format: OutputFormat) -> ExitCode {
    let write_result = match format {
        OutputFormat::Human => write_human_maps_error(io::stderr().lock(), report),
        OutputFormat::Json => write_json(io::stdout().lock(), report),
    };

    match write_result {
        Ok(()) => ExitCode::from(1),
        Err(write_error) => report_write_error(write_error),
    }
}

fn write_tilesets_error_report(report: &TilesetsErrorReport, format: OutputFormat) -> ExitCode {
    let write_result = match format {
        OutputFormat::Human => write_human_tilesets_error(io::stderr().lock(), report),
        OutputFormat::Json => write_json(io::stdout().lock(), report),
    };

    match write_result {
        Ok(()) => ExitCode::from(1),
        Err(write_error) => report_write_error(write_error),
    }
}

fn write_selected_map_error_report(
    report: &SelectedMapErrorReport,
    format: OutputFormat,
) -> ExitCode {
    let write_result = match format {
        OutputFormat::Human => write_human_selected_map_error(io::stderr().lock(), report),
        OutputFormat::Json => write_json(io::stdout().lock(), report),
    };

    match write_result {
        Ok(()) => ExitCode::from(1),
        Err(write_error) => report_write_error(write_error),
    }
}

fn write_map_events_error_report(report: &MapEventsErrorReport, format: OutputFormat) -> ExitCode {
    let write_result = match format {
        OutputFormat::Human => write_human_map_events_error(io::stderr().lock(), report),
        OutputFormat::Json => write_json(io::stdout().lock(), report),
    };

    match write_result {
        Ok(()) => ExitCode::from(1),
        Err(write_error) => report_write_error(write_error),
    }
}

fn write_system_error_report(report: &SystemErrorReport, format: OutputFormat) -> ExitCode {
    let write_result = match format {
        OutputFormat::Human => write_human_system_error(io::stderr().lock(), report),
        OutputFormat::Json => write_json(io::stdout().lock(), report),
    };

    match write_result {
        Ok(()) => ExitCode::from(1),
        Err(write_error) => report_write_error(write_error),
    }
}

fn write_validation_error_report(report: &ValidationErrorReport, format: OutputFormat) -> ExitCode {
    let write_result = match format {
        OutputFormat::Human => write_human_validation_error(io::stderr().lock(), report),
        OutputFormat::Json => write_json(io::stdout().lock(), report),
    };

    match write_result {
        Ok(()) => ExitCode::from(1),
        Err(write_error) => report_write_error(write_error),
    }
}

fn write_map_tileset_validation_error_report(
    report: &MapTilesetValidationErrorReport,
    format: OutputFormat,
) -> ExitCode {
    let write_result = match format {
        OutputFormat::Human => {
            write_human_map_tileset_validation_error(io::stderr().lock(), report)
        }
        OutputFormat::Json => write_json(io::stdout().lock(), report),
    };

    match write_result {
        Ok(()) => ExitCode::from(1),
        Err(write_error) => report_write_error(write_error),
    }
}

fn escape_controls(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_control() {
                c.escape_debug().to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}

fn write_human_report(mut writer: impl Write, report: &DiscoveryReport) -> io::Result<()> {
    match report.result {
        DiscoveryResult::Candidate => {
            write_expected_marker(&mut writer, "RPG Maker MZ candidate", &report.markers)
        }
        DiscoveryResult::CaseVariantCandidate => write_expected_marker(
            &mut writer,
            "Case-variant RPG Maker MZ candidate",
            &report.markers,
        ),
        DiscoveryResult::NoMarker => writeln!(
            writer,
            "No RPG Maker MZ marker found in {}",
            escape_controls(&report.root.display)
        ),
        DiscoveryResult::AmbiguousMarkers => {
            writeln!(
                writer,
                "Ambiguous RPG Maker MZ markers in {}:",
                escape_controls(&report.root.display)
            )?;
            for marker in &report.markers {
                writeln!(
                    writer,
                    "  - {} ({})",
                    escape_controls(&marker.path.display),
                    marker.kind.name()
                )?;
            }
            Ok(())
        }
        DiscoveryResult::SymlinkMarker => write_expected_marker(
            &mut writer,
            "RPG Maker MZ marker is a symlink",
            &report.markers,
        ),
        DiscoveryResult::NonRegularMarker => write_expected_marker(
            &mut writer,
            "RPG Maker MZ marker is not a regular file",
            &report.markers,
        ),
        DiscoveryResult::Unrecognized => writeln!(
            writer,
            "Discovery returned an unrecognized experimental result for {}",
            escape_controls(&report.root.display)
        ),
    }
}

fn write_human_inventory_report(
    mut writer: impl Write,
    report: &InventoryReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "Inventory for {}:",
        escape_controls(&report.root.display)
    )?;
    if report.entries.is_empty() {
        writeln!(writer, "  (empty)")?;
        return Ok(());
    }

    for entry in &report.entries {
        let kind_str = match entry.kind {
            EntryKindReport::File => "file",
            EntryKindReport::Directory => "dir",
            EntryKindReport::Symlink => "symlink",
            EntryKindReport::Other => "other",
            EntryKindReport::Unrecognized => "unrecognized",
        };

        let class_str = match &entry.classification {
            ClassificationReport::Known { family } => match family {
                KnownFamilyReport::StandardRootEntry => "known (standard root entry)",
                KnownFamilyReport::StandardDataFile => "known (standard data file)",
                KnownFamilyReport::MapDataFile => "known (map data file)",
                KnownFamilyReport::Unrecognized => "known (unrecognized family)",
            },
            ClassificationReport::ExtensionCandidate { family } => match family {
                ExtensionFamilyReport::DataJson => "extension candidate (data json)",
                ExtensionFamilyReport::Unrecognized => "extension candidate (unrecognized family)",
            },
            ClassificationReport::Unknown => "unknown",
            ClassificationReport::Unrecognized => "unrecognized classification",
        };

        writeln!(
            writer,
            "  - {} [{}] ({})",
            escape_controls(&entry.path.display),
            kind_str,
            class_str
        )?;
    }
    Ok(())
}

fn write_human_snapshot_report(mut writer: impl Write, report: &SnapshotReport) -> io::Result<()> {
    writeln!(
        writer,
        "Snapshot for {}:",
        escape_controls(&report.root.display)
    )?;
    writeln!(writer, "  Completeness: {}", report.completeness.name())?;
    writeln!(
        writer,
        "  Inventory entries: {}",
        report.inventory_entry_count
    )?;
    writeln!(
        writer,
        "  Loaded documents: {}",
        report.loaded_document_count
    )?;
    writeln!(writer, "  Diagnostics: {}", report.diagnostic_count)?;
    writeln!(
        writer,
        "  Limits: {} documents, {} bytes/document, {} aggregate bytes",
        report.limits.max_documents,
        report.limits.max_bytes_per_document,
        report.limits.max_aggregate_bytes
    )?;

    if !report.documents.is_empty() {
        writeln!(writer, "Documents:")?;
        for document in &report.documents {
            writeln!(
                writer,
                "  - {} ({} bytes)",
                escape_controls(&document.path.display),
                document.byte_length
            )?;
        }
    }

    write_human_snapshot_diagnostics(&mut writer, "Diagnostics:", &report.diagnostics)?;

    Ok(())
}

fn write_human_maps_report(mut writer: impl Write, report: &MapsReport) -> io::Result<()> {
    writeln!(
        writer,
        "Maps for {}:",
        escape_controls(&report.root.display)
    )?;
    writeln!(
        writer,
        "  Snapshot completeness: {}",
        report.snapshot_completeness.name()
    )?;
    writeln!(writer, "  Maps: {}", report.map_count)?;
    writeln!(writer, "  Findings: {}", report.finding_count)?;
    writeln!(
        writer,
        "  Snapshot diagnostics: {}",
        report.snapshot_diagnostic_count
    )?;
    writeln!(
        writer,
        "  Limits: {} documents, {} bytes/document, {} aggregate bytes",
        report.limits.max_documents,
        report.limits.max_bytes_per_document,
        report.limits.max_aggregate_bytes
    )?;

    if report.maps.is_empty() {
        writeln!(writer, "  (no maps)")?;
    } else {
        writeln!(writer, "Map catalog:")?;
        for map in &report.maps {
            let parent = map
                .parent_id
                .map_or_else(|| "none".to_string(), |id| id.to_string());
            writeln!(
                writer,
                "  - {}: {} (order {}, parent {})",
                map.id,
                escape_controls(&map.name),
                map.order,
                parent
            )?;
        }
    }

    if !report.findings.is_empty() {
        writeln!(writer, "Map catalog findings:")?;
        for finding in &report.findings {
            write_human_map_finding(&mut writer, finding)?;
        }
    }

    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_tilesets_report(mut writer: impl Write, report: &TilesetsReport) -> io::Result<()> {
    writeln!(
        writer,
        "Tilesets for {}:",
        escape_controls(&report.root.display)
    )?;
    writeln!(writer, "  Tilesets: {}", report.tileset_count)?;
    writeln!(
        writer,
        "  Snapshot completeness: {}",
        report.snapshot_completeness.name()
    )?;
    writeln!(
        writer,
        "  Snapshot diagnostics: {}",
        report.snapshot_diagnostic_count
    )?;
    writeln!(
        writer,
        "  Limits: {} documents, {} bytes/document, {} aggregate bytes",
        report.limits.max_documents,
        report.limits.max_bytes_per_document,
        report.limits.max_aggregate_bytes
    )?;

    if report.tilesets.is_empty() {
        writeln!(writer, "  (no tilesets)")?;
    } else {
        writeln!(writer, "Tileset catalog:")?;
        for tileset in &report.tilesets {
            writeln!(
                writer,
                "  - {}: {}",
                tileset.id,
                escape_controls(&tileset.name)
            )?;
        }
    }

    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_selected_map_report(
    mut writer: impl Write,
    report: &SelectedMapReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "Map {} for {}:",
        report.map.id,
        escape_controls(&report.root.display)
    )?;
    writeln!(
        writer,
        "  Catalog name: {}",
        escape_controls(&report.map.catalog_name)
    )?;
    writeln!(
        writer,
        "  Display name: {}",
        escape_controls(&report.map.display_name)
    )?;
    writeln!(
        writer,
        "  Document: {}",
        escape_controls(&report.map.document_path.display)
    )?;
    writeln!(
        writer,
        "  Size: {} x {}",
        report.map.width, report.map.height
    )?;
    writeln!(writer, "  Tileset ID: {}", report.map.tileset_id)?;
    writeln!(writer, "  Opaque event objects: {}", report.map.event_count)?;
    writeln!(
        writer,
        "  Snapshot completeness: {}",
        report.snapshot_completeness.name()
    )?;
    writeln!(
        writer,
        "  Snapshot diagnostics: {}",
        report.snapshot_diagnostic_count
    )?;
    writeln!(
        writer,
        "  Limits: {} documents, {} bytes/document, {} aggregate bytes",
        report.limits.max_documents,
        report.limits.max_bytes_per_document,
        report.limits.max_aggregate_bytes
    )?;
    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_map_events_report(
    mut writer: impl Write,
    report: &MapEventsReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "Events on map {} ({}) for {}:",
        report.map.id,
        escape_controls(&report.map.catalog_name),
        escape_controls(&report.root.display)
    )?;
    writeln!(
        writer,
        "  Document: {}",
        escape_controls(&report.map.document_path.display)
    )?;
    writeln!(
        writer,
        "  Map size: {} x {}",
        report.map.width, report.map.height
    )?;
    writeln!(writer, "  Events: {}", report.event_count)?;
    writeln!(writer, "  Findings: {}", report.finding_count)?;
    writeln!(
        writer,
        "  Snapshot completeness: {}",
        report.snapshot_completeness.name()
    )?;
    writeln!(
        writer,
        "  Snapshot diagnostics: {}",
        report.snapshot_diagnostic_count
    )?;
    writeln!(
        writer,
        "  Limits: {} documents, {} bytes/document, {} aggregate bytes",
        report.limits.max_documents,
        report.limits.max_bytes_per_document,
        report.limits.max_aggregate_bytes
    )?;

    if report.events.is_empty() {
        writeln!(writer, "  (no events)")?;
    } else {
        writeln!(writer, "Event catalog:")?;
        for event in &report.events {
            writeln!(
                writer,
                "  - {}: {} at ({}, {}) ({} {})",
                event.id,
                escape_controls(&event.name),
                event.x,
                event.y,
                event.page_count,
                if event.page_count == 1 {
                    "page"
                } else {
                    "pages"
                }
            )?;
        }
    }

    if !report.findings.is_empty() {
        writeln!(writer, "Event findings:")?;
        for finding in &report.findings {
            match finding {
                MapEventFindingReport::CoordinatesOutsideMap {
                    event_id,
                    x,
                    y,
                    width,
                    height,
                } => writeln!(
                    writer,
                    "  - event {event_id} coordinate ({x}, {y}) is outside map size {width} x {height}"
                )?,
                MapEventFindingReport::Unrecognized => {
                    writeln!(writer, "  - unrecognized event finding")?
                }
            }
        }
    }

    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_system_report(mut writer: impl Write, report: &SystemReport) -> io::Result<()> {
    writeln!(
        writer,
        "System summary for {}:",
        escape_controls(&report.root.display)
    )?;
    writeln!(
        writer,
        "  Game title: {}",
        escape_controls(&report.system.game_title)
    )?;
    writeln!(
        writer,
        "  Currency unit: {}",
        escape_controls(&report.system.currency_unit)
    )?;
    writeln!(
        writer,
        "  Locale: {}",
        escape_controls(&report.system.locale)
    )?;
    writeln!(writer, "  Editor map ID: {}", report.system.edit_map_id)?;
    writeln!(
        writer,
        "  Player start: map {} at ({}, {})",
        report.system.start_map_id, report.system.start_x, report.system.start_y
    )?;
    writeln!(
        writer,
        "  Document: {}",
        escape_controls(&report.system.document_path.display)
    )?;
    writeln!(
        writer,
        "  Snapshot completeness: {}",
        report.snapshot_completeness.name()
    )?;
    writeln!(
        writer,
        "  Snapshot diagnostics: {}",
        report.snapshot_diagnostic_count
    )?;
    writeln!(
        writer,
        "  Limits: {} documents, {} bytes/document, {} aggregate bytes",
        report.limits.max_documents,
        report.limits.max_bytes_per_document,
        report.limits.max_aggregate_bytes
    )?;
    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_validation_report(
    mut writer: impl Write,
    report: &ValidationReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "Validation for {}:",
        escape_controls(&report.root.display)
    )?;
    writeln!(writer, "  Scope: player start")?;
    writeln!(
        writer,
        "  Player start: map {} at ({}, {})",
        report.validation.start_map_id, report.validation.start_x, report.validation.start_y
    )?;
    writeln!(writer, "  Findings: {}", report.validation.finding_count)?;
    writeln!(
        writer,
        "  Snapshot completeness: {}",
        report.snapshot_completeness.name()
    )?;
    writeln!(
        writer,
        "  Snapshot diagnostics: {}",
        report.snapshot_diagnostic_count
    )?;
    writeln!(
        writer,
        "  Limits: {} documents, {} bytes/document, {} aggregate bytes",
        report.limits.max_documents,
        report.limits.max_bytes_per_document,
        report.limits.max_aggregate_bytes
    )?;

    if report.validation.findings.is_empty() {
        writeln!(writer, "  No player-start findings.")?;
    } else {
        writeln!(writer, "Player-start findings:")?;
        for finding in &report.validation.findings {
            write_human_player_start_finding(&mut writer, finding)?;
        }
    }

    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_map_tileset_validation_report(
    mut writer: impl Write,
    report: &MapTilesetValidationReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "Map-to-tileset validation for {}:",
        escape_controls(&report.root.display)
    )?;
    writeln!(writer, "  Maps checked: {}", report.validation.map_count)?;
    writeln!(writer, "  Findings: {}", report.validation.finding_count)?;
    writeln!(
        writer,
        "  Snapshot completeness: {}",
        report.snapshot_completeness.name()
    )?;
    writeln!(
        writer,
        "  Snapshot diagnostics: {}",
        report.snapshot_diagnostic_count
    )?;
    writeln!(
        writer,
        "  Limits: {} documents, {} bytes/document, {} aggregate bytes",
        report.limits.max_documents,
        report.limits.max_bytes_per_document,
        report.limits.max_aggregate_bytes
    )?;

    if report.validation.findings.is_empty() {
        writeln!(writer, "  No map-to-tileset findings.")?;
    } else {
        writeln!(writer, "Map-to-tileset findings:")?;
        for finding in &report.validation.findings {
            match finding {
                MapTilesetFindingReport::MissingTileset { map_id, tileset_id } => writeln!(
                    writer,
                    "  - map {map_id} refers to missing tileset {tileset_id}"
                )?,
                MapTilesetFindingReport::Unrecognized => writeln!(
                    writer,
                    "  - unrecognized experimental map-to-tileset finding"
                )?,
            }
        }
    }

    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_player_start_finding(
    writer: &mut impl Write,
    finding: &PlayerStartFindingReport,
) -> io::Result<()> {
    match finding {
        PlayerStartFindingReport::MissingPlayerStart => writeln!(
            writer,
            "  - player starting position is unset; RPG Maker MZ cannot start the game without it"
        ),
        PlayerStartFindingReport::ZeroMapIdWithCoordinates { start_x, start_y } => writeln!(
            writer,
            "  - map ID 0 has unevidenced coordinates ({start_x}, {start_y})"
        ),
        PlayerStartFindingReport::MissingMapRecord { map_id } => {
            writeln!(writer, "  - map {map_id} has no map-catalog record")
        }
        PlayerStartFindingReport::OutOfBounds {
            map_id,
            start_x,
            start_y,
            width,
            height,
        } => writeln!(
            writer,
            "  - ({start_x}, {start_y}) is outside map {map_id} dimensions {width} x {height}"
        ),
        PlayerStartFindingReport::Unrecognized => {
            writeln!(writer, "  - unrecognized experimental player-start finding")
        }
    }
}

fn write_human_map_finding(writer: &mut impl Write, finding: &MapFindingReport) -> io::Result<()> {
    match finding {
        MapFindingReport::MissingParent { map_id, parent_id } => {
            writeln!(
                writer,
                "  - map {map_id} refers to missing parent {parent_id}"
            )
        }
        MapFindingReport::ParentCycle { map_ids } => {
            writeln!(
                writer,
                "  - parent cycle among maps {}",
                join_map_ids(map_ids)
            )
        }
        MapFindingReport::DuplicateOrder { order, map_ids } => writeln!(
            writer,
            "  - display order {order} is shared by maps {}",
            join_map_ids(map_ids)
        ),
        MapFindingReport::MissingMapDocument {
            map_id,
            expected_path,
        } => writeln!(
            writer,
            "  - map {map_id} has no entry at {}",
            escape_controls(&expected_path.display)
        ),
        MapFindingReport::OrphanMapDocument { map_id, path } => writeln!(
            writer,
            "  - {} encodes map {map_id} without a catalog record",
            escape_controls(&path.display)
        ),
        MapFindingReport::UnrecognizedMapDocumentIdentity { path } => writeln!(
            writer,
            "  - {} does not encode an evidenced positive map ID",
            escape_controls(&path.display)
        ),
        MapFindingReport::UnevidencedMapDocumentPath { map_id } => writeln!(
            writer,
            "  - map {map_id} is outside the evidenced three-digit filename relationship"
        ),
        MapFindingReport::Unrecognized => {
            writeln!(writer, "  - unrecognized experimental map-catalog finding")
        }
    }
}

fn join_map_ids(map_ids: &[u32]) -> String {
    map_ids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn write_human_snapshot_diagnostics(
    writer: &mut impl Write,
    heading: &str,
    diagnostics: &[SnapshotDiagnosticReport],
) -> io::Result<()> {
    if diagnostics.is_empty() {
        return Ok(());
    }

    writeln!(writer, "{heading}")?;
    for diagnostic in diagnostics {
        writeln!(
            writer,
            "  - {} [{}]: {}",
            escape_controls(&diagnostic.path.display),
            diagnostic.category.name(),
            escape_controls(&diagnostic.message)
        )?;
        if let Some(cause) = &diagnostic.cause {
            writeln!(writer, "    caused by: {}", escape_controls(cause))?;
        }
        if let Some(range) = &diagnostic.byte_range {
            writeln!(writer, "    at bytes {}..{}", range.start, range.end)?;
        }
    }
    Ok(())
}

fn write_human_maps_error(mut writer: impl Write, report: &MapsErrorReport) -> io::Result<()> {
    writeln!(
        writer,
        "error: {} for {}",
        escape_controls(&report.error.message),
        escape_controls(&report.root.display)
    )?;
    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_tilesets_error(
    mut writer: impl Write,
    report: &TilesetsErrorReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "error: {} for {}",
        escape_controls(&report.error.message),
        escape_controls(&report.root.display)
    )?;
    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_selected_map_error(
    mut writer: impl Write,
    report: &SelectedMapErrorReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "error: {} for {}",
        escape_controls(&report.error.message),
        escape_controls(&report.root.display)
    )?;
    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_map_events_error(
    mut writer: impl Write,
    report: &MapEventsErrorReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "error: {} for {}",
        escape_controls(&report.error.message),
        escape_controls(&report.root.display)
    )?;
    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_system_error(mut writer: impl Write, report: &SystemErrorReport) -> io::Result<()> {
    writeln!(
        writer,
        "error: {} for {}",
        escape_controls(&report.error.message),
        escape_controls(&report.root.display)
    )?;
    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_validation_error(
    mut writer: impl Write,
    report: &ValidationErrorReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "error: {} for {}",
        escape_controls(&report.error.message),
        escape_controls(&report.root.display)
    )?;
    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_map_tileset_validation_error(
    mut writer: impl Write,
    report: &MapTilesetValidationErrorReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "error: {} for {}",
        escape_controls(&report.error.message),
        escape_controls(&report.root.display)
    )?;
    write_human_snapshot_diagnostics(
        &mut writer,
        "Snapshot diagnostics:",
        &report.snapshot_diagnostics,
    )
}

fn write_human_inspect_json_report(
    mut writer: impl Write,
    report: &InspectJsonReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "Strict JSON syntax accepted for {}",
        escape_controls(&report.path.display)
    )?;
    writeln!(writer, "  Byte length: {}", report.byte_length)?;
    writeln!(
        writer,
        "  Byte-identical serialization: {}",
        report.byte_identical
    )?;
    Ok(())
}

impl SnapshotCompletenessReport {
    fn new(completeness: SnapshotCompleteness) -> Self {
        match completeness {
            SnapshotCompleteness::Complete => Self::Complete,
            SnapshotCompleteness::Partial => Self::Partial,
            _ => Self::Unrecognized,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Unrecognized => "unrecognized",
        }
    }
}

impl SnapshotLimitsReport {
    fn new(limits: SnapshotLimits) -> Self {
        Self {
            max_documents: limits.max_documents.get(),
            max_bytes_per_document: limits.max_bytes_per_document.get(),
            max_aggregate_bytes: limits.max_aggregate_bytes.get(),
        }
    }
}

impl SnapshotDiagnosticCategory {
    fn name(&self) -> &'static str {
        match self {
            Self::UnsupportedEntryKind => "unsupported_entry_kind",
            Self::OpenError => "open_error",
            Self::ReadError => "read_error",
            Self::ExceedsDocumentByteLimit => "exceeds_document_byte_limit",
            Self::ExceedsAggregateByteLimit => "exceeds_aggregate_byte_limit",
            Self::ExceedsDocumentCountLimit => "exceeds_document_count_limit",
            Self::InvalidUtf8 => "invalid_utf8",
            Self::Utf8Bom => "utf8_bom",
            Self::InvalidSyntax => "invalid_syntax",
            Self::Unrecognized => "unrecognized",
        }
    }
}

fn write_human_inspect_json_error(
    mut writer: impl Write,
    report: &InspectJsonErrorReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "error: {} for {}",
        escape_controls(&report.error.message),
        escape_controls(&report.path.display)
    )?;
    if let Some(range) = &report.error.byte_range {
        writeln!(writer, "  at bytes {}..{}", range.start, range.end)?;
    }
    Ok(())
}

fn write_expected_marker(
    writer: &mut impl Write,
    label: &str,
    markers: &[MarkerReport],
) -> io::Result<()> {
    let marker = markers
        .first()
        .ok_or_else(|| io::Error::other("discovery result omitted its expected marker"))?;
    write_single_marker(writer, label, marker)
}

fn write_single_marker(
    writer: &mut impl Write,
    label: &str,
    marker: &MarkerReport,
) -> io::Result<()> {
    writeln!(
        writer,
        "{label}: {} ({})",
        escape_controls(&marker.path.display),
        marker.kind.name()
    )
}

fn write_human_error(mut writer: impl Write, report: &ErrorReport) -> io::Result<()> {
    writeln!(writer, "error: {}", escape_controls(&report.error.message))?;
    if let Some(cause) = &report.error.cause {
        writeln!(writer, "caused by: {}", escape_controls(cause))?;
    }
    Ok(())
}

fn write_json(mut writer: impl Write, value: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut writer, value).map_err(io::Error::other)?;
    writeln!(writer)
}

fn finish_write(result: io::Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => report_write_error(error),
    }
}

fn report_write_error(error: io::Error) -> ExitCode {
    let _ = writeln!(
        io::stderr().lock(),
        "error: failed to write output: {error}"
    );
    ExitCode::from(1)
}

impl MarkerKind {
    fn name(&self) -> &'static str {
        match self {
            Self::RegularFile => "regular file",
            Self::Symlink => "symlink",
            Self::Directory => "directory",
            Self::OtherNonRegular => "other non-regular entry",
            Self::Unrecognized => "unrecognized entry kind",
        }
    }
}
