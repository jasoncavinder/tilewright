// SPDX-License-Identifier: MPL-2.0

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use jsonc_parser_study::strict_parse;

#[derive(Default)]
struct Counts {
    json_files: usize,
    valid_utf8: usize,
    invalid_utf8: usize,
    bom: usize,
    strict_accepted: usize,
    strict_rejected: usize,
    byte_identical: usize,
    byte_changed: usize,
    symlinks_skipped: usize,
    read_errors: usize,
}

fn inspect_tree(root: &Path, counts: &mut Counts) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_symlink() {
            counts.symlinks_skipped += 1;
        } else if file_type.is_dir() {
            inspect_tree(&path, counts)?;
        } else if file_type.is_file() && path.extension().is_some_and(|value| value == "json") {
            inspect_json(&path, counts);
        }
    }
    Ok(())
}

fn inspect_json(path: &Path, counts: &mut Counts) {
    counts.json_files += 1;
    let Ok(bytes) = fs::read(path) else {
        counts.read_errors += 1;
        return;
    };

    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        counts.bom += 1;
        return;
    }

    let Ok(input) = std::str::from_utf8(&bytes) else {
        counts.invalid_utf8 += 1;
        return;
    };
    counts.valid_utf8 += 1;

    let Ok(root) = strict_parse(input) else {
        counts.strict_rejected += 1;
        return;
    };
    counts.strict_accepted += 1;
    if root.to_string().as_bytes() == bytes {
        counts.byte_identical += 1;
    } else {
        counts.byte_changed += 1;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: corpus_noop <authorized-project-copy>")?;
    let metadata = fs::symlink_metadata(&root)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("corpus root must be a non-symlink directory".into());
    }

    let mut counts = Counts::default();
    inspect_tree(&root, &mut counts)?;

    println!("json_files={}", counts.json_files);
    println!("valid_utf8={}", counts.valid_utf8);
    println!("invalid_utf8={}", counts.invalid_utf8);
    println!("bom={}", counts.bom);
    println!("strict_accepted={}", counts.strict_accepted);
    println!("strict_rejected={}", counts.strict_rejected);
    println!("byte_identical={}", counts.byte_identical);
    println!("byte_changed={}", counts.byte_changed);
    println!("symlinks_skipped={}", counts.symlinks_skipped);
    println!("read_errors={}", counts.read_errors);

    if counts.read_errors > 0 || counts.byte_changed > 0 {
        return Err("corpus experiment encountered an unsafe result".into());
    }
    Ok(())
}
