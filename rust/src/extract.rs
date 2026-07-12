use anyhow::{anyhow, Context, Result};
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::archive::{
    ar, cab, cpio, deb, format, iso, lha, rar, rpm, sevenz, squashfs, stream, tar, unarc, xar,
    zip as zip_mod,
};
use crate::cli::OnExists;

/// Options controlling extraction or listing.
#[derive(Debug, Clone)]
pub struct ExtractOptions {
    pub output_dir: PathBuf,
    pub quiet: bool,
    pub list: bool,
    pub on_exists: OnExists,
    pub rename_suffix: String,
    pub strip_components: Option<usize>,
    pub patterns: Vec<String>,
    pub password: Option<String>,
    pub format: Option<String>,
    pub limits: LimitTracker,
}

/// Tracks security limits during extraction and prompts the user when a limit
/// would be exceeded.
#[derive(Debug, Clone)]
pub struct LimitTracker {
    inner: Arc<Mutex<LimitTrackerInner>>,
}

#[derive(Debug)]
struct LimitTrackerInner {
    max_total_size: u64,
    max_entry_size: u64,
    max_entry_count: u64,
    max_compression_ratio: u64,
    max_recursion_depth: u32,
    allow_unsafe: bool,
    total_written: u64,
    entry_count: u64,
    depth: u32,
    warned: bool,
    confirmed: bool,
}

impl LimitTracker {
    pub fn new(
        max_total_size: u64,
        max_entry_size: u64,
        max_entry_count: u64,
        max_compression_ratio: u64,
        max_recursion_depth: u32,
        allow_unsafe: bool,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LimitTrackerInner {
                max_total_size,
                max_entry_size,
                max_entry_count,
                max_compression_ratio,
                max_recursion_depth,
                allow_unsafe,
                total_written: 0,
                entry_count: 0,
                depth: 0,
                warned: false,
                confirmed: false,
            })),
        }
    }

    fn with_inner<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut LimitTrackerInner) -> T,
    {
        let mut inner = self.inner.lock().unwrap();
        f(&mut inner)
    }

    fn is_allowed(&self) -> bool {
        self.with_inner(|inner| inner.allow_unsafe || inner.confirmed)
    }

    fn set_warned(&self) {
        self.with_inner(|inner| {
            inner.warned = true;
        });
    }

    fn set_confirmed(&self) {
        self.with_inner(|inner| {
            inner.confirmed = true;
        });
    }

    fn prompt(message: &str) -> Result<bool> {
        if !is_tty() {
            return Ok(false);
        }
        eprint!("Warning: {message}\nContinue anyway? [y/N] ");
        Write::flush(&mut io::stderr())?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        Ok(input == "y" || input == "yes")
    }

    fn maybe_continue(&self, message: String) -> Result<()> {
        if self.is_allowed() {
            return Ok(());
        }
        self.set_warned();
        if Self::prompt(&message)? {
            self.set_confirmed();
            Ok(())
        } else {
            Err(anyhow!("Aborted: {message}"))
        }
    }

    pub fn record_entry(&self, size: u64) -> Result<()> {
        let exceeds_entry = self.with_inner(|inner| {
            inner.entry_count += 1;
            if inner.entry_count > inner.max_entry_count {
                Some(format!(
                    "Archive contains more than {} entries",
                    inner.max_entry_count
                ))
            } else if size > inner.max_entry_size {
                Some(format!(
                    "Archive entry exceeds maximum entry size of {}",
                    format_size(inner.max_entry_size)
                ))
            } else {
                None
            }
        });
        if let Some(msg) = exceeds_entry {
            self.maybe_continue(msg)?;
        }
        Ok(())
    }

    pub fn record_written(&self, bytes: u64) -> Result<()> {
        let exceeds_total = self.with_inner(|inner| {
            inner.total_written = inner.total_written.saturating_add(bytes);
            if inner.total_written > inner.max_total_size {
                Some(format!(
                    "Total extracted size exceeds {}",
                    format_size(inner.max_total_size)
                ))
            } else {
                None
            }
        });
        if let Some(msg) = exceeds_total {
            self.maybe_continue(msg)?;
        }
        Ok(())
    }

    pub fn check_ratio(&self, compressed: u64, uncompressed: u64) -> Result<()> {
        if compressed == 0 {
            return Ok(());
        }
        let ratio = (uncompressed as f64) / (compressed as f64);
        let max = self.with_inner(|inner| inner.max_compression_ratio as f64);
        if ratio > max {
            self.maybe_continue(format!(
                "Compression ratio ({ratio:.0}:1) exceeds maximum allowed ({max:.0}:1)"
            ))?;
        }
        Ok(())
    }

    pub fn enter_archive(&self) -> Result<()> {
        let exceeds_depth = self.with_inner(|inner| {
            inner.depth += 1;
            if inner.depth > inner.max_recursion_depth {
                Some(format!(
                    "Archive recursion depth exceeds {}",
                    inner.max_recursion_depth
                ))
            } else {
                None
            }
        });
        if let Some(msg) = exceeds_depth {
            self.maybe_continue(msg)?;
        }
        Ok(())
    }

    pub fn exit_archive(&self) {
        self.with_inner(|inner| {
            if inner.depth > 0 {
                inner.depth -= 1;
            }
        });
    }
}

