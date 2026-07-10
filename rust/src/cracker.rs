use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor, Read, Write};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use sevenz_rust2::{ArchiveReader, Password};
use unarc_rs::unified::{ArchiveFormat, ArchiveOptions, UnifiedArchive};
use zip::ZipArchive;

use crate::archive::format::{self, Format};
use crate::extract::{extract_extension, ExtractOptions};

const DEFAULT_WORDLIST: &[u8] = include_bytes!("../assets/default_wordlist.txt.zst");

pub fn resolve_format_for(file_path: &Path, cli_format: Option<&str>) -> Result<Format> {
    if let Some(fmt) = cli_format {
        return Format::from_cli(fmt);
    }
    let file_name_lower = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    let ext = extract_extension(&file_name_lower).unwrap_or_default();
    format::resolve_format(file_path, None, ext).with_context(|| {
        format!(
            "Cannot determine format for cracking: {}",
            file_path.display()
        )
    })
}

/// Try to crack a password-protected archive using a dictionary attack.
/// Returns the cracked password if found, or `None` if no password matched.
pub fn crack_archive(
    file_path: &Path,
    options: &ExtractOptions,
    wordlist_path: Option<&Path>,
) -> Result<Option<String>> {
    let fmt = resolve_format_for(file_path, options.format.as_deref())?;
    match fmt {
        Format::Zip => crack_with_wordlist(wordlist_path, |p| try_zip_password(file_path, p)),
        Format::SevenZ => crack_with_wordlist(wordlist_path, |p| try_7z_password(file_path, p)),
        Format::Rar => crack_with_wordlist(wordlist_path, |p| try_rar_password(file_path, p)),
        Format::Arj => crack_with_wordlist(wordlist_path, |p| try_arj_password(file_path, p)),
        _ => Err(anyhow!("Cracking is not supported for this format")),
    }
}

fn crack_with_wordlist<F>(wordlist_path: Option<&Path>, mut try_fn: F) -> Result<Option<String>>
where
    F: FnMut(&str) -> Result<bool>,
{
    if let Some(path) = wordlist_path {
        let file = File::open(path)
            .with_context(|| format!("Cannot open wordlist: {}", path.display()))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let password = line?;
            if try_fn(&password)? {
                return Ok(Some(password));
            }
        }
    } else {
        let cursor = Cursor::new(DEFAULT_WORDLIST);
        let decoder = ruzstd::decoding::StreamingDecoder::new(cursor)
            .context("Failed to decompress built-in wordlist")?;
        let reader = BufReader::new(decoder);
        for line in reader.lines() {
            let password = line?;
            if try_fn(&password)? {
                return Ok(Some(password));
            }
        }
    }
    Ok(None)
}

fn try_zip_password(file_path: &Path, password: &str) -> Result<bool> {
    let file = File::open(file_path)?;
    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return Ok(false),
    };

    let mut index = 0;
    for i in 0..archive.len() {
        let is_dir = match archive.by_index(i) {
            Ok(e) => e.is_dir(),
            Err(_) => continue,
        };
        if !is_dir {
            index = i;
            break;
        }
    }

    let mut entry = match archive.by_index_decrypt(index, password.as_bytes()) {
        Ok(e) => e,
        Err(_) => return Ok(false),
    };

    let mut buf = [0u8; 1024];
    loop {
        match entry.read(&mut buf) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => return Ok(false),
        }
    }
    Ok(true)
}

fn try_7z_password(file_path: &Path, password: &str) -> Result<bool> {
    let file = File::open(file_path)?;
    let password_obj = Password::from(password);
    let mut archive_reader = match ArchiveReader::new(file, password_obj) {
        Ok(r) => r,
        Err(_) => return Ok(false),
    };
    let mut reached = false;
    let result = archive_reader.for_each_entries(|_entry, reader| {
        reached = true;
        io::copy(reader, &mut io::sink())?;
        Ok(true)
    });
    Ok(reached && result.is_ok())
}

fn try_rar_password(file_path: &Path, password: &str) -> Result<bool> {
    let archive = match rars::ArchiveReader::read_path(file_path) {
        Ok(a) => a,
        Err(_) => return Ok(false),
    };
    let result = archive.extract_to(Some(password.as_bytes()), |_meta| {
        Ok(Box::new(io::sink()) as Box<dyn Write>)
    });
    Ok(result.is_ok())
}

fn try_arj_password(file_path: &Path, password: &str) -> Result<bool> {
    let file = File::open(file_path)?;
    let options = ArchiveOptions::new().with_password(password);
    let mut archive =
        match UnifiedArchive::open_with_format_and_options(file, ArchiveFormat::Arj, options) {
            Ok(a) => a,
            Err(_) => return Ok(false),
        };
    let entry = match archive.next_entry() {
        Ok(Some(e)) => e,
        Ok(None) => return Ok(true),
        Err(_) => return Ok(false),
    };
    match archive.read(&entry) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Print a john/hashcat-compatible hash for supported formats.
pub fn extract_hash(file_path: &Path, format: Format) -> Result<()> {
    match format {
        Format::Zip => run_external("zip2john", file_path),
        Format::Rar => run_external("rar2john", file_path),
        Format::SevenZ => run_external("7z2john.pl", file_path),
        _ => Err(anyhow!(
            "Hash extraction is only supported for ZIP, RAR, and 7z archives"
        )),
    }
}

fn run_external(name: &str, file_path: &Path) -> Result<()> {
    let output = std::process::Command::new(name)
        .arg(file_path)
        .output()
        .with_context(|| format!("{name} is not installed or failed to run"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("{name} failed: {stderr}"));
    }
    io::stdout().write_all(&output.stdout)?;
    Ok(())
}
