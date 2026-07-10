use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
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

fn create_password_7z(
    dir: &std::path::Path,
    name: &str,
    password: &str,
    files: &[(&str, &str)],
) -> std::path::PathBuf {
    let path = dir.join(name);
    let input = dir.join("7z_input");
    for (name, content) in files {
        let file_path = input.join(name);
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, content).unwrap();
    }
    std::process::Command::new("7z")
        .current_dir(&input)
        .arg("a")
        .arg(format!("-p{password}"))
        .arg("-mhe=on")
        .arg(&path)
        .arg(".")
        .status()
        .expect("7z failed to create password archive");
    path
}

#[derive(Copy, Clone)]
enum Compression {
    Gz,
    Bz2,
    Xz,
    Lzma,
    Lz,
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
            let mut enc = liblzma::write::XzEncoder::new(Vec::new(), 6);
            enc.write_all(input).unwrap();
            enc.finish().unwrap()
        }
        Compression::Lzma => {
            let mut out = Vec::new();
            lzma_rs::lzma_compress(&mut std::io::Cursor::new(input), &mut out).unwrap();
            out
        }
        Compression::Lz => {
            let mut enc =
                lzma_rust2::LzipWriter::new(Vec::new(), lzma_rust2::LzipOptions::default());
            enc.write_all(input).unwrap();
            enc.finish().unwrap()
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

fn create_ar(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut builder = ar::Builder::new(Vec::new());
    for (name, content) in files {
        let mut header = ar::Header::new(name.as_bytes().to_vec(), content.len() as u64);
        header.set_mode(0o100644);
        builder.append(&header, content.as_bytes()).unwrap();
    }
    fs::write(&path, builder.into_inner().unwrap()).unwrap();
    path
}

fn create_cpio(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut inputs: Vec<(cpio::NewcBuilder, std::io::Cursor<Vec<u8>>)> = Vec::new();
    for (idx, (name, content)) in files.iter().enumerate() {
        let builder = cpio::NewcBuilder::new(*name)
            .ino(idx as u32 + 1)
            .uid(1000)
            .gid(1000)
            .mode(0o100644);
        inputs.push((builder, std::io::Cursor::new(content.as_bytes().to_vec())));
    }
    let output = cpio::write_cpio(inputs.into_iter(), Vec::new()).unwrap();
    fs::write(&path, output).unwrap();
    path
}

fn create_cab(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut cab_builder = cab::CabinetBuilder::new();
    let folder = cab_builder.add_folder(cab::CompressionType::None);
    for (name, _) in files {
        folder.add_file(*name);
    }
    let cab_file = File::create(&path).unwrap();
    let mut cab_writer = cab_builder.build(cab_file).unwrap();
    let mut idx = 0;
    while let Some(mut writer) = cab_writer.next_file().unwrap() {
        let (_, content) = files[idx];
        writer.write_all(content.as_bytes()).unwrap();
        idx += 1;
    }
    cab_writer.finish().unwrap();
    path
}

fn create_xar(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut toc = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xar>
  <toc>"#,
    );
    let mut offset = 0u64;
    let mut heap = Vec::new();
    for (id, (name, content)) in files.iter().enumerate() {
        let len = content.len() as u64;
        toc.push_str(&format!(
            "\n    <file id=\"{}\">\n      <name>{}</name>\n      <type>file</type>\n      <data>\n        <offset>{}</offset>\n        <length>{}</length>\n        <size>{}</size>\n        <encoding style=\"application/octet-stream\"/>\n      </data>\n    </file>",
            id + 1,
            name,
            offset,
            len,
            len
        ));
        heap.extend_from_slice(content.as_bytes());
        offset += len;
    }
    toc.push_str("\n  </toc>\n</xar>");
    let toc_xml = toc.into_bytes();

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&toc_xml).unwrap();
    let compressed_toc = encoder.finish().unwrap();

    let mut xar_buf = Vec::new();
    xar_buf.extend_from_slice(&0x78617221u32.to_be_bytes());
    xar_buf.extend_from_slice(&28u16.to_be_bytes());
    xar_buf.extend_from_slice(&1u16.to_be_bytes());
    xar_buf.extend_from_slice(&(compressed_toc.len() as u64).to_be_bytes());
    xar_buf.extend_from_slice(&(toc_xml.len() as u64).to_be_bytes());
    xar_buf.extend_from_slice(&0u32.to_be_bytes());
    xar_buf.extend_from_slice(&compressed_toc);
    xar_buf.extend_from_slice(&heap);

    fs::write(&path, xar_buf).unwrap();
    path
}

