# Format Detection Plan

## Goal

Add content-based (magic number) format detection and a `--format` override so that `untar` can extract archives whose file extension is missing, wrong, or misleading.

## Scope

- All formats currently supported by `untar` should be detectable when they have a reliable magic number.
- Formats without a fixed magic number (`br`, `lzma`) will continue to rely on extension or `--format`.
- Compressed tar variants (`.tar.gz`, `.tar.bz2`, ...) and single-stream variants (`.gz`, `.bz2`, ...) share the same compression magic, so disambiguation uses the extension when available.

## Implementation

### 1. New module: `rust/src/archive/format.rs`

Introduce a `Format` enum and helper functions:

```rust
pub enum Format {
    TarGz, TarXz, TarBz2, TarLzma, TarZst, TarLz4, TarBr,
    Tar,
    Zip, SevenZ, Rar, Cab, Ar, Cpio, Iso, Xar, Lha, Lzh,
    Gz, Bz2, Xz, Zst, Lz4, Br, Lzma,
}

pub fn detect_format(file_path: &Path) -> Result<Format>;
```

Detection rules (read the first 264 bytes to cover tar magic at offset 257):

| Format(s) | Magic/Heuristic |
|-----------|-----------------|
| ZIP | starts with `PK\x03\x04` or `PK\x05\x06` |
| 7z | starts with `7z\xbc\xaf\x27\x1c` |
| RAR | starts with `Rar!` |
| CAB | starts with `MSCF` |
| AR | starts with `!<arch>\n` |
| CPIO | starts with `070701` or `070702` |
| XAR | starts with `xar!` |
| LHA / LZH | starts with a known `-lh0-`/`-lh1-`/`-lh5-`/`-lz4-` method ID |
| ISO | at offset 0x8000 (sector 16) the bytes are `CD001` with valid header |
| Tar | at offset 257 the bytes are `ustar` (and version `00` at 263) |
| GZ | starts with `0x1f 0x8b` |
| BZ2 | starts with `BZh` |
| XZ | starts with `\xFD7zXZ\x00` |
| ZST | starts with little-endian `0x28B52FFD` |
| LZ4 | starts with little-endian `0x184D2204` |
| BR | no fixed magic; rely on extension / `--format` |
| LZMA | no fixed magic; rely on extension / `--format` |

For GZ/BZ2/XZ/ZST/LZ4 compressed streams, after magic detection, use the file extension to decide between the compressed-tar variant and the single-stream variant. If the extension is missing or unrecognized, default to the single-stream variant.

### 2. Update `rust/src/extract.rs`

- Replace the current extension-only dispatch with a `resolve_format(file_path, ext_hint, cli_format)` helper.
- Order of precedence:
  1. CLI `--format` override.
  2. Content-based magic detection.
  3. File extension as a fallback.
- When the detected format conflicts with the extension, prefer the detected format and emit a warning unless `--quiet` is set.

### 3. Update `rust/src/cli.rs`

Add a new option:

```rust
#[arg(long, value_name = "FORMAT")]
format: Option<String>,
```

Accepted values map to `Format` variants, e.g. `tar.gz`, `zip`, `7z`, `rar`, `cab`, `ar`, `cpio`, `iso`, `xar`, `lha`, `gz`, `bz2`, `xz`, `zst`, `lz4`, `br`, `lzma`, `tar`.

### 4. Update `rust/src/archive/mod.rs`

Export the new `format` module:

```rust
pub mod format;
```

### 5. Tests

Add integration tests in `rust/tests/integration.rs`:

- Rename a `.zip` file to `.tar.gz` and verify extraction succeeds as ZIP.
- Strip the extension from a `.gz` file and verify extraction as gzip stream.
- Use `--format=zip` on a file with the wrong extension.
- Verify that a misnamed `.tar.gz` still works via magic + extension.

### 6. Documentation

- Update `README.md` to mention automatic format detection and `--format`.
- Update `docs/supported-formats.md` with the same note.
- Mark this plan as completed once implemented.

## Excluded

- Heuristic content inspection beyond magic numbers (e.g., trying to parse ambiguous data as tar).
- `.br` and `.lzma` content detection (no reliable magic).
