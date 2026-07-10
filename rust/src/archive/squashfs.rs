use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use backhand::{FilesystemReader, InnerNode};

use crate::extract::{
    format_size, print_entry, resolve_conflict, safe_output_path, should_extract,
    strip_path_components, EntryInfo, ExtractOptions, Progress,
};

pub fn extract_squashfs(file_path: &Path, options: &ExtractOptions) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;
    let reader = BufReader::new(file);
    let fs = FilesystemReader::from_reader(reader)
        .with_context(|| format!("Failed to read SquashFS image: {}", file_path.display()))?;

    let mut entry_count = 0u64;
    let mut extracted_count = 0u64;
    let progress = (!options.quiet && !options.list).then(Progress::spinner);

    for node in fs.files() {
        entry_count += 1;

        let path = strip_leading_slash(&node.fullpath);
        if path.as_os_str().is_empty() {
            continue;
        }
        let path = match strip_path_components(&path, options.strip_components) {
            Some(p) => p,
            None => continue,
        };

        if !should_extract(&path, &options.patterns) {
            continue;
        }

        let (size, is_dir) = match &node.inner {
            InnerNode::File(file) => (file.file_len() as u64, false),
            InnerNode::Dir(_) => (0u64, true),
            InnerNode::Symlink(_) => (0u64, false),
            _ => continue,
        };

        if options.list {
            print_entry(&EntryInfo {
                path: path.clone(),
                size,
                is_dir,
                mode: Some(node.header.permissions as u32),
            });
            continue;
        }

        let entry_path = safe_output_path(&options.output_dir, &path)
            .with_context(|| format!("Unsafe entry path: {}", path.display()))?;

        let target_path =
            match resolve_conflict(&entry_path, options.on_exists, &options.rename_suffix)
                .with_context(|| format!("Conflict handling failed for {}", entry_path.display()))?
            {
                Some(p) => p,
                None => continue,
            };

        if is_dir {
            fs::create_dir_all(&target_path)?;
            continue;
        }

        if let InnerNode::Symlink(symlink) = &node.inner {
            #[cfg(unix)]
            {
                if let Some(parent) = target_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                std::os::unix::fs::symlink(&symlink.link, &target_path)?;
            }
            #[cfg(not(unix))]
            {
                // Symlinks are skipped on non-Unix platforms.
                let _ = symlink;
            }
            continue;
        }

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if let Some(ref pb) = progress {
            pb.set_message(format!(
                "[{:?}] {} ({})",
                entry_count,
                path.display(),
                format_size(size)
            ));
        }

        let mut target_file = File::create(&target_path)?;
        match &node.inner {
            InnerNode::File(file) => {
                let mut reader = fs.file(file).reader();
                io::copy(&mut reader, &mut target_file)?;
            }
            _ => unreachable!(),
        }
        extracted_count += 1;
        if let Some(ref pb) = progress {
            pb.inc(1);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = PermissionsExt::from_mode(node.header.permissions as u32);
            if let Err(e) = fs::set_permissions(&target_path, permissions) {
                eprintln!("Warning: Could not set permissions: {e}");
            }
        }
    }

    if !options.quiet && !options.list {
        if let Some(ref pb) = progress {
            pb.finish(format!("Extracted {extracted_count} files"));
        } else {
            println!("Extracted files: {extracted_count}");
        }
    }

    Ok(())
}

fn strip_leading_slash(path: &Path) -> PathBuf {
    path.strip_prefix("/").unwrap_or(path).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_strip_leading_slash() {
        assert_eq!(strip_leading_slash(Path::new("/a/b")), PathBuf::from("a/b"));
        assert_eq!(strip_leading_slash(Path::new("a/b")), PathBuf::from("a/b"));
        assert_eq!(strip_leading_slash(Path::new("/")), PathBuf::from(""));
    }
}
