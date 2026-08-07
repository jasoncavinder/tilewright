// SPDX-License-Identifier: MPL-2.0

//! Strict, lossless JSON syntax representation.
//!
//! This module implements the immutable raw-document foundation selected by
//! ADR 0004. It preserves accepted source bytes exactly while keeping the CST
//! backend private. It does not interpret RPG Maker data, perform filesystem
//! I/O, expose arbitrary DOM mutation, or establish a project compatibility
//! claim.

use jsonc_parser::cst::CstRootNode;
use jsonc_parser::errors::ParseError;
use jsonc_parser::{CollectOptions, ParseOptions, parse_to_ast};
use std::error::Error;
use std::fmt;
use std::ops::Range;
use std::str::Utf8Error;

/// An immutable strict-JSON document that preserves its accepted source bytes.
///
/// Parsing validates syntax without converting numeric lexemes to Rust numeric
/// values. The concrete CST implementation is intentionally private so typed
/// domain views can evolve without exposing a third-party JSON DOM as public
/// API.
///
/// The current experimental input contract accepts valid UTF-8 strict JSON
/// without a leading UTF-8 byte-order mark. Comments and JSON extensions are
/// rejected. Duplicate object names are retained rather than collapsed.
/// Ownership and cross-thread behavior are not yet stable API guarantees.
pub struct LosslessJsonDocument {
    source: Box<str>,
    root: CstRootNode,
}

impl LosslessJsonDocument {
    /// Parses one immutable strict-JSON document from caller-supplied bytes.
    ///
    /// This operation performs no filesystem I/O and does not infer anything
    /// about the document's domain meaning or RPG Maker compatibility.
    ///
    /// # Errors
    ///
    /// Returns [`LosslessJsonError`] when the bytes are not UTF-8, begin with a
    /// UTF-8 byte-order mark, or do not contain exactly one strict JSON value.
    pub fn parse(bytes: &[u8]) -> Result<Self, LosslessJsonError> {
        if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
            return Err(LosslessJsonError::Utf8ByteOrderMark);
        }

        let source = std::str::from_utf8(bytes)
            .map_err(|source| LosslessJsonError::InvalidUtf8 { source })?;
        validate_lexical_domain(source)?;
        validate_strict_syntax(source)?;

        let root = CstRootNode::parse(source, &strict_options()).map_err(|error| {
            LosslessJsonError::InvalidSyntax {
                diagnostic: parser_diagnostic(error),
            }
        })?;

        Ok(Self {
            source: source.into(),
            root,
        })
    }

    /// Returns the exact accepted source text without allocating.
    pub fn source_text(&self) -> &str {
        &self.source
    }

    /// Returns the exact accepted source bytes without allocating.
    pub fn source_bytes(&self) -> &[u8] {
        self.source.as_bytes()
    }

    pub(crate) fn cst_root(&self) -> &CstRootNode {
        &self.root
    }
}

impl fmt::Debug for LosslessJsonDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LosslessJsonDocument")
            .field("source_bytes", &self.source.len())
            .finish_non_exhaustive()
    }
}

impl fmt::Display for LosslessJsonDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.root, formatter)
    }
}

/// A backend-neutral strict-JSON syntax diagnostic.
///
/// Byte ranges are zero-based half-open offsets into the accepted UTF-8 input.
/// Line and display-column policy remains deliberately outside this initial
/// contract.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct JsonSyntaxDiagnostic {
    byte_range: Range<usize>,
    message: String,
}

impl JsonSyntaxDiagnostic {
    /// Returns the zero-based half-open error range in the original bytes.
    pub fn byte_range(&self) -> Range<usize> {
        self.byte_range.clone()
    }

    /// Returns a human-readable description of the syntax problem.
    ///
    /// The wording is diagnostic text rather than a stable machine-readable
    /// error code.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for JsonSyntaxDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at bytes {}..{}",
            self.message, self.byte_range.start, self.byte_range.end
        )
    }
}

impl Error for JsonSyntaxDiagnostic {}

/// Errors returned while constructing a [`LosslessJsonDocument`].
#[derive(Debug)]
#[non_exhaustive]
pub enum LosslessJsonError {
    /// The input is not valid UTF-8.
    #[non_exhaustive]
    InvalidUtf8 {
        /// The standard UTF-8 decoding error.
        source: Utf8Error,
    },
    /// The input starts with a UTF-8 byte-order mark.
    Utf8ByteOrderMark,
    /// The UTF-8 text is not exactly one strict JSON value.
    #[non_exhaustive]
    InvalidSyntax {
        /// Backend-neutral byte-range and message information.
        diagnostic: JsonSyntaxDiagnostic,
    },
}

