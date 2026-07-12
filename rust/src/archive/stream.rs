use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use brotli_decompressor::Decompressor;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use liblzma::read::XzDecoder;
use lz4_flex::frame::FrameDecoder;
use lzma_rust2::LzipReader;
use ruzstd::decoding::StreamingDecoder;

use crate::archive::lzo;
use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, LimitedWriter, Progress,
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

    options.limits.record_entry(0)?;

    let target_path = match resolve_conflict(
        &entry_path,
        options.on_exists,
        &options.rename_suffix,
        options.is_tty,
    )
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
    let target_file = File::create(&target_path)?;
    let mut limited = LimitedWriter::new(target_file, options.limits.clone());

    match ext {
        ".gz" => {
            let mut decoder = GzDecoder::new(reader);
            io::copy(&mut decoder, &mut limited)?
        }
        ".bz2" => {
            let mut decoder = BzDecoder::new(reader);
            io::copy(&mut decoder, &mut limited)?
        }
        ".xz" => {
            let mut decoder = XzDecoder::new(reader);
            io::copy(&mut decoder, &mut limited)?
        }
        ".zst" => {
            let mut decoder = StreamingDecoder::new(reader)?;
            io::copy(&mut decoder, &mut limited)?
        }
        ".lz4" => {
            let mut decoder = FrameDecoder::new(reader);
            io::copy(&mut decoder, &mut limited)?
        }
        ".br" => {
            let mut decoder = Decompressor::new(reader, 4096);
            io::copy(&mut decoder, &mut limited)?
        }
        ".lz" => {
            let mut decoder = LzipReader::new(reader);
            io::copy(&mut decoder, &mut limited)?
        }
        ".lzma" => {
            lzma_rs::lzma_decompress(&mut reader, &mut limited)?;
            0
        }
        ".lzo" => {
            let mut decoder = lzo::LzopReader::new(reader)?;
            io::copy(&mut decoder, &mut limited)?
        }
        _ => return Err(anyhow!("Unsupported stream extension: {ext}")),
    };

    if let Some(ref pb) = progress {
        let target_file = limited.into_inner();
        let size = target_file.metadata().map(|m| m.len()).unwrap_or(0);
        pb.finish(format!(
            "Decompressed to {} ({})",
            target_path.display(),
            format_size(size)
        ));
    }

    Ok(())
}
