use std::fs::{self, File};
use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use cpio::NewcReader;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, Progress,
};

pub fn extract_cpio(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;
    let mut reader = NewcReader::new(file)
        .with_context(|| format!("Failed to read cpio archive: {}", file_path.display()))?;
    let mut entry_count = 0u64;
    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(Progress::spinner);

    loop {
        let entry = reader.entry().clone();
        if entry.is_trailer() {
            break;
        }
        entry_count += 1;

        let path = Path::new(entry.name());
        let path = match strip_path_components(path, options.strip_components) {
            Some(p) => p,
            None => {
                reader = NewcReader::new(reader.finish()?)?;
                continue;
            }
        };

        if !should_extract(&path, &options.patterns) {
            reader = NewcReader::new(reader.finish()?)?;
            continue;
        }

        let size = entry.file_size() as u64;
        let mode = entry.mode() & 0o7777;
        let is_dir = (entry.mode() & 0o170000) == 0o040000;

        if options.list {
            print_entry(&EntryInfo {
                path: path.clone(),
                size,
                is_dir,
                mode: Some(mode),
            });
            reader = NewcReader::new(reader.finish()?)?;
            continue;
        }

        let entry_path = safe_output_path(&options.output_dir, &path)
            .with_context(|| format!("Unsafe entry path: {}", path.display()))?;

        let target_path =
            match resolve_conflict(&entry_path, options.on_exists, &options.rename_suffix)
                .with_context(|| format!("Conflict handling failed for {}", entry_path.display()))?
            {
                Some(p) => p,
                None => {
                    reader = NewcReader::new(reader.finish()?)?;
                    continue;
                }
            };

        if is_dir {
            fs::create_dir_all(&target_path)?;
            reader = NewcReader::new(reader.finish()?)?;
        } else {
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

            let mut target_file = File::create(&target_path)?;
            io::copy(&mut reader, &mut target_file)?;
            extracted_count += 1;
            if let Some(ref pb) = progress {
                pb.inc(1);
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = PermissionsExt::from_mode(mode);
                if let Err(e) = fs::set_permissions(&target_path, permissions) {
                    eprintln!("Warning: Could not set permissions: {e}");
                }
            }

            reader = NewcReader::new(reader.finish()?)?;
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
