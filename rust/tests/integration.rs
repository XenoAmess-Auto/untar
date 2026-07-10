use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

fn create_tar_gz(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let path = dir.join(name);
    let file = File::create(&path).unwrap();
    let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(gz);

    for (name, content) in files {
        let mut header = tar::Header::new_gnu();
        header.set_path(name).unwrap();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, content.as_bytes()).unwrap();
    }

    tar.finish().unwrap();
    path
}

fn create_zip(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let path = dir.join(name);
    let file = File::create(&path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for (name, content) in files {
        zip.start_file(*name, options).unwrap();
        zip.write_all(content.as_bytes()).unwrap();
    }

    zip.finish().unwrap();
    path
}

fn create_password_zip(
    dir: &std::path::Path,
    name: &str,
    password: &str,
    files: &[(&str, &str)],
) -> std::path::PathBuf {
    let path = dir.join(name);
    let file = File::create(&path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .with_aes_encryption(zip::AesMode::Aes256, password);

    for (name, content) in files {
        zip.start_file(*name, options).unwrap();
        zip.write_all(content.as_bytes()).unwrap();
    }

    zip.finish().unwrap();
    path
}

fn create_7z(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let path = dir.join(name);
    let input = dir.join("7z_input");
    for (name, content) in files {
        let file_path = input.join(name);
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, content).unwrap();
    }
    sevenz_rust2::compress_to_path(&input, &path).unwrap();
    path
}

#[derive(Copy, Clone)]
enum Compression {
    Gz,
    Bz2,
    Xz,
    Lzma,
    Zst,
    Lz4,
    Br,
}

fn compress_bytes(input: &[u8], kind: Compression) -> Vec<u8> {
    match kind {
        Compression::Gz => {
            let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            enc.write_all(input).unwrap();
            enc.finish().unwrap()
        }
        Compression::Bz2 => {
            let mut enc = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::default());
            enc.write_all(input).unwrap();
            enc.finish().unwrap()
        }
        Compression::Xz => {
            let mut enc = xz2::write::XzEncoder::new(Vec::new(), 6);
            enc.write_all(input).unwrap();
            enc.finish().unwrap()
        }
        Compression::Lzma => {
            let mut out = Vec::new();
            lzma_rs::lzma_compress(&mut std::io::Cursor::new(input), &mut out).unwrap();
            out
        }
        Compression::Zst => {
            let mut enc = zstd::stream::write::Encoder::new(Vec::new(), 3).unwrap();
            enc.write_all(input).unwrap();
            enc.finish().unwrap()
        }
        Compression::Lz4 => {
            let mut enc = lz4_flex::frame::FrameEncoder::new(Vec::new());
            enc.write_all(input).unwrap();
            enc.finish().unwrap()
        }
        Compression::Br => {
            let mut enc = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
            enc.write_all(input).unwrap();
            enc.flush().unwrap();
            enc.into_inner()
        }
    }
}

fn create_tar_compressed(
    dir: &std::path::Path,
    name: &str,
    files: &[(&str, &str)],
    kind: Compression,
) -> std::path::PathBuf {
    let mut tar_buf = Vec::new();
    {
        let mut tar = tar::Builder::new(&mut tar_buf);
        for (name, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_path(name).unwrap();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, content.as_bytes()).unwrap();
        }
        tar.finish().unwrap();
    }
    let path = dir.join(name);
    fs::write(&path, compress_bytes(&tar_buf, kind)).unwrap();
    path
}

fn create_stream(
    dir: &std::path::Path,
    name: &str,
    content: &[u8],
    kind: Compression,
) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::write(&path, compress_bytes(content, kind)).unwrap();
    path
}

#[test]
fn extracts_tar_gz() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_gz(
        tmp.path(),
        "test.tar.gz",
        &[("hello.txt", "Hello, world!"), ("dir/nested.txt", "Nested")],
    );

    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(output.join("hello.txt")).unwrap(),
        "Hello, world!"
    );
    assert_eq!(
        fs::read_to_string(output.join("dir/nested.txt")).unwrap(),
        "Nested"
    );
}

