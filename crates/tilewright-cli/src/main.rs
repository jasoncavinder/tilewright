// SPDX-License-Identifier: MPL-2.0

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
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
    /// Inspect a file for strict lossless JSON syntax.
    InspectJson {
        /// File to inspect.
        path: PathBuf,
        /// Output intended for a person or a script.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        /// Maximum bytes to read.
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

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Discover { path, format } => run_discover(&path, format),
        Command::Inventory { path, format } => run_inventory(&path, format),
        Command::InspectJson {
            path,
            format,
            max_bytes,
        } => run_inspect_json(&path, format, max_bytes),
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
