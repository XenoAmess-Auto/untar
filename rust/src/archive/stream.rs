use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use brotli_decompressor::Decompressor;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use lz4_flex::frame::FrameDecoder;
use ruzstd::decoding::StreamingDecoder;
use xz2::read::XzDecoder;

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, Progress,
};

pub fn extract_stream(
    file: File,
    file_path: &Path,
    options: &ExtractOptions,
    ext: &str,
) -> Result<()> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output");

    let stripped = file_name.strip_suffix(ext).unwrap_or(file_name);
    let path = Path::new(stripped);
    let path = match strip_path_components(path, options.strip_components) {
        Some(p) => p,
        None => return Ok(()),
    };

    if !should_extract(&path, &options.patterns) {
        return Ok(());
    }

    let entry_path = safe_output_path(&options.output_dir, &path)
        .with_context(|| format!("Unsafe output path: {}", path.display()))?;

    if options.list {
        print_entry(&EntryInfo {
            path: entry_path,
            size: 0,
            is_dir: false,
            mode: None,
        });
        return Ok(());
    }

    let target_path = match resolve_conflict(&entry_path, options.on_exists, &options.rename_suffix)
        .with_context(|| format!("Conflict handling failed for {}", entry_path.display()))?
    {
        Some(p) => p,
        None => return Ok(()),
    };

    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let progress = (!options.quiet && !options.list).then(Progress::spinner);
    if let Some(ref pb) = progress {
        pb.set_message(format!("Decompressing {}", file_path.display()));
    }

    let mut reader = BufReader::new(file);
    let mut target_file = File::create(&target_path)?;

    match ext {
        ".gz" => {
            let mut decoder = GzDecoder::new(reader);
            io::copy(&mut decoder, &mut target_file)?
        }
        ".bz2" => {
            let mut decoder = BzDecoder::new(reader);
            io::copy(&mut decoder, &mut target_file)?
        }
        ".xz" => {
            let mut decoder = XzDecoder::new(reader);
            io::copy(&mut decoder, &mut target_file)?
        }
        ".zst" => {
            let mut decoder = StreamingDecoder::new(reader)?;
            io::copy(&mut decoder, &mut target_file)?
        }
        ".lz4" => {
            let mut decoder = FrameDecoder::new(reader);
            io::copy(&mut decoder, &mut target_file)?
        }
        ".br" => {
            let mut decoder = Decompressor::new(reader, 4096);
            io::copy(&mut decoder, &mut target_file)?
        }
        ".lzma" => {
            lzma_rs::lzma_decompress(&mut reader, &mut target_file)?;
            0
        }
        _ => return Err(anyhow!("Unsupported stream extension: {ext}")),
    };

    if let Some(ref pb) = progress {
        let size = target_file.metadata().map(|m| m.len()).unwrap_or(0);
        pb.finish(format!(
            "Decompressed to {} ({})",
            target_path.display(),
            format_size(size)
        ));
    }

    Ok(())
}
