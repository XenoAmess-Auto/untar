use std::fs::{self, File};
use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use rpm::FileType;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, Progress,
};

pub fn extract_rpm(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let package = rpm::Package::open(file_path)
        .with_context(|| format!("Failed to open RPM package: {}", file_path.display()))?;

    let files = package
        .files()
        .with_context(|| format!("Failed to read RPM payload: {}", file_path.display()))?;
    let total_count = files.len();
    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(|| Progress::bar(total_count as u64));

    if !options.quiet && !options.list {
        println!("Total files: {total_count}");
    }

    for file_result in files {
        let file = file_result?;
        let metadata = file.metadata;
        let path = {
            let p = metadata.path();
            let s = p.to_string_lossy();
            let s = s.trim_start_matches('/');
            std::path::PathBuf::from(s)
        };

        let path = match strip_path_components(&path, options.strip_components) {
            Some(p) => p,
            None => continue,
        };

        if !should_extract(&path, &options.patterns) {
            continue;
        }

        let size = file.content.len() as u64;
        let mode = metadata.permissions() as u32;
        let is_dir = metadata.file_type() == FileType::Dir;
        let is_symlink = metadata.file_type() == FileType::SymbolicLink;

        if options.list {
            print_entry(&EntryInfo {
                path: path.clone(),
                size,
                is_dir,
                mode: Some(mode),
            });
            continue;
        }

        let entry_path = safe_output_path(&options.output_dir, &path)
            .with_context(|| format!("Unsafe entry path: {}", path.display()))?;

        if is_dir {
            fs::create_dir_all(&entry_path)?;
            continue;
        }

        if is_symlink {
            #[cfg(unix)]
            {
                if let Some(target) = metadata.linkto() {
                    if let Some(parent) = entry_path.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }
                    std::os::unix::fs::symlink(target, &entry_path)?;
                }
            }
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

        if let Some(ref pb) = progress {
            pb.set_message(format!(
                "[{}] {} ({})",
                extracted_count + 1,
                path.display(),
                format_size(size)
            ));
        }

        let mut target_file = File::create(&target_path)?;
        io::copy(&mut file.content.as_slice(), &mut target_file)?;
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
