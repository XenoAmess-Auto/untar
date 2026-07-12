use std::fs;
use std::io::{self, Read, Seek};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use sevenz_rust2::{ArchiveReader, Password};

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, LimitedWriter, Progress,
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
    let progress = (!options.quiet && !options.list).then(|| Progress::bar(total_count as u64));

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

            let size = entry.size();
            let packed_size = entry.compressed_size;
            if let Err(e) = options.limits.record_entry(size) {
                return Err(map_err(e));
            }
            if let Err(e) = options.limits.check_ratio(packed_size, size) {
                return Err(map_err(e));
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
                if let Some(ref pb) = progress {
                    pb.set_message(format!("[{}] {}", extracted_count + 1, name));
                }
                fs::create_dir_all(&entry_path).map_err(map_err)?;
                if let Some(ref pb) = progress {
                    pb.inc(1);
                }
                return Ok(true);
            }

            let target_path = match resolve_conflict(
                &entry_path,
                options.on_exists,
                &options.rename_suffix,
                options.is_tty,
            )
            .with_context(|| format!("Conflict handling failed for {}", entry_path.display()))
            .map_err(map_err)?
            {
                Some(p) => p,
                None => {
                    io::copy(reader, &mut io::sink()).map_err(map_err)?;
                    if let Some(ref pb) = progress {
                        pb.inc(1);
                    }
                    return Ok(true);
                }
            };

            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(map_err)?;
                }
            }

            if let Some(ref pb) = progress {
                pb.set_message(format!(
                    "[{}] {} ({})",
                    extracted_count + 1,
                    name,
                    format_size(entry.size())
                ));
            }

            let file = fs::File::create(&target_path).map_err(map_err)?;
            let mut limited = LimitedWriter::new(file, options.limits.clone());
            io::copy(reader, &mut limited).map_err(map_err)?;
            extracted_count += 1;
            if let Some(ref pb) = progress {
                pb.inc(1);
            }

            Ok(true)
        })
        .map_err(|e| anyhow!("7z extraction failed: {e}"))?;

    if !options.quiet && !options.list {
        if let Some(ref pb) = progress {
            pb.finish(format!("Extracted {extracted_count} files"));
        } else {
            println!("Extracted files: {extracted_count}");
        }
    }

    Ok(())
}

fn map_err<E: std::fmt::Display>(e: E) -> sevenz_rust2::Error {
    sevenz_rust2::Error::Other(e.to_string().into())
}