/// Wraps a writer so that per-entry and total extraction limits are enforced.
pub struct LimitedWriter<W> {
    inner: W,
    tracker: LimitTracker,
    entry_written: u64,
    entry_limit: u64,
}

impl<W: Write> LimitedWriter<W> {
    pub fn new(inner: W, tracker: LimitTracker) -> Self {
        let entry_limit = tracker.with_inner(|inner| inner.max_entry_size);
        Self {
            inner,
            tracker,
            entry_written: 0,
            entry_limit,
        }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for LimitedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tracker
            .record_written(buf.len() as u64)
            .map_err(io::Error::other)?;
        self.entry_written = self.entry_written.saturating_add(buf.len() as u64);
        if !self.tracker.is_allowed() && self.entry_written > self.entry_limit {
            return Err(io::Error::other(
                "Exceeded maximum entry size while writing",
            ));
        }
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Information about an archive entry for listing.
pub struct EntryInfo {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub mode: Option<u32>,
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

/// Validate that a symlink target stays inside the output directory.
///
/// The target is resolved relative to `link_parent` (the directory containing the
/// symlink). Absolute targets and targets that escape `output_dir` are rejected.
pub fn validate_symlink_target(output_dir: &Path, link_parent: &Path, target: &Path) -> Result<()> {
    if target.is_absolute() {
        return Err(anyhow!(
            "Absolute symlink target is not allowed: {}",
            target.display()
        ));
    }

    let relative_parent = link_parent
        .strip_prefix(output_dir)
        .map_err(|_| anyhow!("Symlink parent is outside output directory"))?;
    let mut components: Vec<std::ffi::OsString> = relative_parent
        .components()
        .map(|c| c.as_os_str().to_os_string())
        .collect();

    for component in target.components() {
        match component {
            std::path::Component::Normal(c) => components.push(c.to_os_string()),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if components.pop().is_none() {
                    return Err(anyhow!(
                        "Symlink target escapes output directory: {}",
                        target.display()
                    ));
                }
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(anyhow!(
                    "Absolute symlink target is not allowed: {}",
                    target.display()
                ));
            }
        }
    }

    Ok(())
}

/// Strip the first N leading path components from a path.
/// Returns `None` if the path has fewer than N components.
/// Returns the original path when N is `None`.
pub fn strip_path_components(path: &Path, n: Option<usize>) -> Option<PathBuf> {
    match n {
        None => Some(path.to_path_buf()),
        Some(n) => {
            let components: Vec<_> = path.components().collect();
            if components.len() < n {
                return None;
            }
            Some(components.into_iter().skip(n).collect())
        }
    }
}

/// Check if a path matches the user-provided patterns.
/// Returns `true` if there are no patterns or if the path starts with any pattern.
pub fn should_extract(path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return true;
    }
    patterns
        .iter()
        .any(|p| path.starts_with(Path::new(p)) || Path::new(p).starts_with(path))
}

/// Resolve what to do when an output path already exists.
/// Returns `Some(target_path)` if the operation should proceed, `None` if it should be skipped.
pub fn resolve_conflict(
    output_path: &Path,
    on_exists: OnExists,
    rename_suffix: &str,
) -> Result<Option<PathBuf>> {
    if !output_path.exists() {
        return Ok(Some(output_path.to_path_buf()));
    }

    match on_exists {
        OnExists::Overwrite => Ok(Some(output_path.to_path_buf())),
        OnExists::Skip => Ok(None),
        OnExists::Error => Err(anyhow!("File already exists: {}", output_path.display())),
        OnExists::Rename => {
            let candidate = find_rename_target(output_path, rename_suffix)?;
            Ok(Some(candidate))
        }
        OnExists::Ask => {
            if !is_tty() {
                return Err(anyhow!(
                    "File already exists and stdin is not a TTY: {}",
                    output_path.display()
                ));
            }
            loop {
                eprint!(
                    "File {} already exists. Overwrite/skip/rename? [o/s/r]: ",
                    output_path.display()
                );
                io::Write::flush(&mut io::stderr())?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                match input.trim().to_lowercase().as_str() {
                    "o" | "overwrite" => return Ok(Some(output_path.to_path_buf())),
                    "s" | "skip" => return Ok(None),
                    "r" | "rename" => {
                        let candidate = find_rename_target(output_path, rename_suffix)?;
                        return Ok(Some(candidate));
                    }
                    _ => eprintln!("Invalid choice. Please enter 'o', 's', or 'r'."),
                }
            }
        }
    }
}

/// Find a non-existing target path by appending the rename suffix, then `.2`, `.3`, etc.
fn find_rename_target(output_path: &Path, rename_suffix: &str) -> Result<PathBuf> {
    let base = format!("{}{}", output_path.display(), rename_suffix);
    let mut candidate = PathBuf::from(&base);
    if !candidate.exists() {
        return Ok(candidate);
    }

    let mut counter = 2u32;
    loop {
        let path_str = format!("{base}.{counter}");
        candidate = PathBuf::from(&path_str);
        if !candidate.exists() {
            return Ok(candidate);
        }
        counter += 1;
        if counter > 10000 {
            return Err(anyhow!(
                "Could not find a free rename target for {}",
                output_path.display()
            ));
        }
    }
}

/// Return whether stdin is connected to a terminal.
pub fn is_tty() -> bool {
    std::io::stdin().is_terminal()
}

/// Progress reporter for extraction.
///
/// When the total number of items is known upfront, a bar is shown; otherwise a spinner is used.
pub struct Progress {
    bar: indicatif::ProgressBar,
}

impl Progress {
    pub fn spinner() -> Self {
        let bar = indicatif::ProgressBar::new_spinner();
        bar.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("invalid spinner template"),
        );
        Self { bar }
    }