fn create_iso(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let path = dir.join(name);
    let input = dir.join("iso_input");
    for (name, content) in files {
        let file_path = input.join(name);
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, content).unwrap();
    }
    std::process::Command::new("xorriso")
        .arg("-as")
        .arg("mkisofs")
        .arg("-R")
        .arg("-J")
        .arg("-o")
        .arg(&path)
        .arg(&input)
        .status()
        .expect("xorriso failed to create ISO");
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
fn extracts_apk() {
    let tmp = TempDir::new().unwrap();
    let archive = create_zip(tmp.path(), "test.apk", &[("a.txt", "A"), ("b/c.txt", "C")]);

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
fn extracts_jar() {
    let tmp = TempDir::new().unwrap();
    let archive = create_zip(tmp.path(), "test.jar", &[("a.txt", "A")]);

    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(output.join("a.txt")).unwrap(), "A");
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
fn extracts_tar_lz() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_compressed(
        tmp.path(),
        "test.tar.lz",
        &[("a.txt", "A"), ("b/c.txt", "C")],
        Compression::Lz,
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

fn assert_extracts_fixture(
    archive: &std::path::Path,
    output: &std::path::Path,
    expected_file: &str,
) {
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(output)
        .arg(archive)
        .assert()
        .success();
    let expected = fs::read_to_string("tests/fixtures/unarc/LICENSE.unarc-rs")
        .unwrap()
        .replace("\r\n", "\n");
    let actual = fs::read_to_string(output.join(expected_file))
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(actual, expected);
}

#[test]
fn extracts_z() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("out");
    assert_extracts_fixture(
        Path::new("tests/fixtures/unarc/LICENSE.Z"),
        &output,
        "LICENSE",
    );
}

#[test]
fn extracts_tar_z() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("out");
    assert_extracts_fixture(
        Path::new("tests/fixtures/unarc/license.tar.Z"),
        &output,
        "LICENSE",
    );
}

#[test]
fn extracts_arc() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("out");
    assert_extracts_fixture(
        Path::new("tests/fixtures/unarc/store.arc"),
        &output,
        "LICENSE",
    );
}

#[test]
fn extracts_ace() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("out");
    assert_extracts_fixture(
        Path::new("tests/fixtures/unarc/license1.ace"),
        &output,
        "LICENSE",
    );
}

#[test]
fn extracts_zoo() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("out");
    assert_extracts_fixture(
        Path::new("tests/fixtures/unarc/store.zoo"),
        &output,
        "license",
    );
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

