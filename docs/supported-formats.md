# Supported Formats

`untar` supports a broad range of archive and compression formats. Formats are detected by file extension, so archives should use the correct extension.

## Archive Formats

| Format | Extension(s) | Crate | Notes |
|--------|--------------|-------|-------|
| Tar | `.tar` | [tar](https://crates.io/crates/tar) | Uncompressed tar archive |
| Gzip-compressed Tar | `.tar.gz`, `.tgz` | [tar](https://crates.io/crates/tar) + [flate2](https://crates.io/crates/flate2) | |
| XZ-compressed Tar | `.tar.xz`, `.txz` | [tar](https://crates.io/crates/tar) + [xz2](https://crates.io/crates/xz2) | |
| BZip2-compressed Tar | `.tar.bz2`, `.tbz2`, `.tbz` | [tar](https://crates.io/crates/tar) + [bzip2](https://crates.io/crates/bzip2) | |
| LZMA-compressed Tar | `.tar.lzma`, `.tlz` | [tar](https://crates.io/crates/tar) + [lzma-rs](https://crates.io/crates/lzma-rs) | Decompressed to a temporary file before tar extraction |
| Zstandard-compressed Tar | `.tar.zst`, `.tzst` | [tar](https://crates.io/crates/tar) + [ruzstd](https://crates.io/crates/ruzstd) | |
| LZ4-compressed Tar | `.tar.lz4` | [tar](https://crates.io/crates/tar) + [lz4_flex](https://crates.io/crates/lz4_flex) | |
| Brotli-compressed Tar | `.tar.br` | [tar](https://crates.io/crates/tar) + [brotli-decompressor](https://crates.io/crates/brotli-decompressor) | |
| ZIP | `.zip` | [zip](https://crates.io/crates/zip) | Supports AES password-protected archives (`--password`) |
| 7-Zip | `.7z` | [sevenz-rust2](https://crates.io/crates/sevenz-rust2) | Supports password-protected archives (`--password`) |
| RAR | `.rar` | [rars](https://crates.io/crates/rars) | Decode-only |
| Cabinet | `.cab` | [cab](https://crates.io/crates/cab) | Windows Cabinet files; directory separators are normalized to `/` |
| Unix Archive | `.ar`, `.a` | [ar](https://crates.io/crates/ar) | Common, BSD, and GNU variants |
| CPIO | `.cpio` | [cpio](https://crates.io/crates/cpio) | `newc` (SVR4) format only |
| ISO 9660 | `.iso` | [iso9660-rs](https://crates.io/crates/iso9660-rs) | Primary volume names; Joliet/Rock Ridge are detected but not used for filenames |
| XAR | `.xar` | [xara](https://crates.io/crates/xara) | macOS `.pkg` payload; symlinks are skipped |
| LHA/LZH | `.lha`, `.lzh` | [delharc](https://crates.io/crates/delharc) | Level 0-3 headers; common compression methods |

## Single-Stream Compression Formats

These formats contain exactly one compressed file. `untar` decompresses the input and writes the result to a file with the compression extension removed (e.g., `file.txt.gz` becomes `file.txt`).

| Format | Extension | Crate | Notes |
|--------|-----------|-------|-------|
| Gzip | `.gz` | [flate2](https://crates.io/crates/flate2) | |
| BZip2 | `.bz2` | [bzip2](https://crates.io/crates/bzip2) | |
| XZ | `.xz` | [xz2](https://crates.io/crates/xz2) | |
| Zstandard | `.zst` | [ruzstd](https://crates.io/crates/ruzstd) | |
| LZ4 | `.lz4` | [lz4_flex](https://crates.io/crates/lz4_flex) | |
| Brotli | `.br` | [brotli-decompressor](https://crates.io/crates/brotli-decompressor) | |
| LZMA | `.lzma` | [lzma-rs](https://crates.io/crates/lzma-rs) | |

## Explicitly Unsupported Formats

These formats are intentionally excluded because there is no mature, pure-Rust implementation available:

- `.tar.lz` (LZIP)
- `.tar.Z` (Unix `compress`)
- `.ace`
- `.arc`
- `.zoo`

## Common Options

All formats support the following shared options:

- `-d, --directory <DIR>`: Output directory
- `-q, --quiet`: Suppress progress output
- `-l, --list`: List archive contents instead of extracting
- `--on-exists <POLICY>`: How to handle existing files (`ask`, `error`, `overwrite`, `skip`, `rename`)
- `--rename-suffix <SUFFIX>`: Suffix for renamed files (default `.1`)
- `--strip-components <N>`: Strip the first N leading path components
- `--password <PASSWORD>`: Password for ZIP and 7-Zip archives

## Path Safety

All archive paths are normalized and rejected if they contain absolute paths or `..` components, preventing Zip Slip / path traversal attacks.
