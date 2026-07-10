use std::fs::File;
use std::io::{self, Seek};
use std::path::Path;

use anyhow::{Context, Result};
use ar::Archive;

use crate::archive::tar;
use crate::extract::{format_size, ExtractOptions, Progress};

fn extract_inner_tar<R: io::Read>(
    reader: R,
    compression: &str,
    options: &ExtractOptions,
    extracted_count: &mut u64,
) -> Result<()> {
    match compression {
        "" => tar::extract_tar(reader, options)?,
        ".gz" => tar::extract_tar_gz(reader, options)?,
        ".bz2" => tar::extract_tar_bz2(reader, options)?,
        ".xz" => tar::extract_tar_xz(reader, options)?,
        ".zst" => tar::extract_tar_zst(reader, options)?,
        ".lz4" => tar::extract_tar_lz4(reader, options)?,
        ".lzma" => tar::extract_tar_lzma(reader, options)?,
        ".br" => tar::extract_tar_br(reader, options)?,
        ".lzo" => tar::extract_tar_lzo(reader, options)?,
        _ => anyhow::bail!("Unsupported inner tar compression: {compression}"),
    }
    // extract_tar_* functions print their own progress; we cannot easily get the count back.
    // For simplicity, treat each inner tar as a unit.
    *extracted_count += 1;
    Ok(())
}

pub fn extract_deb(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;
    let mut archive = Archive::new(file);

    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(Progress::spinner);

    while let Some(entry_result) = archive.next_entry() {
        let mut entry = entry_result?;
        let name = {
            let id = entry.header().identifier();
            String::from_utf8_lossy(id)
                .trim_end_matches('/')
                .trim()
                .to_string()
        };

        let Some(tar_suffix) = data_tar_suffix(&name) else {
            continue;
        };

        let mut temp = tempfile::NamedTempFile::new()?;
        io::copy(&mut entry, temp.as_file_mut())?;
        temp.as_file_mut().rewind()?;

        if let Some(ref pb) = progress {
            pb.set_message(format!(
                "Extracting {} inner tar ({})",
                name,
                format_size(temp.as_file().metadata().map(|m| m.len()).unwrap_or(0))
            ));
        }

        options.limits.enter_archive()?;
        let res = extract_inner_tar(temp.as_file(), tar_suffix, options, &mut extracted_count);
        options.limits.exit_archive();
        res.with_context(|| format!("Failed to extract inner tar from {name}"))?;
    }

    if !options.quiet && !options.list {
        if let Some(ref pb) = progress {
            pb.finish(format!("Extracted {extracted_count} inner tar archives"));
        } else {
            println!("Extracted inner tar archives: {extracted_count}");
        }
    }

    Ok(())
}

fn data_tar_suffix(name: &str) -> Option<&str> {
    // .deb data payloads are named data.tar, data.tar.gz, data.tar.xz, etc.
    name.strip_prefix("data.tar")
        .map(|s| s.trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_tar_suffix() {
        assert_eq!(data_tar_suffix("data.tar"), Some(""));
        assert_eq!(data_tar_suffix("data.tar.gz"), Some(".gz"));
        assert_eq!(data_tar_suffix("data.tar.xz"), Some(".xz"));
        assert_eq!(data_tar_suffix("data.tar.gz/"), Some(".gz"));
        assert_eq!(data_tar_suffix("control.tar.gz"), None);
        assert_eq!(data_tar_suffix("debian-binary"), None);
    }
}
