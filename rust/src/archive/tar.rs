use std::fs::{self, File};
use std::io::{self, BufReader, Read};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use anyhow::{Context, Result};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use tar::Archive;
use xz2::read::XzDecoder;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions,
};

pub fn extract_tar_gz<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    extract_tar_reader(GzDecoder::new(reader), options)
}

pub fn extract_tar_xz<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    extract_tar_reader(XzDecoder::new(reader), options)
}

pub fn extract_tar_bz2<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    extract_tar_reader(BzDecoder::new(reader), options)
}

pub fn extract_tar<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    extract_tar_reader(BufReader::new(reader), options)
}

fn extract_tar_reader<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    let mut archive = Archive::new(reader);
    let mut entry_count = 0u64;
    let mut extracted_count = 0u64;

    for entry in archive.entries()? {
        let mut entry = entry?;
        entry_count += 1;

        let path = entry.path()?.into_owned();

        let path = match strip_path_components(&path, options.strip_components) {
            Some(p) => p,
            None => continue,
        };

        if !should_extract(&path, &options.patterns) {
            continue;
        }

        let size = entry.size();
        let is_dir = entry.header().entry_type() == tar::EntryType::Directory;
        let mode = entry.header().mode().ok().filter(|m| *m != 0);

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
            .with_context(|| format!("Unsafe entry path: {}", path.display()))?;

        if is_dir {
            if let Some(parent) = entry_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            if !options.quiet {
                println!("[{:?}] {}", entry_count, path.display());
            }
            fs::create_dir_all(&entry_path)?;
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

        if !options.quiet {
            println!(
                "[{:?}] {} ({})",
                entry_count,
                path.display(),
                format_size(size)
            );
        }

        let mut file = File::create(&target_path)?;
        io::copy(&mut entry, &mut file)?;
        extracted_count += 1;

        // Preserve file permissions (Unix mode).
        if let Some(m) = mode {
            #[cfg(unix)]
            {
                let permissions = PermissionsExt::from_mode(m);
                if let Err(e) = fs::set_permissions(&target_path, permissions) {
                    eprintln!("Warning: Could not set permissions: {e}");
                }
            }
        }
    }

    if !options.quiet && !options.list {
        println!("Total files: {extracted_count}");
    }

    Ok(())
}
