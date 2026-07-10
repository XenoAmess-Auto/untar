# Format Expansion Plan

This document describes the plan to extend `untar` to support additional archive and compression formats.

## Scope

Add support for all common formats that have a viable Rust crate, excluding those already identified as impractical or legacy-only.

Explicitly excluded:
- `.tar.lz` (LZIP): no mature pure-Rust implementation (now supported via `lzma-rust2`)
- `.tar.Z` (Unix compress): no active Rust crate (now supported via `unarc-rs`)
- `.ace`, `.arc`, `.zoo`: no active Rust crates (now supported via `unarc-rs`)

All previously excluded legacy formats are now implemented.

## Phase A: TAR compression extensions and single-stream decompression

### Dependencies
Add the following crates to `rust/Cargo.toml`:

| Crate | Version | Use |
|-------|---------|-----|
| `lzma-rs` | `0.3` | `.tar.lzma` and `.lzma` (pure Rust) |
| `ruzstd` | `0.8` | `.tar.zst` / `.tzst` and `.zst` (pure Rust) |
| `lz4_flex` | `0.13` | `.tar.lz4` and `.lz4` (pure Rust) |
| `brotli-decompressor` | `5` | `.tar.br` and `.br` (pure Rust, decompress only) |

### TAR variants to add
- `.tar.lzma` (via `lzma-rs` decompression to a temporary file, then tar extraction)
- `.tar.zst` / `.tzst` (via `ruzstd::StreamingDecoder`)
- `.tar.lz4` (via `lz4_flex::frame::FrameDecoder`)
- `.tar.br` (via `brotli-decompressor`)

All new TAR variants reuse the existing path-safety, pattern, strip-components, and overwrite logic in `archive/tar.rs`.

### Single-stream decompression
Add a new module `archive/stream.rs` to handle single-file compressed inputs:
- `.gz` (via `flate2`)
- `.bz2` (via `bzip2`)
- `.xz` (via `liblzma`)
- `.zst` (via `ruzstd`)
- `.lz4` (via `lz4_flex`)
- `.br` (via `brotli-decompressor`)
- `.lzma` (via `lzma-rs`)

Behavior: write the decompressed result to a file with the compressed extension removed. For example, `untar file.txt.gz` produces `output_dir/file.txt`.

### Code changes
- `rust/Cargo.toml`: add dependencies
- `rust/src/archive/tar.rs`: add `extract_tar_lzma`, `extract_tar_zst`, `extract_tar_lz4`, `extract_tar_br`
- `rust/src/archive/stream.rs`: new module for single-stream decompression
- `rust/src/archive/mod.rs`: export `stream`
- `rust/src/extract.rs`: extend extension-based dispatch; ensure `.tar.*` checks happen before generic compression checks

## Phase B: Independent archive formats

### Dependencies
Add the following crates to `rust/Cargo.toml`:

| Crate | Version | Use |
|-------|---------|-----|
| `cab` | `0.6` | `.cab` (Windows Cabinet) |
| `ar` | `0.9` | `.ar` (Unix archive) |
| `cpio` | `0.4` | `.cpio` (newc/SVR4) |
| `iso9660-rs` | `1.0` | `.iso` (ISO 9660) |
| `xara` | `0.3` | `.xar` / macOS `.pkg` |
| `delharc` | `0.6.2` | `.lha` / `.lzh` (pinned to 0.6.2 for Rust 1.65 compatibility) |

### Code changes
Create one module per format:
- `rust/src/archive/cab.rs`
- `rust/src/archive/ar.rs`
- `rust/src/archive/cpio.rs`
- `rust/src/archive/iso.rs`
- `rust/src/archive/xar.rs`
- `rust/src/archive/lha.rs`

Each module implements extraction using the same shared helpers (`safe_output_path`, `resolve_conflict`, `strip_path_components`, `should_extract`, `Progress`) to support `--list`, `--strip-components`, patterns, and overwrite policies.

`rust/src/archive/mod.rs` exports the new modules.

`rust/src/extract.rs` dispatches based on extension (e.g., `.cab`, `.ar`, `.cpio`, `.iso`, `.xar`, `.lha`, `.lzh`).

### Special notes
- `iso9660-rs` uses a custom `BlockIo` abstraction; implement a thin `File` adapter.
- `xara` extracts to a `Vec<u8>` or `Write`; stream files directly to the output path.
- `delharc` 0.6.2 exposes `LhaDecodeReader` which implements `Read`.

## Phase C: Tests

For every format added in Phase A and Phase B, add an integration test in `rust/tests/integration.rs` that:
1. Creates a minimal archive of that format.
2. Runs the `untar` binary to extract it.
3. Verifies the extracted contents.

Additional tests:
- `extracts_rar` (currently missing)
- `extracts_password_7z`
- Edge cases: empty archive, Unicode filenames, nested directories, path traversal rejection

## Phase D: Documentation

- Update `README.md` with a `Supported formats` table (extension, supported, notes).
- Create `docs/supported-formats.md` with detailed per-format descriptions, implementation crates, and any limitations.
- Update `docs/development-plan.md` to reference this plan as Phase 6.

## Execution order

1. Write this plan document to `docs/format-expansion-plan.md`.
2. Implement Phase A and its tests.
3. Commit and push.
4. Implement Phase B and its tests.
5. Commit and push.
6. Implement Phase C remaining tests (RAR, password 7z, edge cases).
7. Commit and push.
8. Implement Phase D documentation.
9. Commit and push.
