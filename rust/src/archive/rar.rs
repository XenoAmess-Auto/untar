use std::cell::RefCell;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::extract::{
    print_entry, resolve_conflict, safe_output_path, should_extract, strip_path_components,
    EntryInfo, ExtractOptions,
};

pub fn extract_rar<P: AsRef<Path>>(path: P, options: &ExtractOptions) -> Result<()> {
    let archive = rars::ArchiveReader::read_path(path.as_ref())?;

    let total_count = archive.members().count();
    let extracted_count = RefCell::new(0u64);

    if !options.quiet && !options.list {
        println!("Total files: {total_count}");
    }

    if options.list {
        for member in archive.members() {
            let meta = &member.meta;
            let name = meta.name_lossy();
            let path = match strip_path_components(Path::new(&name), options.strip_components) {
                Some(p) => p,
                None => continue,
            };
            if !should_extract(&path, &options.patterns) {
                continue;
            }

            print_entry(&EntryInfo {
                path,
                size: meta.unpacked_size,
                is_dir: meta.is_directory,
                mode: Some(meta.file_attr as u32),
            });
        }
        return Ok(());
    }

    let password = options.password.as_deref().map(|p| p.as_bytes());

    archive
        .extract_to(password, |meta| {
            let name = meta.name_lossy();
            let path = match strip_path_components(Path::new(&name), options.strip_components) {
                Some(p) => p,
                None => return Ok(Box::new(io::sink()) as Box<dyn Write>),
            };
            if !should_extract(&path, &options.patterns) {
                return Ok(Box::new(io::sink()) as Box<dyn Write>);
            }

            let entry_path = match safe_output_path(&options.output_dir, &path)
                .with_context(|| format!("Unsafe entry path: {name}"))
                .map_err(map_rar_err)
            {
                Ok(p) => p,
                Err(e) => return Err(e),
            };

            if meta.is_directory {
                if let Some(parent) = entry_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(map_rar_err)?;
                    }
                }
                fs::create_dir_all(&entry_path).map_err(map_rar_err)?;
                return Ok(Box::new(io::sink()) as Box<dyn Write>);
            }

            let target_path =
                match resolve_conflict(&entry_path, options.on_exists, &options.rename_suffix)
                    .with_context(|| {
                        format!("Conflict handling failed for {}", entry_path.display())
                    })
                    .map_err(map_rar_err)
                {
                    Ok(Some(p)) => p,
                    Ok(None) => return Ok(Box::new(io::sink()) as Box<dyn Write>),
                    Err(e) => return Err(e),
                };

            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(map_rar_err)?;
                }
            }

            if !options.quiet {
                let mut count = extracted_count.borrow_mut();
                *count += 1;
                println!("[{}] {}", *count, name);
            }

            match fs::File::create(&target_path).map_err(map_rar_err) {
                Ok(file) => Ok(Box::new(file) as Box<dyn Write>),
                Err(e) => Err(e),
            }
        })
        .map_err(|e| anyhow!("RAR extraction failed: {e}"))?;

    if !options.quiet && !options.list {
        println!("Extracted files: {}", extracted_count.borrow());
    }

    Ok(())
}

fn map_rar_err<E: std::fmt::Display>(e: E) -> rars::Error {
    std::io::Error::other(e.to_string()).into()
}
