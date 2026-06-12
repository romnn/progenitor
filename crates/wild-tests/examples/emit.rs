// Copyright 2026

//! Materialize the generated client crate for the named corpus specs so
//! the code can be inspected (and its spec-tests iterated on) without
//! running the whole compile tier:
//! `cargo run -p wild-tests --example emit -- discord [out_dir]`.
//! The last argument is the output workspace root when it contains a
//! path separator; it defaults to `target/tmp/wild-emit`.

use wild_tests::{load_manifest, write_compile_crate, write_compile_workspace};

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let out_root = if args.last().is_some_and(|a| a.contains('/')) {
        std::path::PathBuf::from(args.pop().unwrap())
    } else {
        std::path::PathBuf::from("target/tmp/wild-emit")
    };
    assert!(!args.is_empty(), "usage: emit <spec>... [out_dir]");

    let specs = load_manifest().expect("manifest loads");
    let mut members = Vec::new();
    for entry in &specs {
        if !args.iter().any(|n| n == &entry.name) {
            continue;
        }
        match write_compile_crate(entry, &out_root) {
            Ok(dir) => {
                println!("{}: {}", entry.name, dir.display());
                members.push(entry.name.clone());
            }
            Err(err) => {
                eprintln!("{}: generation failed: {err}", entry.name);
                std::process::exit(1);
            }
        }
    }
    write_compile_workspace(&out_root, &members).expect("workspace manifest");
    println!("workspace: {}", out_root.display());
}