#[test]
fn extracts_lz_stream() {
    let tmp = TempDir::new().unwrap();
    let archive = create_stream(tmp.path(), "test.txt.lz", b"Hello", Compression::Lz);
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
fn extracts_ar() {
    let tmp = TempDir::new().unwrap();
    let archive = create_ar(tmp.path(), "test.a", &[("a.txt", "A"), ("b/c.txt", "C")]);
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
fn extracts_cpio() {
    let tmp = TempDir::new().unwrap();
    let archive = create_cpio(
        tmp.path(),
        "test.cpio",
        &[("./a.txt", "A"), ("./b/c.txt", "C")],
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
fn extracts_cab() {
    let tmp = TempDir::new().unwrap();
    let archive = create_cab(tmp.path(), "test.cab", &[("a.txt", "A"), ("b/c.txt", "C")]);
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
fn extracts_xar() {
    let tmp = TempDir::new().unwrap();
    let archive = create_xar(tmp.path(), "test.xar", &[("a.txt", "A"), ("b/c.txt", "C")]);
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

fn xorriso_available() -> bool {
    std::process::Command::new("xorriso")
        .arg("--version")
        .output()
        .is_ok()
}

#[test]
fn extracts_iso() {
    if !xorriso_available() {
        eprintln!("skipping extracts_iso: xorriso not found in PATH");
        return;
    }
    let tmp = TempDir::new().unwrap();
    let archive = create_iso(tmp.path(), "test.iso", &[("A.TXT", "A"), ("B/C.TXT", "C")]);
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(output.join("A.TXT")).unwrap(), "A");
    assert_eq!(fs::read_to_string(output.join("B/C.TXT")).unwrap(), "C");
}

#[test]
fn extracts_password_7z() {
    let tmp = TempDir::new().unwrap();
    let archive = create_password_7z(
        tmp.path(),
        "test.7z",
        "secret",
        &[("a.txt", "A"), ("b/c.txt", "C")],
    );
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
    assert_eq!(fs::read_to_string(output.join("b/c.txt")).unwrap(), "C");
}

#[test]
fn extracts_empty_tar_gz() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_gz(tmp.path(), "empty.tar.gz", &[]);
    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();
    assert!(output.exists());
}

#[test]
fn lists_empty_archive() {
    let tmp = TempDir::new().unwrap();
    let archive = create_zip(tmp.path(), "empty.zip", &[]);
    Command::cargo_bin("untar")
        .unwrap()
        .arg("--list")
        .arg(&archive)
        .assert()
        .success()
        .stdout(predicate::str::contains("Extracted").not());
}

#[test]
fn extracts_unicode_filenames() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_gz(
        tmp.path(),
        "unicode.tar.gz",
        &[("中文.txt", "Chinese"), ("🎉.txt", "Party")],
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
        fs::read_to_string(output.join("中文.txt")).unwrap(),
        "Chinese"
    );
    assert_eq!(fs::read_to_string(output.join("🎉.txt")).unwrap(), "Party");
}

#[test]
fn extracts_nested_directories() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_gz(
        tmp.path(),
        "nested.tar.gz",
        &[("a/b/c/d.txt", "Deep"), ("a/b/other.txt", "Other")],
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
        fs::read_to_string(output.join("a/b/c/d.txt")).unwrap(),
        "Deep"
    );
    assert_eq!(
        fs::read_to_string(output.join("a/b/other.txt")).unwrap(),
        "Other"
    );
}

#[test]
fn detects_zip_renamed_as_tar_gz() {
    let tmp = TempDir::new().unwrap();
    let zip_path = create_zip(tmp.path(), "real.zip", &[("a.txt", "A"), ("b/c.txt", "C")]);
    let archive = tmp.path().join("fake.tar.gz");
    fs::copy(&zip_path, &archive).unwrap();

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
fn detects_gz_stream_without_extension() {
    let tmp = TempDir::new().unwrap();
    let gz_path = create_stream(tmp.path(), "data.gz", b"Hello", Compression::Gz);
    let archive = tmp.path().join("data");
    fs::copy(&gz_path, &archive).unwrap();

    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(output.join("data")).unwrap(), "Hello");
}

#[test]
fn format_override_forces_zip() {
    let tmp = TempDir::new().unwrap();
    let zip_path = create_zip(tmp.path(), "real.zip", &[("a.txt", "A")]);
    let archive = tmp.path().join("archive.txt");
    fs::copy(&zip_path, &archive).unwrap();

    let output = tmp.path().join("out");
    Command::cargo_bin("untar")
        .unwrap()
        .arg("-d")
        .arg(&output)
        .arg("--format")
        .arg("zip")
        .arg(&archive)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(output.join("a.txt")).unwrap(), "A");
}

fn create_deb(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let mut control_tar = Vec::new();
    {
        let mut tar = tar::Builder::new(&mut control_tar);
        let mut header = tar::Header::new_gnu();
        header.set_path("control").unwrap();
        header.set_size(0);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, &[][..]).unwrap();
        tar.finish().unwrap();
    }
    let control_tar_gz = compress_bytes(&control_tar, Compression::Gz);

    let mut data_tar = Vec::new();
    {
        let mut tar = tar::Builder::new(&mut data_tar);
        for (name, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_path(*name).unwrap();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, content.as_bytes()).unwrap();
        }
        tar.finish().unwrap();
    }
    let data_tar_gz = compress_bytes(&data_tar, Compression::Gz);

    let path = dir.join(name);
    let mut builder = ar::Builder::new(Vec::new());
    for (name, content) in [
        ("debian-binary", "2.0\n".as_bytes()),
        ("control.tar.gz", control_tar_gz.as_slice()),
        ("data.tar.gz", data_tar_gz.as_slice()),
    ] {
        let mut header = ar::Header::new(name.as_bytes().to_vec(), content.len() as u64);
        header.set_mode(0o100644);
        builder.append(&header, content).unwrap();
    }
    fs::write(&path, builder.into_inner().unwrap()).unwrap();
    path
}

