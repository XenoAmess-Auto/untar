use std::fs::{self, File};
use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use ar::Archive;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, LimitedWriter, Progress,
};

pub fn extract_ar(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;
    let mut archive = Archive::new(file);
    let mut entry_count = 0u64;
    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(Progress::spinner);

    while let Some(entry_result) = archive.next_entry() {
        let mut entry = entry_result?;
        entry_count += 1;

        let name = entry.header().identifier();
        let name = String::from_utf8_lossy(name);
        let path = Path::new(&*name);

        let path = match strip_path_components(path, options.strip_components) {
            Some(p) => p,
            None => continue,
        };

        if !should_extract(&path, &options.patterns) {
            continue;
        }

        let size = entry.header().size();
        let mode = entry.header().mode() & 0o7777;

        if !options.list {
            options.limits.record_entry(size)?;
        }

        if options.list {
            print_entry(&EntryInfo {
                path: path.clone(),
                size,
                is_dir: false,
                mode: Some(mode),
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

        let file = File::create(&target_path)?;
        let mut limited = LimitedWriter::new(file, options.limits.clone());
        io::copy(&mut entry, &mut limited)?;
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