impl fmt::Display for LosslessJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 { source, .. } => {
                write!(formatter, "JSON input is not valid UTF-8: {source}")
            }
            Self::Utf8ByteOrderMark => {
                formatter.write_str("JSON input starts with a UTF-8 byte-order mark")
            }
            Self::InvalidSyntax { diagnostic, .. } => {
                write!(formatter, "invalid strict JSON: {diagnostic}")
            }
        }
    }
}

impl Error for LosslessJsonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidUtf8 { source, .. } => Some(source),
            Self::InvalidSyntax { diagnostic, .. } => Some(diagnostic),
            Self::Utf8ByteOrderMark => None,
        }
    }
}

fn strict_options() -> ParseOptions {
    ParseOptions {
        allow_comments: false,
        allow_loose_object_property_names: false,
        allow_trailing_commas: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

fn validate_lexical_domain(input: &str) -> Result<(), LosslessJsonError> {
    let mut in_string = false;
    let mut escaped = false;

    for (byte_index, character) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            } else if character <= '\u{001f}' {
                return Err(syntax_error(
                    byte_index..byte_index + character.len_utf8(),
                    "unescaped control character in string",
                ));
            }
        } else if character == '"' {
            in_string = true;
        } else if character.is_whitespace() && !matches!(character, ' ' | '\t' | '\n' | '\r') {
            return Err(syntax_error(
                byte_index..byte_index + character.len_utf8(),
                "non-JSON whitespace",
            ));
        }
    }

    Ok(())
}

fn validate_strict_syntax(input: &str) -> Result<(), LosslessJsonError> {
    let parsed =
        parse_to_ast(input, &CollectOptions::default(), &strict_options()).map_err(|error| {
            LosslessJsonError::InvalidSyntax {
                diagnostic: parser_diagnostic(error),
            }
        })?;

    if parsed.value.is_none() {
        return Err(syntax_error(0..0, "JSON text must contain one value"));
    }

    Ok(())
}

fn parser_diagnostic(error: ParseError) -> JsonSyntaxDiagnostic {
    let range = error.range();
    JsonSyntaxDiagnostic {
        byte_range: range.start..range.end,
        message: error.kind().to_string(),
    }
}

