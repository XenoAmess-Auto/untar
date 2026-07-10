# Untar Development Plan

## Overview

This document records the phased development plan for the `untar` project.

Current status: Rust implementation in `rust/`, single-file `src/main.rs`, CI/CD with multi-platform release builds and Dependabot auto-merge.

---

## Phase 1: Foundation + Safety

**Status: completed**

### 1.1 Module Refactoring
- [x] Split `rust/src/main.rs` into focused modules:
  - `cli.rs` — CLI argument parsing and help/version text
  - `extract.rs` — high-level extraction orchestration, overwrite policy, path safety
  - `archive/tar.rs` — tar / tar.gz / tar.xz / tar.bz2 extraction
  - `archive/zip.rs` — zip extraction (including password-protected archives)
  - `tests/integration.rs` — end-to-end CLI tests using `assert_cmd`
- [x] Preserve all existing behavior during the move.

### 1.2 Security: Zip Slip / Path Traversal
- [x] Normalize every archive entry path.
- [x] Reject entries containing `..` or absolute paths that escape `output_dir`.
- [x] Add dedicated tests for Zip Slip attempts in both tar and zip.

### 1.3 Help / Version Fix
- [x] Remove hardcoded `1.0.3` from `HELP`.
- [x] Generate help text from `env!("CARGO_PKG_VERSION")` / `clap` built-in long help.
- [x] Remove the redundant `help: bool` field from `Args`; rely on `clap`'s built-in `-h/--help`.

### 1.4 Testing
- [x] Add `assert_cmd` + `predicates` integration tests.
- [x] Cover: successful extraction, overwrite behavior, listing, strip-components, and unsupported formats.
### 1.5 MSRV

**Deferred**: this is a binary CLI tool distributed primarily via pre-built binaries. Requiring a specific MSRV adds maintenance overhead (dependencies like `clap` already move to `edition2024`/Rust 1.85+) with limited practical benefit for end users. The project builds with the latest stable Rust; MSRV can be revisited later if downstream distro packaging requires it.


---

## Phase 2: Core CLI Features

**Status: completed**

### 2.1 Overwrite Behavior
- [x] Default in an interactive TTY: prompt the user (`overwrite/skip/rename?`).
- [x] Default in a non-interactive environment: exit with an error unless a policy is specified.
- [x] `--on-exists=overwrite` — silently overwrite existing files.
- [x] `--on-exists=skip` — silently skip existing files.
- [x] `--on-exists=rename` — silently rename by appending a suffix.
- [x] `--rename-suffix=.SUFFIX` — customize the suffix (default `.1`).

### 2.2 List Archive Contents
- [x] `-l, --list <ARCHIVE>`: print file list, size, and permissions without extracting.

### 2.3 Selective Extraction
- [x] `untar archive.tar.gz path/inside/file.txt` extracts only matching entries.
- [x] Prefix matching support (e.g., `untar archive.tar.gz dir/` extracts everything under `dir`).

### 2.4 Strip Components
- [x] `--strip-components N`: strip the first N path components, matching `tar` behavior.

---

## Phase 3: Extended Format Support

### 3.1 Password-Protected ZIP
- `-p, --password <PASSWORD>`: decrypt password-protected ZIP archives using the `zip` crate.

### 3.2 7z Support
- Support `.7z` and `.tar.7z` via `sevenz-rust2` (pure Rust).

### 3.3 RAR Support (Pure Rust)
- Support `.rar` via the `rars` crate (pure Rust high-level API), falling back to `rar` if needed.
- If the pure Rust implementation proves insufficient for production use, document the limitation and skip RAR.

---

## Phase 4: CI/CD + Release Optimization

**Status: completed**

### 4.1 Release Checksums
- [x] Generate `sha256sums.txt` for all release artifacts and upload it alongside the binaries/packages.

### 4.2 CI Cache Optimization
- [x] Cache `~/.cargo/bin` for `cross` and `cargo-wix`.
- [x] Cache `~/go/bin` for `nfpm`.
- [x] Cache Rust dependencies via `rust-cache` already in place.

### 4.3 Documentation & Links
- [x] Fix README clone/badge URLs to point to `XenoAmess-Auto/untar`.
- [x] Keep `docs/dependabot-optimization-notes.md` updated.

### 4.4 MSRV Check in CI
- [x] Removed explicit MSRV enforcement; project builds with the latest stable Rust.

---

## Phase 5: Optional Enhancements

### 5.1 Progress Bar
- Use `indicatif` to show overall extraction progress (disabled by `--quiet`).

### 5.2 Docker Image
- Provide a minimal Dockerfile for containerized usage.

### 5.3 Shell Completion & Man Page
- Generate shell completions via `clap_complete`.
- Generate a man page via `clap_mangen`.

---

## Execution Order

Recommended order: **Phase 1 → Phase 2 → Phase 4 → Phase 3 → Phase 5**.

Phase 1 must come first because it establishes the code structure that later features depend on. Phase 4 is placed before Phase 3 so the release pipeline is stable before adding heavier format dependencies.

---

## Platform Scope

- Linux x86_64 / ARM64
- Windows x86_64
- macOS is **not** required.
