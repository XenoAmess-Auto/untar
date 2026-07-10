use clap::Parser;
use std::path::PathBuf;

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

    /// Archive file to extract.
    #[arg(value_name = "FILE", index = 1)]
    pub file: Option<String>,
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
