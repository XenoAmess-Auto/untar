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

/// Extract tar/tar.gz/tgz/tar.xz/tar.bz2/tar.zip packages.
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

    /// Archive file to extract.
    #[arg(value_name = "FILE", index = 1)]
    pub file: Option<String>,

    /// Patterns of files/paths to extract.
    #[arg(value_name = "PATTERNS", index = 2, num_args = 0..)]
    pub patterns: Vec<String>,
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
