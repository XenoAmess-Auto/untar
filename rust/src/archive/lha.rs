use std::fs::{self, File};
use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use delharc::parse_file;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, LimitedWriter, Progress,
};

pub fn extract_lha(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let mut reader = parse_file(file_path)
        .with_context(|| format!("Failed to read LHA archive: {}", file_path.display()))?;
    let mut entry_count = 0u64;
    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(Progress::spinner);

    loop {
        let header = reader.header();
        let path = header.parse_pathname();
        let is_dir = header.is_directory();
        let size = header.original_size;
        entry_count += 1;

        if !options.list {
            options.limits.record_entry(size)?;
        }

        let path = match strip_path_components(&path, options.strip_components) {
            Some(p) => p,
            None => {
                if !reader.next_file()? {
                    break;
                }
                continue;
            }
        };

        if !should_extract(&path, &options.patterns) {
            if !reader.next_file()? {
                break;
            }
            continue;
        }

        if options.list {
            print_entry(&EntryInfo {
                path: path.clone(),
                size,
                is_dir,
                mode: None,
            });
            if !reader.next_file()? {
                break;
            }
            continue;
        }

        let entry_path = safe_output_path(&options.output_dir, &path)
            .with_context(|| format!("Unsafe entry path: {}", path.display()))?;

        let target_path = match resolve_conflict(
            &entry_path,
            options.on_exists,
            &options.rename_suffix,
            options.is_tty,
        )
        .with_context(|| format!("Conflict handling failed for {}", entry_path.display()))?
        {
            Some(p) => p,
            None => {
                if !reader.next_file()? {
                    break;
                }
                continue;
            }
        };

        if is_dir {
            fs::create_dir_all(&target_path)?;
            if !reader.next_file()? {
                break;
            }
            continue;
        }

        if !reader.is_decoder_supported() {
            if !reader.next_file()? {
                break;
            }
            continue;
        }

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if let Some(ref pb) = progress {
            pb.set_message(format!(
                "[{:?}] {} ({})",
                entry_count,
                path.display(),
                format_size(size)
            ));
        }

        let target_file = File::create(&target_path)?;
        let mut limited = LimitedWriter::new(target_file, options.limits.clone());
        io::copy(&mut reader, &mut limited)?;
        extracted_count += 1;
        if let Some(ref pb) = progress {
            pb.inc(1);
        }

        if !reader.next_file()? {
            break;
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
