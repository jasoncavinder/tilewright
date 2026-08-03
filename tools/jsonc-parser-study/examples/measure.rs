// SPDX-License-Identifier: MPL-2.0

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

use indexmap::IndexMap;
use jsonc_parser_study::strict_parse;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize)]
struct TypedDocument {
    known: String,
    #[serde(flatten)]
    extensions: IndexMap<String, Value>,
}

struct MeasurementCase {
    name: &'static str,
    records: usize,
    iterations: usize,
}

fn synthetic_document(records: usize) -> String {
    let mut output = String::from("{\"unknown_before\":[");
    for index in 0..records {
        if index > 0 {
            output.push(',');
        }
        write!(
            output,
            "{{\"id\":{index},\"n\":1e+02,\"s\":\"\\u0061\",\"active\":true}}"
        )
        .unwrap();
    }
    output.push_str("],\"known\":\"alpha\",\"unknown_after\":{\"z\":1,\"a\":2}}");
    output
}

fn measure(
    candidate: &str,
    case: &MeasurementCase,
    input: &str,
    mut operation: impl FnMut(&str) -> usize,
) {
    black_box(operation(black_box(input)));

    let start = Instant::now();
    let mut output_bytes = 0;
    for _ in 0..case.iterations {
        output_bytes = black_box(operation(black_box(input)));
    }
    let elapsed = start.elapsed();
    let total_micros = elapsed.as_micros();
    let per_iteration = total_micros as f64 / case.iterations as f64;

    println!(
        "{candidate},{},{},{},{total_micros},{per_iteration:.2},{output_bytes}",
        case.name,
        input.len(),
        case.iterations
    );
}

fn main() {
    let cases = [
        MeasurementCase {
            name: "small",
            records: 16,
            iterations: 200,
        },
        MeasurementCase {
            name: "medium",
            records: 1_600,
            iterations: 20,
        },
        MeasurementCase {
            name: "large",
            records: 8_000,
            iterations: 5,
        },
    ];

    println!(
        "candidate,case,input_bytes,iterations,total_microseconds,per_iteration_microseconds,output_bytes"
    );
    for case in cases {
        let input = synthetic_document(case.records);

        measure("typed_extensions", &case, &input, |input| {
            let value = serde_json::from_str::<TypedDocument>(input).unwrap();
            serde_json::to_string(&value).unwrap().len()
        });
        measure("ordered_dom", &case, &input, |input| {
            let value = serde_json::from_str::<Value>(input).unwrap();
            serde_json::to_string(&value).unwrap().len()
        });
        measure("cst", &case, &input, |input| {
            strict_parse(input).unwrap().to_string().len()
        });
    }
}
