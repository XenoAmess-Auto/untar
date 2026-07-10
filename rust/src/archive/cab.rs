use std::fs::{self, File};
use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use cab::Cabinet;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, LimitedWriter, Progress,
};

pub fn extract_cab(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;
    let mut cabinet = Cabinet::new(file)
        .with_context(|| format!("Failed to read CAB archive: {}", file_path.display()))?;

    let entries: Vec<(String, u64)> = cabinet
        .folder_entries()
        .flat_map(|folder| folder.file_entries())
        .map(|file_entry| {
            (
                file_entry.name().to_string(),
                file_entry.uncompressed_size() as u64,
            )
        })
        .collect();

    let mut entry_count = 0u64;
    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(Progress::spinner);

    for (name, size) in entries {
        entry_count += 1;
        let path_name = name.replace('\\', "/");
        let path = Path::new(&path_name);

        let path = match strip_path_components(path, options.strip_components) {
            Some(p) => p,
            None => continue,
        };

        if !should_extract(&path, &options.patterns) {
            continue;
        }

        if !options.list {
            options.limits.record_entry(size)?;
        }

        if options.list {
            print_entry(&EntryInfo {
                path: path.clone(),
                size,
                is_dir: false,
                mode: None,
            });
            continue;
        }

        let entry_path = safe_output_path(&options.output_dir, &path)
            .with_context(|| format!("Unsafe entry path: {}", path.display()))?;

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

        if let Some(ref pb) = progress {
            pb.set_message(format!(
                "[{:?}] {} ({})",
                entry_count,
                path.display(),
                format_size(size)
            ));
        }

        let mut reader = cabinet
            .read_file(&name)
            .with_context(|| format!("Failed to read CAB entry: {name}"))?;
        let target_file = File::create(&target_path)?;
        let mut limited = LimitedWriter::new(target_file, options.limits.clone());
        io::copy(&mut reader, &mut limited)?;
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
