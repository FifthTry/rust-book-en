#[macro_use] extern crate lazy_static;
extern crate regex;

use std::env;
use std::io;
use std::io::{Read, Write};
use std::process::exit;
use std::fs::{create_dir, read_dir, File};
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use regex::Regex;


static PATTERNS: &'static [(&'static str, &'static str)] = &[
    (r"ch(\d\d)-\d\d-.*\.md", "chapter$1.md"),
    (r"appendix-(\d\d).*\.md", "appendix.md"),
];


lazy_static! {
    static ref MATCHERS: Vec<(Regex, &'static str)> = {
        PATTERNS.iter()
            .map(|&(expr, repl)| (Regex::new(expr).unwrap(), repl))
            .collect()
    };
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} <src-dir> <target-dir>", args[0]);
        exit(1);
    }

    let source_dir = ensure_dir_exists(&args[1]).unwrap();
    let target_dir = ensure_dir_exists(&args[2]).unwrap();

    let mut matched_files = match_files(source_dir, target_dir);
    matched_files.sort();

    for (target_path, source_paths) in group_by_target(matched_files) {
        concat_files(source_paths, target_path).unwrap();
    }
}



fn match_files(source_dir: &Path, target_dir: &Path) -> Vec<(PathBuf, PathBuf)> {
    read_dir(source_dir)
        .expect("Unable to read source directory")
        .filter_map(|maybe_entry| maybe_entry.ok())
        .filter_map(|entry| {
            let source_filename = entry.file_name();
            let source_filename = &source_filename.to_string_lossy().into_owned();
            for &(ref regex, replacement) in MATCHERS.iter() {
                if regex.is_match(source_filename) {
                    let target_filename = regex.replace_all(source_filename, replacement);
                    let source_path = entry.path();
                    let mut target_path = PathBuf::from(&target_dir);
                    target_path.push(target_filename);
                    return Some((source_path, target_path));
                }
            }
            None
        })
        .collect()
}


fn group_by_target(matched_files: Vec<(PathBuf, PathBuf)>) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut grouped: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for (source, target) in matched_files {
        if let Some(source_paths) = grouped.get_mut(&target) {
            source_paths.push(source);
            continue;
        }
        let source_paths = vec![source];
        grouped.insert(target.clone(), source_paths);
    }
    grouped
}


fn concat_files(source_paths: Vec<PathBuf>, target_path: PathBuf) -> io::Result<()> {
    println!("Concatenating into {}:", target_path.to_string_lossy());
    let mut target = try!(File::create(target_path));
    let mut is_first = true;
    for path in source_paths {
        println!("  {}", path.to_string_lossy());
        let mut source = try!(File::open(path));
        let mut contents: Vec<u8> = Vec::new();
        try!(source.read_to_end(&mut contents));

        if !is_first {
            try!(target.write_all(b"\n"));
        }
        try!(target.write_all(&contents));

        is_first = false;
    }
    Ok(())
}


fn ensure_dir_exists(dir_string: &str) -> io::Result<&Path> {
    let path = Path::new(dir_string);
    if !path.exists() {
        try!(create_dir(path));
    }
    Ok(&path)
}
