use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::exit;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

    /// Show version
    #[arg(short = 'v', long)]
    version: bool,

    /// Archive file to extract
    #[arg(value_name = "FILE", index = 1)]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.version {
        println!("untar {}", VERSION);
        exit(0);
    }

    if args.help {
        println!("{}", HELP);
        exit(0);
    }

    let file = match args.file {
        Some(f) => f,
        None => {
            eprintln!("Error: No archive file specified");
            println!("{}", HELP);
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

const HELP: &str = "untar 1.0.1 - Extract tar/tar.gz/tgz/tar.xz/tar.bz2/tar.zip packages

Usage: untar [OPTIONS] FILE

Options:
  -d, --directory DIR    Extract files into DIR (default: current directory)
  -q, --quiet            Suppress output
  -v, --version          Show version
  -h, --help             Show help

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[allow(dead_code)]
    fn create_test_file(dir: &std::path::Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    fn create_test_tar(dir: &std::path::Path) -> PathBuf {
        let tar_path = dir.join("test.tar");
        let mut tar = tar::Builder::new(File::create(&tar_path).unwrap());

        let mut header = tar::Header::new_gnu();
        header.set_path("test.txt").unwrap();
        header.set_size(13);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, "Hello, World!".as_bytes()).unwrap();

        let mut header2 = tar::Header::new_gnu();
        header2.set_path("subdir/nested.txt").unwrap();
        header2.set_size(15);
        header2.set_mode(0o644);
        header2.set_cksum();
        tar.append(&header2, "Nested content!".as_bytes()).unwrap();

        tar.finish().unwrap();
        tar_path
    }

    fn create_test_tar_gz(dir: &std::path::Path) -> PathBuf {
        let tgz_path = dir.join("test.tar.gz");
        let file = File::create(&tgz_path).unwrap();
        let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar = tar::Builder::new(gz);

        let mut header = tar::Header::new_gnu();
        header.set_path("test.txt").unwrap();
        header.set_size(13);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, "Hello, World!".as_bytes()).unwrap();

        let mut header2 = tar::Header::new_gnu();
        header2.set_path("data/config.json").unwrap();
        header2.set_size(16);
        header2.set_mode(0o644);
        header2.set_cksum();
        tar.append(&header2, b"{\"key\": \"value\"}".as_slice())
            .unwrap();

        tar.finish().unwrap();
        tgz_path
    }

    fn create_test_tgz(dir: &std::path::Path) -> PathBuf {
        let tgz_path = dir.join("test.tgz");
        let file = File::create(&tgz_path).unwrap();
        let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar = tar::Builder::new(gz);

        let mut header = tar::Header::new_gnu();
        header.set_path("archive.txt").unwrap();
        header.set_size(12);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, "TGZ archive!".as_bytes()).unwrap();

        tar.finish().unwrap();
        tgz_path
    }

    fn create_test_tar_xz(dir: &std::path::Path) -> PathBuf {
        let txz_path = dir.join("test.tar.xz");
        let file = File::create(&txz_path).unwrap();
        let xz = xz2::write::XzEncoder::new(file, 6);
        let mut tar = tar::Builder::new(xz);

        let mut header = tar::Header::new_gnu();
        header.set_path("xz_test.txt").unwrap();
        header.set_size(14);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, "XZ compressed!".as_bytes()).unwrap();

        let mut header2 = tar::Header::new_gnu();
        header2.set_path("readme.md").unwrap();
        header2.set_size(8);
        header2.set_mode(0o644);
        header2.set_cksum();
        tar.append(&header2, "# README".as_bytes()).unwrap();

        tar.finish().unwrap();
        txz_path
    }

    fn create_test_tar_bz2(dir: &std::path::Path) -> PathBuf {
        let tbz2_path = dir.join("test.tar.bz2");
        let file = File::create(&tbz2_path).unwrap();
        let bz2 = bzip2::write::BzEncoder::new(file, bzip2::Compression::default());
        let mut tar = tar::Builder::new(bz2);

        let mut header = tar::Header::new_gnu();
        header.set_path("bz2_test.txt").unwrap();
        header.set_size(15);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, "BZ2 compressed!".as_bytes()).unwrap();

        tar.finish().unwrap();
        tbz2_path
    }

    fn create_test_zip(dir: &std::path::Path) -> PathBuf {
        let zip_path = dir.join("test.zip");
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("zip_test.txt", options).unwrap();
        zip.write_all("ZIP archive content".as_bytes()).unwrap();

        zip.start_file("docs/info.txt", options).unwrap();
        zip.write_all("Documentation file".as_bytes()).unwrap();

        zip.finish().unwrap();
        zip_path
    }

    fn verify_extracted_content(output_dir: &std::path::Path, expected_files: &[(&str, &str)]) {
        for (file_path, expected_content) in expected_files {
            let full_path = output_dir.join(file_path);
            assert!(full_path.exists(), "File should exist: {}", file_path);
            let content = fs::read_to_string(&full_path).unwrap();
            assert_eq!(
                content, *expected_content,
                "Content mismatch for {}",
                file_path
            );
        }
    }

    #[test]
    fn test_extract_tar() {
        let temp_dir = TempDir::new().unwrap();
        let tar_path = create_test_tar(temp_dir.path());
        let output_dir = temp_dir.path().join("output_tar");

        extract_archive(
            tar_path.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            true,
        )
        .unwrap();

        verify_extracted_content(
            &output_dir,
            &[
                ("test.txt", "Hello, World!"),
                ("subdir/nested.txt", "Nested content!"),
            ],
        );
    }

    #[test]
    fn test_extract_tar_gz() {
        let temp_dir = TempDir::new().unwrap();
        let tgz_path = create_test_tar_gz(temp_dir.path());
        let output_dir = temp_dir.path().join("output_tgz");

        extract_archive(
            tgz_path.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            true,
        )
        .unwrap();

        verify_extracted_content(
            &output_dir,
            &[
                ("test.txt", "Hello, World!"),
                ("data/config.json", "{\"key\": \"value\"}"),
            ],
        );
    }

    #[test]
    fn test_extract_tgz() {
        let temp_dir = TempDir::new().unwrap();
        let tgz_path = create_test_tgz(temp_dir.path());
        let output_dir = temp_dir.path().join("output_tgz2");

        extract_archive(
            tgz_path.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            true,
        )
        .unwrap();

        verify_extracted_content(&output_dir, &[("archive.txt", "TGZ archive!")]);
    }

    #[test]
    fn test_extract_tar_xz() {
        let temp_dir = TempDir::new().unwrap();
        let txz_path = create_test_tar_xz(temp_dir.path());
        let output_dir = temp_dir.path().join("output_txz");

        extract_archive(
            txz_path.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            true,
        )
        .unwrap();

        verify_extracted_content(
            &output_dir,
            &[("xz_test.txt", "XZ compressed!"), ("readme.md", "# README")],
        );
    }

    #[test]
    fn test_extract_tar_bz2() {
        let temp_dir = TempDir::new().unwrap();
        let tbz2_path = create_test_tar_bz2(temp_dir.path());
        let output_dir = temp_dir.path().join("output_tbz2");

        extract_archive(
            tbz2_path.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            true,
        )
        .unwrap();

        verify_extracted_content(&output_dir, &[("bz2_test.txt", "BZ2 compressed!")]);
    }

    #[test]
    fn test_extract_zip() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = create_test_zip(temp_dir.path());
        let output_dir = temp_dir.path().join("output_zip");

        extract_archive(
            zip_path.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            true,
        )
        .unwrap();

        verify_extracted_content(
            &output_dir,
            &[
                ("zip_test.txt", "ZIP archive content"),
                ("docs/info.txt", "Documentation file"),
            ],
        );
    }

    #[test]
    fn test_unsupported_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rar");
        fs::write(&file_path, "fake content").unwrap();

        let result = extract_archive(
            file_path.to_str().unwrap(),
            temp_dir.path().join("out").to_str().unwrap(),
            true,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported archive format"));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }
}