    pub fn bar(total: u64) -> Self {
        let bar = indicatif::ProgressBar::new(total);
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .expect("invalid bar template")
                .progress_chars("#>-"),
        );
        Self { bar }
    }

    pub fn set_message(&self, msg: impl Into<String>) {
        self.bar.set_message(msg.into());
    }

    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn finish(&self, msg: impl Into<String>) {
        self.bar.finish_with_message(msg.into());
    }
}

/// Print a single archive entry for `--list`.
pub fn print_entry(info: &EntryInfo) {
    let mode_str = info.mode.map(|m| format!("{m:04o} ")).unwrap_or_default();
    let size_str = if info.is_dir {
        "-".to_string()
    } else {
        format_size(info.size)
    };
    println!("{}{} {}", mode_str, size_str, info.path.display());
}

/// Extract a supported archive file.
pub fn extract_archive(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let file = std::fs::File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;

    let file_name_lower = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    let ext = extract_extension(&file_name_lower).unwrap_or_default();

    let fmt = format::resolve_format(file_path, options.format.as_deref(), ext)
        .with_context(|| format!("Cannot determine format: {}", file_path.display()))?;

    if !options.quiet && !options.list {
        let file_size = file.metadata()?.len();
        println!("Archive: {}", file_path.display());
        println!("Size: {}", format_size(file_size));
        if !ext.is_empty() {
            println!("Detected format: {fmt:?}");
        }
    }

    std::fs::create_dir_all(&options.output_dir)
        .with_context(|| format!("Cannot create directory: {}", options.output_dir.display()))?;

    options.limits.enter_archive()?;
    let result = match fmt {
        format::Format::TarGz => tar::extract_tar_gz(file, options),
        format::Format::TarXz => tar::extract_tar_xz(file, options),
        format::Format::TarBz2 => tar::extract_tar_bz2(file, options),
        format::Format::TarLzma => tar::extract_tar_lzma(file, options),
        format::Format::TarLz => tar::extract_tar_lz(file, options),
        format::Format::TarZst => tar::extract_tar_zst(file, options),
        format::Format::TarLz4 => tar::extract_tar_lz4(file, options),
        format::Format::TarBr => tar::extract_tar_br(file, options),
        format::Format::Tar => tar::extract_tar(file, options),
        format::Format::Zip => zip_mod::extract_zip(file, options),
        format::Format::SevenZ => sevenz::extract_7z(file, options),
        format::Format::Rar => rar::extract_rar(file_path, options),
        format::Format::Cab => cab::extract_cab(file_path, options),
        format::Format::Ar => ar::extract_ar(file_path, options),
        format::Format::Cpio => cpio::extract_cpio(file_path, options),
        format::Format::Iso => iso::extract_iso(file_path, options),
        format::Format::Xar => xar::extract_xar(file_path, options),
        format::Format::Lha | format::Format::Lzh => lha::extract_lha(file_path, options),
        format::Format::Deb => deb::extract_deb(file_path, options),
        format::Format::Squashfs => squashfs::extract_squashfs(file_path, options),
        format::Format::Rpm => rpm::extract_rpm(file_path, options),
        format::Format::TarLzo => tar::extract_tar_lzo(file, options),
        format::Format::TarZ => unarc::extract_tarz(file_path, options),
        format::Format::Z => unarc::extract_z(file_path, options),
        format::Format::Ace => unarc::extract_ace(file_path, options),
        format::Format::Arc => unarc::extract_arc(file_path, options),
        format::Format::Arj => unarc::extract_arj(file_path, options),
        format::Format::Zoo => unarc::extract_zoo(file_path, options),
        format::Format::Gz => stream::extract_stream(file, file_path, options, ".gz"),
        format::Format::Bz2 => stream::extract_stream(file, file_path, options, ".bz2"),
        format::Format::Xz => stream::extract_stream(file, file_path, options, ".xz"),
        format::Format::Zst => stream::extract_stream(file, file_path, options, ".zst"),
        format::Format::Lz4 => stream::extract_stream(file, file_path, options, ".lz4"),
        format::Format::Br => stream::extract_stream(file, file_path, options, ".br"),
        format::Format::Lzma => stream::extract_stream(file, file_path, options, ".lzma"),
        format::Format::Lzo => stream::extract_stream(file, file_path, options, ".lzo"),
        format::Format::Lz => stream::extract_stream(file, file_path, options, ".lz"),
    };
    options.limits.exit_archive();
    result
}

