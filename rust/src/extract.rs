use anyhow::{anyhow, Context, Result};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::archive::tar;
use crate::archive::zip as zip_mod;

/// Options controlling extraction.
#[derive(Debug, Clone)]
pub struct ExtractOptions {
    pub output_dir: PathBuf,
    pub quiet: bool,
}

impl ExtractOptions {
    pub fn new(output_dir: PathBuf, quiet: bool) -> Self {
        Self { output_dir, quiet }
    }
}

/// Extract a supported archive file.
pub fn extract_archive(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;

    let file_name_lower = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !options.quiet {
        let file_size = file.metadata()?.len();
        println!("Archive: {}", file_path.display());
        println!("Size: {}", format_size(file_size));
    }

    // Create output directory.
    fs::create_dir_all(&options.output_dir)
        .with_context(|| format!("Cannot create directory: {}", options.output_dir.display()))?;

    // Detect format by extension and extract.
    if file_name_lower.ends_with(".tar.gz") || file_name_lower.ends_with(".tgz") {
        tar::extract_tar_gz(file, options)?;
    } else if file_name_lower.ends_with(".tar.xz") {
        tar::extract_tar_xz(file, options)?;
    } else if file_name_lower.ends_with(".tar.bz2") {
        tar::extract_tar_bz2(file, options)?;
    } else if file_name_lower.ends_with(".zip") {
        zip_mod::extract_zip(file, options)?;
    } else if file_name_lower.ends_with(".tar") {
        tar::extract_tar(file, options)?;
    } else {
        return Err(anyhow!(
            "Unsupported archive format. Please use a known extension (.tar, .tar.gz, .tgz, .tar.xz, .tar.bz2, .zip)"
        ));
    }

    Ok(())
}

/// Format a byte size in human-readable form.
pub fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Normalize an archive entry path and ensure it stays within the output directory.
///
/// Rejects absolute paths and any `..` components to prevent path traversal / Zip Slip.
pub fn safe_output_path(output_dir: &Path, entry_path: &Path) -> Result<PathBuf> {
    let mut components = Vec::new();
    for component in entry_path.components() {
        match component {
            std::path::Component::Normal(c) => components.push(c.to_os_string()),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                return Err(anyhow!(
                    "Path traversal detected in entry: {}",
                    entry_path.display()
                ));
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(anyhow!(
                    "Absolute paths are not allowed in archive entries: {}",
                    entry_path.display()
                ));
            }
        }
    }

    let output_path = components
        .iter()
        .fold(output_dir.to_path_buf(), |p, c| p.join(c));
    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_safe_output_path_normal() {
        let out = PathBuf::from("/tmp/out");
        assert_eq!(
            safe_output_path(&out, Path::new("dir/file.txt")).unwrap(),
            PathBuf::from("/tmp/out/dir/file.txt")
        );
    }

    #[test]
    fn test_safe_output_path_traversal() {
        let out = PathBuf::from("/tmp/out");
        assert!(safe_output_path(&out, Path::new("../file.txt")).is_err());
        assert!(safe_output_path(&out, Path::new("dir/../../file.txt")).is_err());
    }

    #[test]
    fn test_safe_output_path_absolute() {
        let out = PathBuf::from("/tmp/out");
        assert!(safe_output_path(&out, Path::new("/etc/passwd")).is_err());
    }
}
