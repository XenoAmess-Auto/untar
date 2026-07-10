use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use anyhow::{Context, Result};
use brotli_decompressor::Decompressor;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use lz4_flex::frame::FrameDecoder;
use tar::Archive;
use xz2::read::XzDecoder;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, Progress,
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

pub fn extract_tar_lzma<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    lzma_rs::lzma_decompress(&mut BufReader::new(reader), temp.as_file_mut())?;
    let file = temp.as_file_mut();
    file.rewind()?;
    extract_tar_reader(file, options)
}

pub fn extract_tar_zst<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    extract_tar_reader(ruzstd::decoding::StreamingDecoder::new(reader)?, options)
}

pub fn extract_tar_lz4<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    extract_tar_reader(FrameDecoder::new(reader), options)
}

pub fn extract_tar_br<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    extract_tar_reader(Decompressor::new(reader, 4096), options)
}

pub fn extract_tar<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    extract_tar_reader(BufReader::new(reader), options)
}

fn extract_tar_reader<R: Read>(reader: R, options: &ExtractOptions) -> Result<()> {
    let mut archive = Archive::new(reader);
    let mut entry_count = 0u64;
    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(Progress::spinner);

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
            if let Some(ref pb) = progress {
                pb.set_message(format!("[{:?}] {}", entry_count, path.display()));
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

        if let Some(ref pb) = progress {
            pb.set_message(format!(
                "[{:?}] {} ({})",
                entry_count,
                path.display(),
                format_size(size)
            ));
        }

        let mut file = File::create(&target_path)?;
        io::copy(&mut entry, &mut file)?;
        extracted_count += 1;
        if let Some(ref pb) = progress {
            pb.inc(1);
        }

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
        if let Some(ref pb) = progress {
            pb.finish(format!("Extracted {extracted_count} files"));
        } else {
            println!("Extracted files: {extracted_count}");
        }
    }

    Ok(())
}
