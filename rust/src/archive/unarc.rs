use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use unarc_rs::unified::{ArchiveEntry, ArchiveFormat, UnifiedArchive};
use unarc_rs::ArchiveError;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, LimitedWriter, Progress,
};

pub fn extract_tarz(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    extract_unarc(file_path, options, ArchiveFormat::TarZ)
}

pub fn extract_z(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    extract_unarc(file_path, options, ArchiveFormat::Z)
}

pub fn extract_ace(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    extract_unarc(file_path, options, ArchiveFormat::Ace)
}

pub fn extract_arc(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    extract_unarc(file_path, options, ArchiveFormat::Arc)
}

pub fn extract_zoo(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    extract_unarc(file_path, options, ArchiveFormat::Zoo)
}

fn extract_unarc(file_path: &Path, options: &ExtractOptions, format: ArchiveFormat) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;
    let mut archive = UnifiedArchive::open_with_format(file, format)
        .with_context(|| format!("Failed to open archive: {}", file_path.display()))?;

    if format == ArchiveFormat::Z {
        if let Some(stem) = file_path.file_stem().and_then(|s| s.to_str()) {
            archive.set_single_file_name(stem.to_string());
        }
    }

    let progress = (!options.quiet && !options.list).then(Progress::spinner);
    let mut entry_count = 0u64;
    let mut extracted_count = 0u64;

    loop {
        let entry = match archive.next_entry() {
            Ok(Some(e)) => e,
            Ok(None) => break,
            Err(ArchiveError::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(anyhow!("Failed to read entry: {e}")),
        };

        entry_count += 1;

        let data = archive
            .read(&entry)
            .map_err(|e| anyhow!("Failed to read entry data: {e}"))?;

        if extract_entry(&entry, options, progress.as_ref(), &data)? {
            extracted_count += 1;
        }
    }

    if let Some(ref pb) = progress {
        pb.finish(format!(
            "Extracted {extracted_count} of {entry_count} entries"
        ));
    }

    Ok(())
}

fn extract_entry(
    entry: &ArchiveEntry,
    options: &ExtractOptions,
    progress: Option<&Progress>,
    data: &[u8],
) -> Result<bool> {
    let name = entry.name().replace('\\', "/");
    let path = Path::new(&name);

    let path = match strip_path_components(path, options.strip_components) {
        Some(p) => p,
        None => return Ok(false),
    };

    if !should_extract(&path, &options.patterns) {
        return Ok(false);
    }

    let entry_path = safe_output_path(&options.output_dir, &path)
        .with_context(|| format!("Unsafe output path: {}", path.display()))?;

    if options.list {
        print_entry(&EntryInfo {
            path: entry_path,
            size: entry.original_size(),
            is_dir: name.ends_with('/'),
            mode: None,
        });
        return Ok(false);
    }

    options.limits.record_entry(entry.original_size())?;

    let target_path = match resolve_conflict(&entry_path, options.on_exists, &options.rename_suffix)
        .with_context(|| format!("Conflict handling failed for {}", entry_path.display()))?
    {
        Some(p) => p,
        None => return Ok(false),
    };

    if name.ends_with('/') {
        fs::create_dir_all(&target_path)?;
        return Ok(true);
    }

    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    if let Some(pb) = progress {
        pb.set_message(format!("Extracting {}", entry_path.display()));
    }

    let mut target = File::create(&target_path)?;
    let mut limited = LimitedWriter::new(&mut target, options.limits.clone());
    limited.write_all(data)?;
    drop(limited);

    if let Some(pb) = progress {
        let size = target.metadata().map(|m| m.len()).unwrap_or(0);
        pb.finish(format!(
            "Extracted {} ({})",
            target_path.display(),
            format_size(size)
        ));
    }

    Ok(true)
}
