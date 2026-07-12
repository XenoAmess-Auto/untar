use std::fs::{self, File};
use std::io::{self, Read, Seek};
use std::path::Path;

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, LimitedWriter, Progress,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn set_unix_mode(path: &Path, mode: u32) {
    #[cfg(unix)]
    {
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
}

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
        let mode = entry.unix_mode();

        if !options.list {
            options.limits.record_entry(size)?;
            let compressed = entry.compressed_size();
            options.limits.check_ratio(compressed, size)?;
        }

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
            if let Some(m) = mode {
                set_unix_mode(&entry_path, m);
            }
            if let Some(ref pb) = progress {
                pb.inc(1);
            }
            continue;
        }

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

        let file = File::create(&target_path)?;
        let mut limited = LimitedWriter::new(file, options.limits.clone());
        io::copy(&mut entry, &mut limited)?;
        if let Some(m) = mode {
            set_unix_mode(&target_path, m);
        }
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
