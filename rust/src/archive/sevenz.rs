use std::fs;
use std::io::{self, Read, Seek};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use sevenz_rust2::{ArchiveReader, Password};

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions,
};

pub fn extract_7z<R: Read + Seek>(reader: R, options: &ExtractOptions) -> Result<()> {
    let password = options
        .password
        .as_deref()
        .map(Password::from)
        .unwrap_or_else(|| Password::from(""));
    let mut archive_reader = ArchiveReader::new(reader, password)?;
    let archive = archive_reader.archive().clone();

    let total_count = archive.files.len();
    let mut extracted_count = 0u64;

    if !options.quiet && !options.list {
        println!("Total files: {total_count}");
    }

    if options.list {
        for entry in &archive.files {
            if entry.is_anti_item() {
                continue;
            }
            let path =
                match strip_path_components(Path::new(entry.name()), options.strip_components) {
                    Some(p) => p,
                    None => continue,
                };
            if !should_extract(&path, &options.patterns) {
                continue;
            }

            let mode = entry
                .has_windows_attributes
                .then_some(entry.windows_attributes());

            print_entry(&EntryInfo {
                path,
                size: entry.size(),
                is_dir: entry.is_directory(),
                mode,
            });
        }
        return Ok(());
    }

    archive_reader
        .for_each_entries(|entry, reader| {
            let name = entry.name();
            if entry.is_anti_item() {
                return Ok(true);
            }
            let path = match strip_path_components(Path::new(name), options.strip_components) {
                Some(p) => p,
                None => {
                    io::copy(reader, &mut io::sink()).map_err(map_err)?;
                    return Ok(true);
                }
            };
            if !should_extract(&path, &options.patterns) {
                io::copy(reader, &mut io::sink()).map_err(map_err)?;
                return Ok(true);
            }

            let entry_path = safe_output_path(&options.output_dir, &path)
                .with_context(|| format!("Unsafe entry path: {name}"))
                .map_err(map_err)?;

            if entry.is_directory() {
                if let Some(parent) = entry_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(map_err)?;
                    }
                }
                fs::create_dir_all(&entry_path).map_err(map_err)?;
                return Ok(true);
            }

            let target_path =
                match resolve_conflict(&entry_path, options.on_exists, &options.rename_suffix)
                    .with_context(|| {
                        format!("Conflict handling failed for {}", entry_path.display())
                    })
                    .map_err(map_err)?
                {
                    Some(p) => p,
                    None => {
                        io::copy(reader, &mut io::sink()).map_err(map_err)?;
                        return Ok(true);
                    }
                };

            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(map_err)?;
                }
            }

            if !options.quiet {
                println!(
                    "[{}] {} ({})",
                    extracted_count + 1,
                    name,
                    format_size(entry.size())
                );
            }

            let mut file = fs::File::create(&target_path).map_err(map_err)?;
            io::copy(reader, &mut file).map_err(map_err)?;
            extracted_count += 1;

            Ok(true)
        })
        .map_err(|e| anyhow!("7z extraction failed: {e}"))?;

    if !options.quiet && !options.list {
        println!("Extracted files: {extracted_count}");
    }

    Ok(())
}

fn map_err<E: std::fmt::Display>(e: E) -> sevenz_rust2::Error {
    sevenz_rust2::Error::Other(e.to_string().into())
}
