use std::fs::{self, File};
use std::io::{self, BufReader, Read};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use anyhow::{Context, Result};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use tar::Archive;
use xz2::read::XzDecoder;

use crate::extract::{format_size, safe_output_path, ExtractOptions};

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

    for entry in archive.entries()? {
        let mut entry = entry?;
        entry_count += 1;

        let path = entry.path()?.into_owned();
        let entry_path = safe_output_path(&options.output_dir, &path)
            .with_context(|| format!("Unsafe entry path: {}", path.display()))?;
        let size = entry.size();

        if let Some(parent) = entry_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if entry.header().entry_type() == tar::EntryType::Directory {
            if !options.quiet {
                println!("[{:?}] {}", entry_count, path.display());
            }
            fs::create_dir_all(&entry_path)?;
        } else {
            if !options.quiet {
                println!(
                    "[{:?}] {} ({})",
                    entry_count,
                    path.display(),
                    format_size(size)
                );
            }

            let mut file = File::create(&entry_path)?;
            io::copy(&mut entry, &mut file)?;

            // Preserve file permissions (Unix mode).
            if let Some(mode) = entry.header().mode().ok().filter(|m| *m != 0) {
                #[cfg(unix)]
                {
                    let permissions = PermissionsExt::from_mode(mode);
                    if let Err(e) = fs::set_permissions(&entry_path, permissions) {
                        eprintln!("Warning: Could not set permissions: {e}");
                    }
                }
            }
        }
    }

    if !options.quiet {
        println!("Total files: {entry_count}");
    }

    Ok(())
}