#[test]
fn extracts_zip() {
    let tmp = TempDir::new().unwrap();
    let archive = create_zip(tmp.path(), "test.zip", &[("a.txt", "A"), ("b/c.txt", "C")]);

    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(output.join("a.txt")).unwrap(), "A");
    assert_eq!(fs::read_to_string(output.join("b/c.txt")).unwrap(), "C");
}

#[test]
fn rejects_unsupported_format() {
    let tmp = TempDir::new().unwrap();
    let archive = tmp.path().join("test.unknown");
    fs::write(&archive, "not a real archive").unwrap();

    Command::cargo_bin("untar")
        .unwrap()
        .arg(&archive)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported archive format"));
}

#[test]
fn rejects_missing_file() {
    Command::cargo_bin("untar")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("No archive file specified"));
}

#[test]
fn rejects_zip_slip() {
    let tmp = TempDir::new().unwrap();
    // Create a zip with an entry that escapes the output directory.
    let archive = tmp.path().join("evil.zip");
    let file = File::create(&archive).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("../evil.txt", options).unwrap();
    zip.write_all(b"pwned").unwrap();
    zip.finish().unwrap();

    let output = tmp.path().join("out");
    fs::create_dir_all(&output).unwrap();

    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Path traversal"));
}

#[test]
fn lists_archive_contents() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_gz(
        tmp.path(),
        "test.tar.gz",
        &[("hello.txt", "Hello"), ("dir/nested.txt", "Nested")],
    );

    Command::cargo_bin("untar")
        .unwrap()
        .arg("--list")
        .arg(&archive)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello.txt"))
        .stdout(predicate::str::contains("dir/nested.txt"));
}

#[test]
fn strips_components() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_gz(
        tmp.path(),
        "test.tar.gz",
        &[("a/b/c.txt", "C"), ("a/b/d.txt", "D")],
    );

    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg("--strip-components")
        .arg("2")
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(output.join("c.txt")).unwrap(), "C");
    assert_eq!(fs::read_to_string(output.join("d.txt")).unwrap(), "D");
}

#[test]
fn extracts_matching_pattern() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_gz(
        tmp.path(),
        "test.tar.gz",
        &[("keep.txt", "KEEP"), ("drop.txt", "DROP")],
    );

    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .arg("keep.txt")
        .assert()
        .success();

    assert_eq!(fs::read_to_string(output.join("keep.txt")).unwrap(), "KEEP");
    assert!(!output.join("drop.txt").exists());
}

#[test]
fn skips_existing_files() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_gz(tmp.path(), "test.tar.gz", &[("hello.txt", "NEW")]);

    let output = tmp.path().join("out");
    fs::create_dir_all(&output).unwrap();
    fs::write(output.join("hello.txt"), "OLD").unwrap();

    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg("--on-exists=skip")
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(output.join("hello.txt")).unwrap(), "OLD");
}

#[test]
fn renames_existing_files() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_gz(tmp.path(), "test.tar.gz", &[("hello.txt", "NEW")]);

    let output = tmp.path().join("out");
    fs::create_dir_all(&output).unwrap();
    fs::write(output.join("hello.txt"), "OLD").unwrap();

    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg("--on-exists=rename")
        .arg("--rename-suffix=.new")
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(output.join("hello.txt")).unwrap(), "OLD");
    assert_eq!(
        fs::read_to_string(output.join("hello.txt.new")).unwrap(),
        "NEW"
    );
}

#[test]
fn shows_version() {
    Command::cargo_bin("untar")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("untar"));
}

#[test]
fn extracts_7z() {
    let tmp = TempDir::new().unwrap();
    let archive = create_7z(tmp.path(), "test.7z", &[("a.txt", "A"), ("b/c.txt", "C")]);

    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(output.join("a.txt")).unwrap(), "A");
    assert_eq!(fs::read_to_string(output.join("b/c.txt")).unwrap(), "C");
}

