use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// How to handle files that already exist in the output directory.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
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
/// lha, lzh, deb, squashfs, sqfs, sfs, snap, rpm, ace, arc, pak, zoo, and pax.
/// Single-stream formats include gz, bz2, xz, zst, lz4, br, lzma, lzo, lz,
/// and Z.
#[derive(Parser, Debug, Clone)]
#[command(name = "untar", author, version, about, long_about = None)]
pub struct Args {
    /// Output directory.
    #[arg(short = 'd', long, value_name = "DIR")]
    pub directory: Option<String>,

    /// Extract each archive into a subdirectory named after the archive's
    /// stem (the file name with its recognized archive extensions removed).
    /// Can be overridden individually when the `--auto` master switch is used.
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_name = "BOOL"
    )]
    pub auto_dir: Option<bool>,

    /// Enable all `--auto-*` flags (currently `--auto-dir`).
    /// Each `--auto-*` flag may still be explicitly disabled, e.g. `--auto --auto-dir=false`.
    #[arg(long)]
    pub auto: bool,

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
    /// sqfs, sfs, snap, rpm, tar.lzo, tar.z, taz, z, ace, arc, pak, zoo, pax,
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

    /// Maximum nested-archive depth for `.deb` package payloads.
    #[arg(long, value_name = "N", default_value = "3")]
    pub max_recursion_depth: u32,

    /// Skip security warnings and continue extraction even when limits are exceeded.
    #[arg(long)]
    pub allow_unsafe: bool,

    /// Try to crack a password-protected archive using a dictionary attack.
    #[arg(long)]
    pub crack: bool,

    /// Use an external wordlist file instead of the built-in dictionary.
    #[arg(long, value_name = "FILE")]
    pub wordlist: Option<String>,

    /// Print a john/hashcat-compatible hash for the archive and exit.
    #[arg(long)]
    pub extract_hash: bool,

    /// Archive file(s) to extract or list.
    #[arg(value_name = "FILES", index = 1, num_args = 1..)]
    pub files: Vec<String>,

    /// Only extract entries whose path matches one of these patterns.
    #[arg(long = "pattern", value_name = "PATTERN")]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_size_units() {
        assert_eq!(parse_size("1").unwrap(), 1);
        assert_eq!(parse_size("1B").unwrap(), 1);
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1.5M").unwrap(), 1572864);
        assert_eq!(parse_size("1G").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("1T").unwrap(), 1024u64 * 1024 * 1024 * 1024);
        assert_eq!(parse_size("1TB").unwrap(), 1024u64 * 1024 * 1024 * 1024);
    }

    #[test]
    fn parse_size_whitespace_and_underscores() {
        assert_eq!(parse_size("1 000").unwrap(), 1000);
        assert_eq!(parse_size("1_000").unwrap(), 1000);
    }

    #[test]
    fn parse_size_errors() {
        assert!(parse_size("").is_err());
        assert!(parse_size("abc").is_err());
        assert!(parse_size("1XB").is_err());
    }

    #[test]
    fn auto_enables_auto_dir() {
        let args = Args::parse_from(["untar", "--auto", "x.zip"]);
        assert!(args.auto);
        assert!(args.auto_dir.unwrap_or(args.auto));
    }

    #[test]
    fn auto_dir_false_overrides_auto() {
        let args = Args::parse_from(["untar", "--auto", "--auto-dir=false", "x.zip"]);
        assert!(!args.auto_dir.unwrap_or(args.auto));
    }
}
