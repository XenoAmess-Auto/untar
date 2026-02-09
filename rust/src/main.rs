use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::exit;

/// Extract tar/tar.gz/tgz/tar.xz/tar.bz2/tar.zip packages
#[derive(Parser, Debug)]
#[command(name = "untar")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output directory
    #[arg(short = 'd', long, value_name = "DIR")]
    directory: Option<String>,

    /// Quiet mode (suppress output)
    #[arg(short, long)]
    quiet: bool,

    /// Show help
    #[arg(short, long)]
    help: bool,

    /// Archive file to extract
    #[arg(value_name = "FILE", index = 1)]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.help {
        print!("{}", HELP);
        exit(0);
    }

    let file = match args.file {
        Some(f) => f,
        None => {
            eprintln!("Error: No archive file specified");
            print!("{}", HELP);
            exit(1);
        }
    };

    let quiet = args.quiet;
    let directory = args.directory.unwrap_or_else(|| ".".to_string());

    if let Err(e) = extract_archive(&file, &directory, quiet) {
        eprintln!("Error: {}", e);
        if !quiet {
            eprintln!("{:?}", e);
        }
        exit(1);
    }

    if !quiet {
        println!("Done: {}", file);
    }
}

const HELP: &str = "untar - Extract tar/tar.gz/tgz/tar.xz/tar.bz2/tar.zip packages

Usage: untar [OPTIONS] FILE

Options:
  -d, --directory DIR    Extract files into DIR (default: current directory)
  -q, --quiet            Suppress output
  -h, --help             Show this help

Supported formats:
  .tar, .tar.gz, .tgz, .tar.xz, .tar.bz2, .zip
";

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn extract_archive(file_path: &str, output_dir: &str, quiet: bool) -> Result<()> {
    let file = File::open(file_path).with_context(|| format!("Cannot open file: {}", file_path))?;

    let file_name_lower = file_path.to_lowercase();

    // Get file size for progress
    let file_size = file.metadata()?.len();

    if !quiet {
        println!("Archive: {}", file_path);
        println!("Size: {}", format_size(file_size));
    }

    // Create output directory
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Cannot create directory: {}", output_dir))?;

    // Detect format by extension and extract
    if file_name_lower.ends_with(".tar.gz") || file_name_lower.ends_with(".tgz") {
        extract_tar_gz(file, output_dir, quiet)?;
    } else if file_name_lower.ends_with(".tar.xz") {
        extract_tar_xz(file, output_dir, quiet)?;
    } else if file_name_lower.ends_with(".tar.bz2") {
        extract_tar_bz2(file, output_dir, quiet)?;
    } else if file_name_lower.ends_with(".zip") {
        extract_zip(file, output_dir, quiet)?;
    } else if file_name_lower.ends_with(".tar") {
        extract_tar(file, output_dir, quiet)?;
    } else {
        return Err(anyhow!(
            "Unsupported archive format. Please use a known extension (.tar, .tar.gz, .tgz, .tar.xz, .tar.bz2, .zip)"
        ));
    }

    Ok(())
}

fn extract_tar_gz<R: Read>(reader: R, output_dir: &str, quiet: bool) -> Result<()> {
    let decoder = flate2::read::GzDecoder::new(reader);
    extract_tar_reader(decoder, output_dir, quiet)
}

fn extract_tar_xz<R: Read>(reader: R, output_dir: &str, quiet: bool) -> Result<()> {
    let decoder = xz2::read::XzDecoder::new(reader);
    extract_tar_reader(decoder, output_dir, quiet)
}

fn extract_tar_bz2<R: Read>(reader: R, output_dir: &str, quiet: bool) -> Result<()> {
    let decoder = bzip2::read::BzDecoder::new(reader);
    extract_tar_reader(decoder, output_dir, quiet)
}

fn extract_tar<R: Read>(reader: R, output_dir: &str, quiet: bool) -> Result<()> {
    extract_tar_reader(BufReader::new(reader), output_dir, quiet)
}

fn extract_tar_reader<R: Read>(reader: R, output_dir: &str, quiet: bool) -> Result<()> {
    let mut archive = tar::Archive::new(reader);
    let mut entry_count = 0u64;

    for entry in archive.entries()? {
        let mut entry = entry?;
        entry_count += 1;

        let path = entry.path()?.into_owned();
        let entry_path = PathBuf::from(output_dir).join(&path);
        let size = entry.size();

        if let Some(parent) = entry_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if entry.header().entry_type() == tar::EntryType::Directory {
            if !quiet {
                println!("[{:?}] {}", entry_count, path.display());
            }
            fs::create_dir_all(&entry_path)?;
        } else {
            if !quiet {
                println!(
                    "[{:?}] {} ({})",
                    entry_count,
                    path.display(),
                    format_size(size)
                );
            }

            let mut file = File::create(&entry_path)?;
            io::copy(&mut entry, &mut file)?;

            // Preserve file permissions (Unix mode)
            if let Some(mode) = entry.header().mode().ok().filter(|&m| m != 0) {
                #[cfg(unix)]
                {
                    let permissions = PermissionsExt::from_mode(mode);
                    if let Err(e) = fs::set_permissions(&entry_path, permissions) {
                        eprintln!("Warning: Could not set permissions: {}", e);
                    }
                }
            }
        }
    }

    if !quiet {
        println!("Total files: {}", entry_count);
    }

    Ok(())
}

fn extract_zip<R: Read + Seek>(reader: R, output_dir: &str, quiet: bool) -> Result<()> {
    let mut archive = zip::ZipArchive::new(reader)?;
    let total_count = archive.len();

    if !quiet {
        println!("Total files: {}", total_count);
    }

    for i in 0..total_count {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        let size = entry.size();

        let entry_path = PathBuf::from(output_dir).join(&name);

        if let Some(parent) = entry_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if entry.is_dir() {
            if !quiet {
                println!("[{:?}] {}", i + 1, name);
            }
            fs::create_dir_all(&entry_path)?;
        } else {
            if !quiet {
                println!("[{:?}] {} ({})", i + 1, name, format_size(size));
            }

            let mut file = File::create(&entry_path)?;
            io::copy(&mut entry, &mut file)?;
        }
    }

    Ok(())
}
