use std::fs::{self, File};
use std::io::{self, Read, Seek};
use std::path::Path;

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, Progress,
};

pub fn extract_zip<R: Read + Seek>(reader: R, options: &ExtractOptions) -> Result<()> {
    let mut archive = ZipArchive::new(reader)?;
    let total_count = archive.len();
    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(|| Progress::bar(total_count as u64));

    if !options.quiet && !options.list {
        println!("Total files: {total_count}");
    }

    for i in 0..total_count {
        let mut entry = if let Some(password) = &options.password {
            archive
                .by_index_decrypt(i, password.as_bytes())
                .map_err(|e| anyhow::anyhow!("Failed to decrypt zip entry: {e}"))?
        } else {
            archive.by_index(i)?
        };
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
            if let Some(ref pb) = progress {
                pb.set_message(format!("[{}] {}", i + 1, name));
            }
            fs::create_dir_all(&entry_path)?;
            if let Some(ref pb) = progress {
                pb.inc(1);
            }
            continue;
        }

        let target_path =
            match resolve_conflict(&entry_path, options.on_exists, &options.rename_suffix)
                .with_context(|| format!("Conflict handling failed for {}", entry_path.display()))?
            {
                Some(p) => p,
                None => {
                    if let Some(ref pb) = progress {
                        pb.inc(1);
                    }
                    continue;
                }
            };

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if let Some(ref pb) = progress {
            pb.set_message(format!("[{}] {} ({})", i + 1, name, format_size(size)));
        }

        let mut file = File::create(&target_path)?;
        io::copy(&mut entry, &mut file)?;
        extracted_count += 1;
        if let Some(ref pb) = progress {
            pb.inc(1);
        }
    }

    if !options.quiet && !options.list {
        if let Some(ref pb) = progress {
            pb.finish(format!("Extracted {extracted_count} files"));
        } else {
            println!("Extracted files: {extracted_count}");
        }
    }

    Ok(())
}
