use std::fs::{self, File};
use std::io::{self, Read, Seek};
use std::path::Path;

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::extract::{format_size, safe_output_path, ExtractOptions};

pub fn extract_zip<R: Read + Seek>(reader: R, options: &ExtractOptions) -> Result<()> {
    let mut archive = ZipArchive::new(reader)?;
    let total_count = archive.len();

    if !options.quiet {
        println!("Total files: {total_count}");
    }

    for i in 0..total_count {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        let size = entry.size();

        let entry_path = safe_output_path(&options.output_dir, Path::new(&name))
            .with_context(|| format!("Unsafe entry path: {name}"))?;

        if let Some(parent) = entry_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if entry.is_dir() {
            if !options.quiet {
                println!("[{:?}] {}", i + 1, name);
            }
            fs::create_dir_all(&entry_path)?;
        } else {
            if !options.quiet {
                println!("[{:?}] {} ({})", i + 1, name, format_size(size));
            }

            let mut file = File::create(&entry_path)?;
            io::copy(&mut entry, &mut file)?;
        }
    }

    Ok(())
}
