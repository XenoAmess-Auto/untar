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
    let archive = tmp.path().join("test.rar");
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
fn shows_help() {
    Command::cargo_bin("untar")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Extract"));
}
