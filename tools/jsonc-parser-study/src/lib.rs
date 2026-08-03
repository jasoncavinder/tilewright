// SPDX-License-Identifier: MPL-2.0

#[cfg(test)]
mod tests {
    use std::fmt;

    use jsonc_parser::cst::{CstInputValue, CstNode, CstObject, CstRootNode};
    use jsonc_parser::{CollectOptions, ParseOptions, parse_to_ast};
    use serde_json::{Value, json};

    const NO_OP_CASES: &[(&str, &str)] = &[
        ("compact_no_newline", r#"{"a":1,"b":"hello"}"#),
        ("pretty_whitespace", "{\n  \"a\": 1,\n  \"b\": \"hello\"\n}"),
        (
            "unusual_whitespace",
            "  { \t\"a\"\t:\t1 ,\n\"b\":\t\"hello\" }  ",
        ),
        ("key_order_stable", r#"{"z":1,"a":2,"m":3,"d":4}"#),
        ("unknown_top_level", r#"{"z_customPlugin":true,"a":1}"#),
        (
            "unknown_nested_deep",
            r#"{"data":{"_ext":{"x":1,"y":[2,3]}},"a":1}"#,
        ),
        ("array_with_nulls", r#"[null,1,"two",{"three":3},null]"#),
        (
            "mixed_nested_array",
            r#"{"items":[null,{"a":1},"b",false,true,0.5]}"#,
        ),
        (
            "number_forms",
            r#"{"n":[0,-0,1e2,1.5,9007199254740993,-42,3.14e-2]}"#,
        ),
        ("unicode_literal", r#"{"s":"café","e":"😀"}"#),
        ("unicode_escaped", r#"{"s":"caf\u00e9","e":"\ud83d\ude00"}"#),
        ("string_escapes", r#"{"s":"\n\t\r\"\\"}"#),
        ("duplicate_keys", r#"{"a":1,"a":2}"#),
        ("crlf_endings", "{\r\n  \"a\": 1\r\n}"),
        ("final_newline", "{\"a\":1}\n"),
        ("no_final_newline", "{\"a\":1}"),
        ("empty_object", "{}"),
        ("empty_array", "[]"),
        ("deeply_nested", r#"{"a":{"b":{"c":{"d":1}}}}"#),
    ];

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

    fn validate_json_lexical_domain(input: &str) -> Result<(), String> {
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
                    return Err(format!("unescaped control character at byte {byte_index}"));
                }
            } else if character == '"' {
                in_string = true;
            } else if character.is_whitespace() && !matches!(character, ' ' | '\t' | '\n' | '\r') {
                return Err(format!("non-JSON whitespace at byte {byte_index}"));
            }
        }

        Ok(())
    }

    fn strict_validate(input: &str) -> Result<(), String> {
        validate_json_lexical_domain(input)?;
        let parsed = parse_to_ast(input, &CollectOptions::default(), &strict_options())
            .map_err(|error| error.to_string())?;
        if parsed.value.is_none() {
            return Err("JSON text must contain one value".to_owned());
        }
        Ok(())
    }

    fn strict_parse(input: &str) -> Result<CstRootNode, String> {
        strict_validate(input)?;
        CstRootNode::parse(input, &strict_options()).map_err(|error| error.to_string())
    }

    fn assert_strict_semantics(root: &CstRootNode, expected: Value) {
        let output = root.to_string();
        strict_validate(&output).expect("mutation output must remain strict JSON");
        assert_eq!(serde_json::from_str::<Value>(&output).unwrap(), expected);
    }

    fn assert_strict_output(root: &CstRootNode, expected_output: &str, expected: Value) {
        let output = root.to_string();
        strict_validate(&output).expect("mutation output must remain strict JSON");
        assert_eq!(output, expected_output);
        assert_eq!(serde_json::from_str::<Value>(&output).unwrap(), expected);
    }

    fn nested_arrays(depth: usize) -> String {
        format!("{}0{}", "[".repeat(depth), "]".repeat(depth))
    }

    fn validated_number(raw: &str) -> Result<CstInputValue, String> {
        validate_json_lexical_domain(raw)?;
        let parsed = parse_to_ast(raw, &CollectOptions::default(), &strict_options())
            .map_err(|error| error.to_string())?;
        match parsed
            .value
            .and_then(|value| value.as_number_lit().cloned())
        {
            Some(_) => Ok(CstInputValue::Number(raw.to_owned())),
            None => Err("value is not exactly one JSON number".to_owned()),
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum UniqueMutationError {
        Missing,
        Ambiguous,
    }

    impl fmt::Display for UniqueMutationError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Missing => formatter.write_str("target property is missing"),
                Self::Ambiguous => formatter.write_str("target property is ambiguous"),
            }
        }
    }

    fn remove_unique_property(object: &CstObject, target: &str) -> Result<(), UniqueMutationError> {
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
            [] => Err(UniqueMutationError::Missing),
            [property] => {
                property.clone().remove();
                Ok(())
            }
            _ => Err(UniqueMutationError::Ambiguous),
        }
    }

    fn contains_comment(node: &CstNode) -> bool {
        node.is_comment() || node.children().iter().any(contains_comment)
    }

    #[test]
    fn strict_parser_rejects_json_extensions_and_non_json_lexemes() {
        let cases = [
            ("empty input", ""),
            ("whitespace-only input", " \t\r\n"),
            ("line comment", r#"{"a":1}// comment"#),
            ("block comment", r#"{"a":1/* comment */}"#),
            ("unquoted key", r#"{a:1}"#),
            ("trailing comma", r#"{"a":1,}"#),
            ("missing comma", r#"{"a":1 "b":2}"#),
            ("single quote", r#"{'a':1}"#),
            ("hex number", r#"{"a":0x1A}"#),
            ("unary plus", r#"{"a":+1}"#),
            ("multiple roots", r#"{"a":1} {"b":2}"#),
            ("leading zero", r#"{"a":01}"#),
            ("invalid escape", r#"{"a":"\x01"}"#),
            ("raw C0 control", "{\"a\":\"\u{0001}\"}"),
            ("vertical tab", "{\u{000b}\"a\":1}"),
            ("form feed", "{\u{000c}\"a\":1}"),
            ("non-breaking space", "{\u{00a0}\"a\":1}"),
        ];

        for (name, input) in cases {
            assert!(
                strict_parse(input).is_err(),
                "expected rejection for {name}"
            );
        }
    }

    #[test]
    fn strict_parser_accepts_large_lexemes_without_numeric_interpretation() {
        let huge_integer = format!("{{\"n\":{}}}", "9".repeat(2_000));
        let huge_exponent = r#"{"n":1e999999}"#;

        for input in [&huge_integer, huge_exponent] {
            let root = strict_parse(input).expect("syntax-only validation must accept the lexeme");
            assert_eq!(root.to_string(), input);
        }
    }

    #[test]
    fn strict_parser_enforces_the_observed_nesting_limit() {
        assert!(strict_parse(&nested_arrays(512)).is_ok());
        let error = strict_parse(&nested_arrays(513)).unwrap_err();
        assert!(error.contains("Maximum nesting depth exceeded"));
    }

    #[test]
    fn direct_parser_default_accepts_trailing_commas() {
        assert!(CstRootNode::parse(r#"{"a":1,}"#, &ParseOptions::default()).is_ok());
    }

    #[test]
    fn direct_parser_strict_options_reject_trailing_commas() {
        let error = CstRootNode::parse(r#"{"a":1,}"#, &strict_options()).unwrap_err();
        assert_eq!(error.kind().to_string(), "Trailing commas are not allowed");
    }

    #[test]
    fn direct_cst_parse_preserves_comments_despite_strict_comment_option() {
        let input = "{\n  // retained\n  \"a\": 1\n}";
        let root = CstRootNode::parse(input, &strict_options()).unwrap();
        assert_eq!(root.to_string(), input);
        assert!(root.children().iter().any(contains_comment));
        assert!(strict_validate(input).is_err());
    }

    #[test]
    fn direct_parser_rejects_utf8_bom() {
        let input = "\u{feff}{\"a\":1}";
        assert!(CstRootNode::parse(input, &ParseOptions::default()).is_err());
        assert!(strict_parse(input).is_err());
    }

    #[test]
    fn direct_parser_reports_exact_diagnostic_location() {
        let input = "{\n  \"a\": 1,\n}";
        let error = CstRootNode::parse(input, &strict_options()).unwrap_err();
        let range = error.range();

        assert_eq!(error.kind().to_string(), "Trailing commas are not allowed");
        assert_eq!((range.start, range.end), (10, 11));
        assert_eq!(error.line_display(), 2);
        assert_eq!(error.column_display(), 9);
        assert_eq!(
            error.to_string(),
            "Trailing commas are not allowed on line 2 column 9"
        );
    }

    #[test]
    fn no_op_round_trip_preserves_every_declared_matrix_case() {
        for (name, input) in NO_OP_CASES {
            let root = strict_parse(input).unwrap_or_else(|error| panic!("{name}: {error}"));
            assert_eq!(root.to_string(), *input, "no-op mismatch for {name}");
        }
    }

    #[test]
    fn string_constructor_escapes_required_characters_and_preserves_unicode() {
        let cases = [
            ("quote\"key", "value with \"quotes\""),
            ("backslash\\key", "value with \\ backslash"),
            ("newline\nkey", "value with \n newline"),
            ("carriage\rkey", "value with \r return"),
            ("tab\tkey", "value with \t tab"),
            ("backspace\u{0008}key", "value with \u{0008} backspace"),
            ("formfeed\u{000c}key", "value with \u{000c} form feed"),
            ("control\u{0001}key", "value with \u{0001} control"),
            ("café", "éclair"),
            ("😀", "🧑‍🦰"),
        ];

        for (name, value) in cases {
            let root = strict_parse(r#"{"base":true}"#).unwrap();
            root.object_value()
                .unwrap()
                .append(name, CstInputValue::String(value.to_owned()));
            let output = root.to_string();
            strict_validate(&output).unwrap();
            let decoded = serde_json::from_str::<Value>(&output).unwrap();
            assert_eq!(decoded.get(name).and_then(Value::as_str), Some(value));
        }
    }

    #[test]
    fn unchecked_raw_literal_apis_can_create_invalid_json() {
        let number_root = strict_parse(r#"{"a":1}"#).unwrap();
        number_root
            .object_value()
            .unwrap()
            .append("bad", CstInputValue::Number("not-a-number".to_owned()));
        assert!(strict_validate(&number_root.to_string()).is_err());

        let raw_number_root = strict_parse(r#"{"a":1}"#).unwrap();
        let number = raw_number_root
            .object_value()
            .unwrap()
            .get("a")
            .unwrap()
            .value()
            .unwrap()
            .as_number_lit()
            .unwrap();
        number.set_raw_value("1]".to_owned());
        assert!(strict_validate(&raw_number_root.to_string()).is_err());

        let raw_string_root = strict_parse(r#"{"a":"ok"}"#).unwrap();
        let string = raw_string_root
            .object_value()
            .unwrap()
            .get("a")
            .unwrap()
            .value()
            .unwrap()
            .as_string_lit()
            .unwrap();
        string.set_raw_value("\"unterminated".to_owned());
        assert!(strict_validate(&raw_string_root.to_string()).is_err());
    }

    #[test]
    fn validated_number_refuses_invalid_input_before_mutation() {
        let root = strict_parse(r#"{"a":1}"#).unwrap();
        let original = root.to_string();
        assert!(validated_number("not-a-number").is_err());
        assert_eq!(root.to_string(), original);

        let value = validated_number("-1.25e+30").unwrap();
        root.object_value().unwrap().append("valid", value);
        strict_validate(&root.to_string()).unwrap();
    }

    #[test]
    fn object_deletion_envelope_is_exact_across_positions() {
        let cases = [
            (
                r#"{"a":1,"b":2,"c":3}"#,
                0,
                r#"{"b":2,"c":3}"#,
                json!({"b": 2, "c": 3}),
            ),
            (
                r#"{"a":1,"b":2,"c":3}"#,
                1,
                r#"{"a":1,"c":3}"#,
                json!({"a": 1, "c": 3}),
            ),
            (
                r#"{"a":1,"b":2,"c":3}"#,
                2,
                r#"{"a":1,"b":2}"#,
                json!({"a": 1, "b": 2}),
            ),
            (r#"{"a":1}"#, 0, r#"{}"#, json!({})),
        ];

        for (input, index, expected_output, expected) in cases {
            let root = strict_parse(input).unwrap();
            root.object_value().unwrap().properties()[index]
                .clone()
                .remove();
            assert_strict_output(&root, expected_output, expected);
        }
    }

    #[test]
    fn array_deletion_envelope_is_exact_across_positions() {
        let cases = [
            ("[1,2,3]", 0, "[2,3]", json!([2, 3])),
            ("[1,2,3]", 1, "[1,3]", json!([1, 3])),
            ("[1,2,3]", 2, "[1,2]", json!([1, 2])),
            ("[1]", 0, "[]", json!([])),
        ];

        for (input, index, expected_output, expected) in cases {
            let root = strict_parse(input).unwrap();
            root.array_value().unwrap().elements()[index]
                .clone()
                .remove();
            assert_strict_output(&root, expected_output, expected);
        }
    }

    #[test]
    fn insertion_envelope_covers_positions_and_layouts() {
        for (index, name, expected_output) in [
            (0, "a", "{\n  \"a\": 1,\n  \"b\":2,\n  \"d\":4\n}"),
            (1, "c", "{\n  \"b\":2,\n  \"c\": 1,\n  \"d\":4\n}"),
            (2, "e", "{\n  \"b\":2,\n  \"d\":4,\n  \"e\": 1\n}"),
        ] {
            let root = strict_parse(r#"{"b":2,"d":4}"#).unwrap();
            root.object_value()
                .unwrap()
                .insert(index, name, validated_number("1").unwrap());
            let output = root.to_string();
            strict_validate(&output).unwrap();
            assert_eq!(output, expected_output);
        }

        for (index, expected_output, expected) in [
            (0, "[0, 1,2]", json!([0, 1, 2])),
            (1, "[1, 0, 2]", json!([1, 0, 2])),
            (2, "[1,2, 0]", json!([1, 2, 0])),
        ] {
            let root = strict_parse("[1,2]").unwrap();
            root.array_value()
                .unwrap()
                .insert(index, validated_number("0").unwrap());
            assert_strict_output(&root, expected_output, expected);
        }

        let multiline = strict_parse("{\n  \"a\": 1,\n  \"c\": 3\n}").unwrap();
        multiline
            .object_value()
            .unwrap()
            .insert(1, "b", validated_number("2").unwrap());
        assert_eq!(
            multiline.to_string(),
            "{\n  \"a\": 1,\n  \"b\": 2,\n  \"c\": 3\n}"
        );
        strict_validate(&multiline.to_string()).unwrap();

        let crlf = strict_parse("{\r\n  \"a\": 1\r\n}").unwrap();
        crlf.object_value()
            .unwrap()
            .append("b", validated_number("2").unwrap());
        assert_strict_output(
            &crlf,
            "{\r\n  \"a\": 1,\r\n  \"b\": 2\r\n}",
            json!({"a": 1, "b": 2}),
        );

        let unusual = strict_parse("{\n\t\"a\"\t:\t1\n}").unwrap();
        unusual
            .object_value()
            .unwrap()
            .append("b", validated_number("2").unwrap());
        assert_strict_output(
            &unusual,
            "{\n\t\"a\"\t:\t1,\n\t\"b\": 2\n}",
            json!({"a": 1, "b": 2}),
        );
    }

    #[test]
    fn mutations_preserve_adversarial_neighbors_outside_the_envelope() {
        let input = concat!(
            "{\n",
            "  \"unknown\": {\"n\":1e+02,\"s\":\"\\u0061\"},\n",
            "  \"remove\": 0,\n",
            "  \"tail\": [-0,9007199254740993]\n",
            "}"
        );
        let after_removal = concat!(
            "{\n",
            "  \"unknown\": {\"n\":1e+02,\"s\":\"\\u0061\"},\n",
            "  \"tail\": [-0,9007199254740993]\n",
            "}"
        );
        let root = strict_parse(input).unwrap();
        remove_unique_property(&root.object_value().unwrap(), "remove").unwrap();
        assert_eq!(root.to_string(), after_removal);
        strict_validate(&root.to_string()).unwrap();

        root.object_value()
            .unwrap()
            .insert(1, "added", validated_number("7e+00").unwrap());
        let after_insertion = concat!(
            "{\n",
            "  \"unknown\": {\"n\":1e+02,\"s\":\"\\u0061\"},\n",
            "  \"added\": 7e+00,\n",
            "  \"tail\": [-0,9007199254740993]\n",
            "}"
        );
        assert_eq!(root.to_string(), after_insertion);
        strict_validate(&root.to_string()).unwrap();
    }

    #[test]
    fn duplicate_targets_are_refused_without_changing_bytes() {
        for input in [r#"{"a":1,"a":2}"#, r#"{"a":1,"\u0061":2}"#] {
            let root = strict_parse(input).unwrap();
            let original = root.to_string();
            assert_eq!(
                remove_unique_property(&root.object_value().unwrap(), "a"),
                Err(UniqueMutationError::Ambiguous)
            );
            assert_eq!(root.to_string(), original);
        }

        let root = strict_parse(r#"{"nested":{"b":1,"\u0062":2}}"#).unwrap();
        let original = root.to_string();
        let nested = root
            .object_value()
            .unwrap()
            .get("nested")
            .unwrap()
            .value()
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(
            remove_unique_property(&nested, "b"),
            Err(UniqueMutationError::Ambiguous)
        );
        assert_eq!(root.to_string(), original);
    }

    #[test]
    fn unique_target_mutation_succeeds_and_missing_target_is_reported() {
        let root = strict_parse(r#"{"a":1,"b":2}"#).unwrap();
        let object = root.object_value().unwrap();
        assert_eq!(remove_unique_property(&object, "a"), Ok(()));
        assert_strict_semantics(&root, json!({"b": 2}));
        assert_eq!(
            remove_unique_property(&object, "missing"),
            Err(UniqueMutationError::Missing)
        );
    }
}