#[test]
fn extracts_password_zip() {
    let tmp = TempDir::new().unwrap();
    let archive = create_password_zip(tmp.path(), "test.zip", "secret", &[("a.txt", "A")]);

    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg("--password")
        .arg("secret")
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(output.join("a.txt")).unwrap(), "A");
}

#[test]
fn shows_help() {
    Command::cargo_bin("untar")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Extract"));
}

#[test]
fn extracts_tar_lzma() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_compressed(
        tmp.path(),
        "test.tar.lzma",
        &[("a.txt", "A"), ("b/c.txt", "C")],
        Compression::Lzma,
    );
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(output.join("a.txt")).unwrap(), "A");
    assert_eq!(fs::read_to_string(output.join("b/c.txt")).unwrap(), "C");
}

#[test]
fn extracts_tar_zst() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_compressed(
        tmp.path(),
        "test.tar.zst",
        &[("a.txt", "A"), ("b/c.txt", "C")],
        Compression::Zst,
    );
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(output.join("a.txt")).unwrap(), "A");
    assert_eq!(fs::read_to_string(output.join("b/c.txt")).unwrap(), "C");
}

#[test]
fn extracts_tar_lz4() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_compressed(
        tmp.path(),
        "test.tar.lz4",
        &[("a.txt", "A"), ("b/c.txt", "C")],
        Compression::Lz4,
    );
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(output.join("a.txt")).unwrap(), "A");
    assert_eq!(fs::read_to_string(output.join("b/c.txt")).unwrap(), "C");
}

#[test]
fn extracts_tar_br() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_compressed(
        tmp.path(),
        "test.tar.br",
        &[("a.txt", "A"), ("b/c.txt", "C")],
        Compression::Br,
    );
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(output.join("a.txt")).unwrap(), "A");
    assert_eq!(fs::read_to_string(output.join("b/c.txt")).unwrap(), "C");
}

#[test]
fn extracts_gz_stream() {
    let tmp = TempDir::new().unwrap();
    let archive = create_stream(tmp.path(), "test.txt.gz", b"Hello", Compression::Gz);
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(output.join("test.txt")).unwrap(),
        "Hello"
    );
}

#[test]
fn extracts_bz2_stream() {
    let tmp = TempDir::new().unwrap();
    let archive = create_stream(tmp.path(), "test.txt.bz2", b"Hello", Compression::Bz2);
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(output.join("test.txt")).unwrap(),
        "Hello"
    );
}

#[test]
fn extracts_xz_stream() {
    let tmp = TempDir::new().unwrap();
    let archive = create_stream(tmp.path(), "test.txt.xz", b"Hello", Compression::Xz);
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(output.join("test.txt")).unwrap(),
        "Hello"
    );
}

#[test]
fn extracts_zst_stream() {
    let tmp = TempDir::new().unwrap();
    let archive = create_stream(tmp.path(), "test.txt.zst", b"Hello", Compression::Zst);
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(output.join("test.txt")).unwrap(),
        "Hello"
    );
}

#[test]
fn extracts_lz4_stream() {
    let tmp = TempDir::new().unwrap();
    let archive = create_stream(tmp.path(), "test.txt.lz4", b"Hello", Compression::Lz4);
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(output.join("test.txt")).unwrap(),
        "Hello"
    );
}

#[test]
fn extracts_br_stream() {
    let tmp = TempDir::new().unwrap();
    let archive = create_stream(tmp.path(), "test.txt.br", b"Hello", Compression::Br);
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(output.join("test.txt")).unwrap(),
        "Hello"
    );
}

#[test]
fn extracts_lzma_stream() {
    let tmp = TempDir::new().unwrap();
    let archive = create_stream(tmp.path(), "test.txt.lzma", b"Hello", Compression::Lzma);
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(output.join("test.txt")).unwrap(),
        "Hello"
    );
}
