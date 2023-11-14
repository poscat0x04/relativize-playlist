use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{rename, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tempfile::tempdir_in;
use walkdir::WalkDir;

use crate::cli::Args;

mod cli;

type DirMap = HashMap<Vec<OsString>, PathBuf>;

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let path = &args.path;
    let ignore_exts = !args.strict_extension;

    let playlist = File::open(path).context("Failed to open playlist")?;
    let file_name = path.file_name().with_context(|| {
        format!(
            "The supplied path \"{}\" does not point to a file",
            path.display()
        )
    })?;
    let reader = BufReader::new(playlist);

    let parent = path.parent().expect("Impossible: file with no parent dir");
    let dir_map = index_dir(parent, args.depth, ignore_exts)?;

    let tmp_dir = tempdir_in(parent)?;
    let tmp_file = File::create(tmp_dir.path().join(file_name))?;
    let writer = BufWriter::new(tmp_file);

    relativize(&dir_map, reader, writer, args.depth, ignore_exts)?;
    rename(tmp_dir.path().join(file_name), path).context("Failed to rename file")?;

    tmp_dir.close().context("Failed to delete tmp dir")?;
    Ok(())
}

fn get_last_n_segments(mut p: PathBuf, mut n: u8, ignore_exts: bool) -> Option<Vec<OsString>> {
    let mut segments = Vec::new();

    if ignore_exts {
        p.set_extension("");
    }

    while n > 0 {
        let segment = p.file_name()?.to_os_string();
        segments.push(segment);
        assert!(p.pop());
        n -= 1;
    }

    Some(segments)
}

fn index_dir(p: &Path, depth: u8, ignore_exts: bool) -> Result<DirMap> {
    let walker = WalkDir::new(p).follow_links(true);
    let mut map = HashMap::new();
    for entry in walker {
        let entry = entry.context("Failed to read directory content")?;
        if entry.file_type().is_file() {
            let rel_path = entry
                .path()
                .strip_prefix(p)
                .expect("Imposssible: failed to strip prefix")
                .to_path_buf();
            if let Some(key) = get_last_n_segments(rel_path.clone(), depth, ignore_exts) {
                map.insert(key, rel_path);
            }
        }
    }
    Ok(map)
}

fn relativize(
    dir_map: &DirMap,
    mut reader: impl BufRead,
    mut writer: impl Write,
    depth: u8,
    ignore_exts: bool,
) -> Result<()> {
    let mut line_buf = String::new();
    while reader
        .read_line(&mut line_buf)
        .context("Failed while reading file content")?
        > 0
    {
        let line = line_buf.trim();
        if line.starts_with('#') {
            // write comments as-is
            writeln!(&mut writer, "{line}").context("Failed while writing to file")?;
        } else {
            let path = PathBuf::from(line);
            if let Some(segments) = get_last_n_segments(path, depth, ignore_exts) {
                if let Some(rel_path) = dir_map.get(&segments) {
                    writeln!(&mut writer, "{}", rel_path.display())
                        .context("Failed while writing to file")?;
                }
            }
        };
        line_buf.clear();
    }
    Ok(())
}
