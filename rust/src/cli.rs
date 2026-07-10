use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// How to handle files that already exist in the output directory.
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OnExists {
    /// Ask the user interactively (default in a TTY).
    #[default]
    Ask,
    /// Exit with an error (safe default in non-interactive environments).
    Error,
    /// Silently overwrite the existing file.
    Overwrite,
    /// Silently skip the existing file.
    Skip,
    /// Rename the new file by appending a suffix.
    Rename,
}

/// Extract a wide variety of archive and compression formats.
///
/// Supported archive formats include tar, tar.gz, tgz, tar.xz, txz, tar.bz2,
/// tbz2, tbz, tar.lzma, tlz, tar.lz, tar.zst, tzst, tar.lz4, tar.br, tar.lzo,
/// tar.Z, taz, zip, apk, jar, war, ear, 7z, rar, cab, ar, a, cpio, iso, xar,
/// lha, lzh, deb, squashfs, sqfs, sfs, snap, rpm, ace, arc, zoo, and pax.
/// Single-stream formats include gz, bz2, xz, zst, lz4, br, lzma, lzo, lz,
/// and Z.
#[derive(Parser, Debug, Clone)]
#[command(name = "untar", author, version, about, long_about = None)]
pub struct Args {
    /// Output directory.
    #[arg(short = 'd', long, value_name = "DIR")]
    pub directory: Option<String>,

    /// Quiet mode (suppress output).
    #[arg(short, long)]
    pub quiet: bool,

    /// List archive contents instead of extracting.
    #[arg(short = 'l', long)]
    pub list: bool,

    /// How to handle existing files.
    #[arg(long = "on-exists", value_enum, default_value_t = OnExists::Ask)]
    pub on_exists: OnExists,

    /// Suffix to append when renaming existing files.
    #[arg(long, default_value = ".1")]
    pub rename_suffix: String,

    /// Strip the first N leading path components from entries.
    #[arg(long = "strip-components")]
    pub strip_components: Option<usize>,

    /// Password for encrypted archives.
    #[arg(short = 'p', long)]
    pub password: Option<String>,

    /// Force a specific archive format instead of auto-detecting from the file
    /// extension or contents.
    ///
    /// Accepted values: tar.gz, tgz, tar.xz, txz, tar.bz2, tbz2, tbz, tar.lzma,
    /// tlz, tar.lz, tar.zst, tzst, tar.lz4, tar.br, tar, zip, apk, jar, war,
    /// ear, 7z, rar, cab, ar, a, cpio, iso, xar, lha, lzh, deb, squashfs,
    /// sqfs, sfs, snap, rpm, tar.lzo, tar.z, taz, z, ace, arc, zoo, pax,
    /// gz, bz2, xz, zst, lz4, br, lzma, lzo, lz.
    #[arg(long = "format", value_name = "FORMAT")]
    pub format: Option<String>,

    /// Maximum total bytes written for the entire extraction.
    #[arg(long, value_name = "SIZE", default_value = "10GB", value_parser = parse_size)]
    pub max_total_size: u64,

    /// Maximum bytes for a single output file.
    #[arg(long, value_name = "SIZE", default_value = "1GB", value_parser = parse_size)]
    pub max_entry_size: u64,

    /// Maximum number of entries extracted.
    #[arg(long, value_name = "N", default_value = "10000")]
    pub max_entry_count: u64,

    /// Maximum allowed uncompressed-to-compressed size ratio.
    #[arg(long, value_name = "N", default_value = "100")]
    pub max_compression_ratio: u64,

    /// Maximum nested-archive depth.
    #[arg(long, value_name = "N", default_value = "3")]
    pub max_recursion_depth: u32,

    /// Skip security warnings and continue extraction even when limits are exceeded.
    #[arg(long)]
    pub allow_unsafe: bool,

    /// Archive file to extract.
    #[arg(value_name = "FILE", index = 1)]
    pub file: Option<String>,

    /// Patterns of files/paths to extract.
    #[arg(value_name = "PATTERNS", index = 2, num_args = 0..)]
    pub patterns: Vec<String>,
}

fn parse_size(s: &str) -> Result<u64, String> {
    let original = s.trim();
    let normalized = original.replace([' ', '_'], "");
    let (num, unit) = if let Some(pos) = normalized.find(|c: char| !c.is_ascii_digit() && c != '.')
    {
        let (n, u) = normalized.split_at(pos);
        (n.to_string(), u.trim().to_lowercase())
    } else {
        (normalized, "b".to_string())
    };
    if num.is_empty() {
        return Err(format!("Empty size: {original}"));
    }
    let value: f64 = num
        .parse()
        .map_err(|_| format!("Invalid size: {original}"))?;
    let multiplier: u64 = match unit.as_str() {
        "b" | "" => 1,
        "k" | "kb" => 1024,
        "m" | "mb" => 1024 * 1024,
        "g" | "gb" => 1024 * 1024 * 1024,
        "t" | "tb" => 1024u64 * 1024 * 1024 * 1024,
        _ => return Err(format!("Unknown size unit: {unit}")),
    };
    Ok((value * multiplier as f64) as u64)
}

impl Args {
    /// Resolve the output directory path.
    pub fn output_dir(&self) -> PathBuf {
        self.directory
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Resolve the archive file path.
    pub fn archive_file(&self) -> Option<String> {
        self.file.clone()
    }
}
