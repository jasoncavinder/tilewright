// SPDX-License-Identifier: MPL-2.0

use indexmap::IndexMap;
use jsonc_parser::cst::{CstInputValue, CstObject, CstObjectProp, CstRootNode};
use jsonc_parser_study::{strict_parse, strict_validate};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
struct TypedDocument {
    known: String,
    #[serde(flatten)]
    extensions: IndexMap<String, Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UniquePropertyError {
    Missing,
    Ambiguous,
}

#[derive(Debug, PartialEq, Eq)]
enum ByteInputError {
    Utf8,
    Bom,
    Syntax,
}

struct ComparisonCase {
    name: &'static str,
    input: &'static str,
    expected_known: &'static str,
    typed_is_exact: bool,
    ordered_dom_is_exact: bool,
}

const COMPARISON_CASES: &[ComparisonCase] = &[
    ComparisonCase {
        name: "compact_known_first",
        input: r#"{"known":"alpha","unknown":{"n":1e+02}}"#,
        expected_known: "alpha",
        typed_is_exact: true,
        ordered_dom_is_exact: true,
    },
    ComparisonCase {
        name: "unknown_before_known",
        input: r#"{"unknown":{"z":1,"a":2},"known":"alpha"}"#,
        expected_known: "alpha",
        typed_is_exact: false,
        ordered_dom_is_exact: true,
    },
    ComparisonCase {
        name: "outer_whitespace",
        input: "{\n  \"known\": \"alpha\",\n  \"unknown\": [1, 2]\n}\n",
        expected_known: "alpha",
        typed_is_exact: false,
        ordered_dom_is_exact: false,
    },
    ComparisonCase {
        name: "known_string_escape",
        input: r#"{"known":"\u0061","unknown":true}"#,
        expected_known: "a",
        typed_is_exact: false,
        ordered_dom_is_exact: false,
    },
    ComparisonCase {
        name: "raw_unknown_lexemes",
        input: r#"{"known":"alpha","unknown":{"n":1e+02,"s":"\u0061"}}"#,
        expected_known: "alpha",
        typed_is_exact: false,
        ordered_dom_is_exact: false,
    },
    ComparisonCase {
        name: "crlf_and_trailing_newline",
        input: "{\r\n  \"known\": \"alpha\",\r\n  \"unknown\": true\r\n}\r\n",
        expected_known: "alpha",
        typed_is_exact: false,
        ordered_dom_is_exact: false,
    },
];

fn typed_round_trip(input: &str) -> Result<(String, String), serde_json::Error> {
    let document = serde_json::from_str::<TypedDocument>(input)?;
    let known = document.known.clone();
    serde_json::to_string(&document).map(|output| (known, output))
}

fn ordered_dom_round_trip(input: &str) -> Result<(String, String), serde_json::Error> {
    let document = serde_json::from_str::<Value>(input)?;
    let known = document
        .get("known")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    serde_json::to_string(&document).map(|output| (known, output))
}

fn unique_property(object: &CstObject, target: &str) -> Result<CstObjectProp, UniquePropertyError> {
    let matching = object
        .properties()
        .into_iter()
        .filter(|property| {
            property
                .name()
                .and_then(|name| name.decoded_value().ok())
                .is_some_and(|name| name == target)
        })
        .collect::<Vec<_>>();

    match matching.as_slice() {
        [] => Err(UniquePropertyError::Missing),
        [property] => Ok(property.clone()),
        _ => Err(UniquePropertyError::Ambiguous),
    }
}

fn cst_string_view(root: &CstRootNode, target: &str) -> Result<String, String> {
    let object = root
        .object_value()
        .ok_or_else(|| "root is not an object".to_owned())?;
    let property = unique_property(&object, target)
        .map_err(|error| format!("property lookup failed: {error:?}"))?;
    property
        .value()
        .and_then(|value| value.as_string_lit())
        .ok_or_else(|| "property is not a string".to_owned())?
        .decoded_value()
        .map_err(|error| error.to_string())
}

fn replace_unique_cst_string(
    root: &CstRootNode,
    target: &str,
    replacement: &str,
) -> Result<(), UniquePropertyError> {
    let object = root.object_value().ok_or(UniquePropertyError::Missing)?;
    unique_property(&object, target)?.set_value(CstInputValue::String(replacement.to_owned()));
    Ok(())
}

fn strict_parse_bytes(bytes: &[u8]) -> Result<CstRootNode, ByteInputError> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(ByteInputError::Bom);
    }
    let input = std::str::from_utf8(bytes).map_err(|_| ByteInputError::Utf8)?;
    strict_parse(input).map_err(|_| ByteInputError::Syntax)
}