fn create_squashfs(
    dir: &std::path::Path,
    name: &str,
    files: &[(&str, &str)],
) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut fs = backhand::FilesystemWriter::default();
    fs.set_only_root_id();
    fs.set_root_mode(0o755);
    fs.set_compressor(
        backhand::FilesystemCompressor::new(backhand::compression::Compressor::Gzip, None).unwrap(),
    );

    for (name, content) in files {
        let file_path = std::path::Path::new(*name);
        if let Some(parent) = file_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs.push_dir_all(parent, backhand::NodeHeader::new(0o755, 0, 0, 0))
                    .unwrap();
            }
        }
        fs.push_file(
            std::io::Cursor::new(content.as_bytes()),
            *name,
            backhand::NodeHeader::new(0o644, 0, 0, 0),
        )
        .unwrap();
    }

    let mut output = File::create(&path).unwrap();
    fs.write(&mut output).unwrap();
    path
}

fn create_rpm(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut builder = rpm::PackageBuilder::new("test", "1.0.0", "MIT", "x86_64", "test package");
    for (name, content) in files {
        let rpm_path = if name.starts_with('/') {
            name.to_string()
        } else {
            format!("/{name}")
        };
        builder
            .with_file_contents(
                content.as_bytes(),
                rpm::FileOptions::new(rpm_path).permissions(0o644),
            )
            .unwrap();
    }
    let pkg = builder.build().unwrap();
    pkg.write_to(&path).unwrap();
    path
}

fn create_lzo(dir: &std::path::Path, name: &str, content: &[u8]) -> std::path::PathBuf {
    const LZOP_MAGIC: &[u8] = b"\x89LZO\x00\r\n\x1a\n";
    let path = dir.join(name);
    let mut out = Vec::new();
    out.extend_from_slice(LZOP_MAGIC);
    let header_start = out.len();
    out.extend_from_slice(&0x1040u16.to_be_bytes());
    out.extend_from_slice(&0x2040u16.to_be_bytes());
    out.extend_from_slice(&0x0940u16.to_be_bytes());
    out.push(1); // method
    out.push(1); // level
    out.extend_from_slice(&0u32.to_be_bytes()); // flags
    out.extend_from_slice(&0u32.to_be_bytes()); // mode
    out.extend_from_slice(&0u32.to_be_bytes()); // mtime_low
    out.extend_from_slice(&0u32.to_be_bytes()); // mtime_high
    out.push(0); // name_len
    let mut hasher = adler::Adler32::new();
    hasher.write_slice(&out[header_start..]);
    let checksum = hasher.checksum();
    out.extend_from_slice(&checksum.to_be_bytes());

    let compressed = lzokay::compress::compress(content).unwrap();
    let stored = if compressed.len() < content.len() {
        compressed.as_slice()
    } else {
        content
    };
    out.extend_from_slice(&(content.len() as u32).to_be_bytes());
    out.extend_from_slice(&(stored.len() as u32).to_be_bytes());
    out.extend_from_slice(stored);
    out.extend_from_slice(&0u32.to_be_bytes()); // end block

    fs::write(&path, out).unwrap();
    path
}

fn create_tar_lzo(dir: &std::path::Path, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let mut tar_buf = Vec::new();
    {
        let mut tar = tar::Builder::new(&mut tar_buf);
        for (name, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_path(*name).unwrap();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, content.as_bytes()).unwrap();
        }
        tar.finish().unwrap();
    }
    create_lzo(dir, name, &tar_buf)
}

#[test]
fn extracts_deb() {
    let tmp = TempDir::new().unwrap();
    let archive = create_deb(tmp.path(), "test.deb", &[("a.txt", "A"), ("b/c.txt", "C")]);
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
fn extracts_squashfs() {
    let tmp = TempDir::new().unwrap();
    let archive = create_squashfs(
        tmp.path(),
        "test.squashfs",
        &[("a.txt", "A"), ("b/c.txt", "C")],
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
fn extracts_rpm() {
    let tmp = TempDir::new().unwrap();
    let archive = create_rpm(tmp.path(), "test.rpm", &[("a.txt", "A"), ("b/c.txt", "C")]);
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
fn extracts_lzo_stream() {
    let tmp = TempDir::new().unwrap();
    let archive = create_lzo(tmp.path(), "test.txt.lzo", b"Hello");
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
fn extracts_tar_lzo() {
    let tmp = TempDir::new().unwrap();
    let archive = create_tar_lzo(
        tmp.path(),
        "test.tar.lzo",
        &[("a.txt", "A"), ("b/c.txt", "C")],
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
