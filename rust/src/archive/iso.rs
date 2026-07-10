use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use gpt_disk_io::BlockIoAdapter;
use gpt_disk_types::BlockSize;
use iso9660::directory::iterator::DirectoryIterator;
use iso9660::file::reader::FileReader;
use iso9660::mount;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, Progress,
};

fn iso_error(e: iso9660::Iso9660Error) -> anyhow::Error {
    anyhow!("ISO9660 error: {e}")
}

pub fn extract_iso(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;
    let block_size = BlockSize::new(2048).context("Invalid ISO block size")?;
    let mut block_io = BlockIoAdapter::new(file, block_size);
    let volume = mount(&mut block_io, 0)
        .map_err(iso_error)
        .with_context(|| format!("Failed to mount ISO image: {}", file_path.display()))?;

    let mut dirs = VecDeque::new();
    dirs.push_back((
        PathBuf::new(),
        volume.root_extent_lba,
        volume.root_extent_len,
    ));

    let mut entry_count = 0u64;
    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(Progress::spinner);

    while let Some((dir_path, lba, len)) = dirs.pop_front() {
        let mut entries = Vec::new();
        {
            let iter = DirectoryIterator::new(&mut block_io, lba, len);
            for entry in iter {
                let entry = entry.map_err(iso_error)?;
                if entry.name == "." || entry.name == ".." {
                    continue;
                }
                entries.push(entry);
            }
        }

        for entry in entries {
            entry_count += 1;
            let path = dir_path.join(&entry.name);

            let path = match strip_path_components(&path, options.strip_components) {
                Some(p) => p,
                None => continue,
            };

            if !should_extract(&path, &options.patterns) {
                continue;
            }

            let size = entry.size;
            let is_dir = entry.flags.directory;

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
                    .with_context(|| {
                        format!("Conflict handling failed for {}", entry_path.display())
                    })? {
                    Some(p) => p,
                    None => continue,
                };

            if is_dir {
                fs::create_dir_all(&target_path)?;
                dirs.push_back((path, entry.extent_lba, entry.data_length));
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

                let mut reader = FileReader::new(&mut block_io, entry.clone());
                let mut target_file = File::create(&target_path)?;

                let mut buf = [0u8; 8192];
                loop {
                    let n = reader.read(&mut buf).map_err(iso_error)?;
                    if n == 0 {
                        break;
                    }
                    target_file.write_all(&buf[..n])?;
                }

                extracted_count += 1;
                if let Some(ref pb) = progress {
                    pb.inc(1);
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