fn syntax_error(byte_range: Range<usize>, message: impl Into<String>) -> LosslessJsonError {
    LosslessJsonError::InvalidSyntax {
        diagnostic: JsonSyntaxDiagnostic {
            byte_range,
            message: message.into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{LosslessJsonDocument, LosslessJsonError};
    use std::error::Error;

    // Every input below is minimal synthetic JSON created for this test module.
    const EXACT_CASES: &[(&str, &str)] = &[
        ("compact", r#"{"a":1,"b":"hello"}"#),
        ("pretty", "{\n  \"a\": 1,\n  \"b\": \"hello\"\n}\n"),
        (
            "unusual whitespace",
            "  { \t\"a\"\t:\t1 ,\n\"b\":\ttrue }  ",
        ),
        ("key order", r#"{"z":1,"a":2,"m":3}"#),
        (
            "unknown nested data",
            r#"{"known":"value","_extension":{"n":1e+02,"s":"\u0061"}}"#,
        ),
        ("duplicate names", r#"{"a":1,"a":2}"#),
        ("number forms", r#"[0,-0,1e2,1.5,9007199254740993,3.14e-2]"#),
        (
            "unicode",
            r#"{"literal":"café😀","escaped":"\ud83d\ude00"}"#,
        ),
        ("string escapes", r#"{"s":"\n\t\r\"\\"}"#),
        ("CRLF", "{\r\n  \"a\": 1\r\n}\r\n"),
        ("root scalar", "true"),
        ("empty object", "{}"),
        ("empty array", "[]"),
    ];

    #[test]
    fn accepted_documents_preserve_exact_source_bytes() {
        for (name, input) in EXACT_CASES {
            let document = LosslessJsonDocument::parse(input.as_bytes())
                .unwrap_or_else(|error| panic!("{name}: {error}"));

            assert_eq!(document.source_text(), *input, "{name}: source text");
            assert_eq!(document.source_bytes(), input.as_bytes(), "{name}: bytes");
            assert_eq!(document.to_string(), *input, "{name}: CST display");
        }
    }

    #[test]
    fn large_numeric_lexemes_are_preserved_without_numeric_conversion() {
        let input = format!(
            "{{\"integer\":{},\"exponent\":1e999999}}",
            "9".repeat(2_000)
        );
        let document = LosslessJsonDocument::parse(input.as_bytes()).unwrap();

        assert_eq!(document.source_text(), input);
        assert_eq!(document.to_string(), input);
    }

    #[test]
    fn invalid_utf8_and_bom_have_distinct_errors() {
        let invalid = LosslessJsonDocument::parse(b"{\"a\":\"\xff\"}").unwrap_err();
        match invalid {
            LosslessJsonError::InvalidUtf8 { source, .. } => {
                assert_eq!(source.valid_up_to(), 6);
                assert_eq!(source.error_len(), Some(1));
            }
            other => panic!("unexpected error: {other}"),
        }

        assert!(matches!(
            LosslessJsonDocument::parse(b"\xef\xbb\xbf{\"a\":1}"),
            Err(LosslessJsonError::Utf8ByteOrderMark)
        ));
    }

    #[test]
    fn strict_extensions_and_lexical_gaps_are_rejected() {
        for input in [
            r#"{"a":1,}"#,
            r#"{"a":1}// comment"#,
            r#"{a:1}"#,
            r#"{'a':1}"#,
            r#"{"a":0x1A}"#,
            r#"{"a":+1}"#,
            r#"{"a":1} {"b":2}"#,
            r#"{"a":01}"#,
            r#"{"a":"\x01"}"#,
            "{\u{0000}\"a\":1}",
            "{\u{000b}\"a\":1}",
            "{\u{000c}\"a\":1}",
        ] {
            assert!(
                matches!(
                    LosslessJsonDocument::parse(input.as_bytes()),
                    Err(LosslessJsonError::InvalidSyntax { .. })
                ),
                "expected strict rejection for {input:?}"
            );
        }
    }

    #[test]
    fn syntax_diagnostics_use_original_byte_ranges() {
        let error = LosslessJsonDocument::parse(b"{\n  \"a\": 1,\n}").unwrap_err();
        let diagnostic = match error {
            LosslessJsonError::InvalidSyntax { diagnostic, .. } => diagnostic,
            other => panic!("unexpected error: {other}"),
        };

        assert_eq!(diagnostic.byte_range(), 10..11);
        assert_eq!(diagnostic.message(), "Trailing commas are not allowed");
        assert_eq!(
            diagnostic.to_string(),
            "Trailing commas are not allowed at bytes 10..11"
        );
    }

    #[test]
    fn lexical_preflight_reports_non_json_whitespace_and_controls() {
        for (input, expected_range, expected_message) in [
            ("{\u{00a0}\"a\":1}", 1..3, "non-JSON whitespace"),
            (
                "{\"a\":\"\u{0001}\"}",
                6..7,
                "unescaped control character in string",
            ),
        ] {
            let error = LosslessJsonDocument::parse(input.as_bytes()).unwrap_err();
            let diagnostic = match error {
                LosslessJsonError::InvalidSyntax { diagnostic, .. } => diagnostic,
                other => panic!("unexpected error: {other}"),
            };
            assert_eq!(diagnostic.byte_range(), expected_range);
            assert_eq!(diagnostic.message(), expected_message);
        }
    }

    #[test]
    fn empty_input_is_not_a_document() {
        let error = LosslessJsonDocument::parse(b" \t\r\n").unwrap_err();
        let diagnostic = match error {
            LosslessJsonError::InvalidSyntax { diagnostic, .. } => diagnostic,
            other => panic!("unexpected error: {other}"),
        };

        assert_eq!(diagnostic.byte_range(), 0..0);
        assert_eq!(diagnostic.message(), "JSON text must contain one value");
    }

    #[test]
    fn observed_nesting_boundary_is_retained() {
        let accepted = format!("{}0{}", "[".repeat(512), "]".repeat(512));
        let rejected = format!("{}0{}", "[".repeat(513), "]".repeat(513));

        assert!(LosslessJsonDocument::parse(accepted.as_bytes()).is_ok());
        let error = LosslessJsonDocument::parse(rejected.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("Maximum nesting depth exceeded"));
    }

    #[test]
    fn error_sources_remain_inspectable() {
        let utf8 = LosslessJsonDocument::parse(b"\xff").unwrap_err();
        assert!(utf8.source().is_some());

        let syntax = LosslessJsonDocument::parse(b"{").unwrap_err();
        assert!(syntax.source().is_some());

        let bom = LosslessJsonDocument::parse(b"\xef\xbb\xbf{}").unwrap_err();
        assert!(bom.source().is_none());
    }
}