pub fn extract_extension(file_name_lower: &str) -> Option<&str> {
    [
        ".tar.gz",
        ".tar.xz",
        ".tar.bz2",
        ".tar.lzma",
        ".tar.lz",
        ".tar.zst",
        ".tar.lz4",
        ".tar.br",
        ".tar.lzo",
        ".tar.z",
        ".taz",
        ".z",
        ".ace",
        ".arc",
        ".arj",
        ".zoo",
        ".tgz",
        ".txz",
        ".tbz2",
        ".tbz",
        ".tlz",
        ".tzst",
        ".zip",
        ".apk",
        ".jar",
        ".war",
        ".ear",
        ".7z",
        ".rar",
        ".cab",
        ".ar",
        ".a",
        ".cpio",
        ".iso",
        ".xar",
        ".lha",
        ".lzh",
        ".deb",
        ".squashfs",
        ".sqfs",
        ".sfs",
        ".snap",
        ".rpm",
        ".tar",
        ".gz",
        ".bz2",
        ".xz",
        ".zst",
        ".lz4",
        ".br",
        ".lzma",
        ".lzo",
        ".lz",
        ".pax",
        ".pak",
    ]
    .into_iter()
    .find(|ext| file_name_lower.ends_with(ext))
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

    #[test]
    fn test_strip_components() {
        assert_eq!(
            strip_path_components(Path::new("a/b/c.txt"), Some(1)).unwrap(),
            PathBuf::from("b/c.txt")
        );
        assert_eq!(
            strip_path_components(Path::new("a/b/c.txt"), Some(2)).unwrap(),
            PathBuf::from("c.txt")
        );
        assert!(strip_path_components(Path::new("a.txt"), Some(2)).is_none());
    }

    #[test]
    fn test_should_extract() {
        assert!(should_extract(Path::new("a/b.txt"), &[]));
        assert!(should_extract(Path::new("a/b.txt"), &["a".to_string()]));
        assert!(should_extract(
            Path::new("a/b.txt"),
            &["a/b.txt".to_string()]
        ));
        assert!(!should_extract(Path::new("c/d.txt"), &["a".to_string()]));
    }
}
