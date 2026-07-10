use std::fs::{self, File};
use std::path::Path;

use anyhow::{Context, Result};
use xara::{XarArchive, XarFileType};

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, Progress,
};

pub fn extract_xar(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;
    let mut archive = XarArchive::open(file)
        .with_context(|| format!("Failed to read XAR archive: {}", file_path.display()))?;

    let files = archive.files().to_vec();
    let mut entry_count = 0u64;
    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(Progress::spinner);

    for file in files {
        entry_count += 1;
        let path = Path::new(&file.path);

        let path = match strip_path_components(path, options.strip_components) {
            Some(p) => p,
            None => continue,
        };

        if !should_extract(&path, &options.patterns) {
            continue;
        }

        let size = file.data.as_ref().map(|d| d.size).unwrap_or(0);
        let is_dir = file.file_type == XarFileType::Directory;

        if options.list {
            print_entry(&EntryInfo {
                path: path.clone(),
                size,
                is_dir,
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

        if is_dir {
            fs::create_dir_all(&target_path)?;
        } else if file.file_type == XarFileType::Symlink {
            continue;
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
            archive
                .read_file_to(&file, &mut target_file)
                .with_context(|| format!("Failed to read XAR entry: {}", path.display()))?;
            extracted_count += 1;
            if let Some(ref pb) = progress {
                pb.inc(1);
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
