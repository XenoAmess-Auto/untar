use std::fs::{self, File};
use std::io::{self, Read, Seek};
use std::path::Path;

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions,
};

pub fn extract_zip<R: Read + Seek>(reader: R, options: &ExtractOptions) -> Result<()> {
    let mut archive = ZipArchive::new(reader)?;
    let total_count = archive.len();
    let mut extracted_count = 0u64;

    if !options.quiet && !options.list {
        println!("Total files: {total_count}");
    }

    for i in 0..total_count {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        let size = entry.size();

        let path = match strip_path_components(Path::new(&name), options.strip_components) {
            Some(p) => p,
            None => continue,
        };

        if !should_extract(&path, &options.patterns) {
            continue;
        }

        let is_dir = entry.is_dir();
        let mode = None; // zip does not expose Unix mode easily without extra features

        if options.list {
            print_entry(&EntryInfo {
                path: path.clone(),
                size,
                is_dir,
                mode,
            });
            continue;
        }

        let entry_path = safe_output_path(&options.output_dir, &path)
            .with_context(|| format!("Unsafe entry path: {name}"))?;

        if is_dir {
            if let Some(parent) = entry_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            fs::create_dir_all(&entry_path)?;
            continue;
        }

        let target_path =
            match resolve_conflict(&entry_path, options.on_exists, &options.rename_suffix)
                .with_context(|| format!("Conflict handling failed for {}", entry_path.display()))?
            {
                Some(p) => p,
                None => continue,
            };

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if !options.quiet {
            println!("[{:?}] {} ({})", i + 1, name, format_size(size));
        }

        let mut file = File::create(&target_path)?;
        io::copy(&mut entry, &mut file)?;
        extracted_count += 1;
    }

    if !options.quiet && !options.list {
        println!("Extracted files: {extracted_count}");
    }

    Ok(())
}