#[test]
fn three_required_strategies_have_comparable_no_op_results() {
    for case in COMPARISON_CASES {
        let (typed_known, typed_output) =
            typed_round_trip(case.input).unwrap_or_else(|error| panic!("{}: {error}", case.name));
        let (dom_known, dom_output) = ordered_dom_round_trip(case.input)
            .unwrap_or_else(|error| panic!("{}: {error}", case.name));
        let cst = strict_parse(case.input).unwrap_or_else(|error| panic!("{}: {error}", case.name));

        assert_eq!(
            typed_known, case.expected_known,
            "{}: typed view",
            case.name
        );
        assert_eq!(
            dom_known, case.expected_known,
            "{}: ordered DOM view",
            case.name
        );
        assert_eq!(
            typed_output == case.input,
            case.typed_is_exact,
            "{}: typed fidelity",
            case.name
        );
        assert_eq!(
            dom_output == case.input,
            case.ordered_dom_is_exact,
            "{}: ordered DOM fidelity",
            case.name
        );
        assert_eq!(cst.to_string(), case.input, "{}: CST fidelity", case.name);
        strict_validate(&typed_output).unwrap();
        strict_validate(&dom_output).unwrap();
    }
}

#[test]
fn duplicate_names_distinguish_refusal_from_silent_collapse() {
    let input = r#"{"known":"first","known":"last","unknown":1}"#;

    assert!(typed_round_trip(input).is_err());

    let (dom_known, dom_output) = ordered_dom_round_trip(input).unwrap();
    assert_eq!(dom_known, "last");
    assert_eq!(dom_output, r#"{"known":"last","unknown":1}"#);

    let cst = strict_parse(input).unwrap();
    assert_eq!(cst.to_string(), input);
    assert!(matches!(
        unique_property(&cst.object_value().unwrap(), "known"),
        Err(UniquePropertyError::Ambiguous)
    ));
}

#[test]
fn cst_typed_view_reads_without_changing_bytes() {
    let input = r#"{"unknown_before":{"n":1e+02},"known":"\u0061","unknown_after":"\u0062"}"#;
    let root = strict_parse(input).unwrap();

    assert_eq!(cst_string_view(&root, "known").unwrap(), "a");
    assert_eq!(root.to_string(), input);
}

#[test]
fn cst_typed_scalar_replacement_has_an_exact_envelope() {
    for (input, expected) in [
        (
            r#"{"unknown_before":{"n":1e+02},"known":"\u0061","unknown_after":"\u0062"}"#,
            r#"{"unknown_before":{"n":1e+02},"known":"changed","unknown_after":"\u0062"}"#,
        ),
        (
            "{\r\n  \"unknown_before\": 1e+02,\r\n  \"known\": \"alpha\",\r\n  \"unknown_after\": \"\\u0062\"\r\n}",
            "{\r\n  \"unknown_before\": 1e+02,\r\n  \"known\": \"changed\",\r\n  \"unknown_after\": \"\\u0062\"\r\n}",
        ),
    ] {
        let root = strict_parse(input).unwrap();
        replace_unique_cst_string(&root, "known", "changed").unwrap();

        assert_eq!(root.to_string(), expected);
        assert_eq!(cst_string_view(&root, "known").unwrap(), "changed");
        strict_validate(&root.to_string()).unwrap();
    }

    let ambiguous = r#"{"known":"first","known":"last"}"#;
    let root = strict_parse(ambiguous).unwrap();
    assert_eq!(
        replace_unique_cst_string(&root, "known", "changed"),
        Err(UniquePropertyError::Ambiguous)
    );
    assert_eq!(root.to_string(), ambiguous);
}

#[test]
fn byte_boundary_can_refuse_invalid_utf8_and_bom_without_normalizing() {
    let valid = br#"{"known":"alpha"}"#;
    assert_eq!(
        strict_parse_bytes(valid).unwrap().to_string().as_bytes(),
        valid
    );
    assert!(matches!(
        strict_parse_bytes(b"{\"known\":\"\xff\"}"),
        Err(ByteInputError::Utf8)
    ));
    assert!(matches!(
        strict_parse_bytes(b"\xef\xbb\xbf{\"known\":\"alpha\"}"),
        Err(ByteInputError::Bom)
    ));
    assert!(matches!(
        strict_parse_bytes(b"{]"),
        Err(ByteInputError::Syntax)
    ));
}

#[test]
fn typed_mutations_preserve_unknown_meaning_but_not_all_lexemes() {
    let input = r#"{"unknown_before":{"n":1e+02,"s":"\u0061"},"known":"alpha"}"#;

    let mut typed = serde_json::from_str::<TypedDocument>(input).unwrap();
    typed.known = "changed".to_owned();
    let typed_output = serde_json::to_string(&typed).unwrap();
    assert_eq!(
        typed_output,
        r#"{"known":"changed","unknown_before":{"n":1e+02,"s":"a"}}"#
    );

    let mut dom = serde_json::from_str::<Value>(input).unwrap();
    dom["known"] = Value::String("changed".to_owned());
    let dom_output = serde_json::to_string(&dom).unwrap();
    assert_eq!(
        dom_output,
        r#"{"unknown_before":{"n":1e+02,"s":"a"},"known":"changed"}"#
    );

    for output in [typed_output, dom_output] {
        let value = serde_json::from_str::<Value>(&output).unwrap();
        assert_eq!(value["unknown_before"]["n"].as_f64(), Some(100.0));
        assert_eq!(value["unknown_before"]["s"], "a");
        assert_eq!(value["known"], "changed");
    }
}
