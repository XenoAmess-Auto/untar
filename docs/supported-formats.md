# Supported Formats

`untar` supports a broad range of archive and compression formats. Most formats are detected by their magic number, so archives work even if the extension is missing or wrong. You can also force a format with `--format <FORMAT>`.

## Archive Formats

| Format | Extension(s) | Crate | Notes |
|--------|--------------|-------|-------|
| Tar | `.tar` | [tar](https://crates.io/crates/tar) | Uncompressed tar archive |
| Gzip-compressed Tar | `.tar.gz`, `.tgz` | [tar](https://crates.io/crates/tar) + [flate2](https://crates.io/crates/flate2) | |
| XZ-compressed Tar | `.tar.xz`, `.txz` | [tar](https://crates.io/crates/tar) + [liblzma](https://crates.io/crates/liblzma) | |
| BZip2-compressed Tar | `.tar.bz2`, `.tbz2`, `.tbz` | [tar](https://crates.io/crates/tar) + [bzip2](https://crates.io/crates/bzip2) | |
| LZMA-compressed Tar | `.tar.lzma`, `.tlz` | [tar](https://crates.io/crates/tar) + [lzma-rs](https://crates.io/crates/lzma-rs) | Decompressed to a temporary file before tar extraction; `.tlz` with non-LZIP header |
| LZIP-compressed Tar | `.tar.lz`, `.tlz` | [tar](https://crates.io/crates/tar) + [lzma-rust2](https://crates.io/crates/lzma-rust2) | `.tlz` with LZIP header |
| Zstandard-compressed Tar | `.tar.zst`, `.tzst` | [tar](https://crates.io/crates/tar) + [ruzstd](https://crates.io/crates/ruzstd) | |
| LZ4-compressed Tar | `.tar.lz4` | [tar](https://crates.io/crates/tar) + [lz4_flex](https://crates.io/crates/lz4_flex) | |
| Brotli-compressed Tar | `.tar.br` | [tar](https://crates.io/crates/tar) + [brotli-decompressor](https://crates.io/crates/brotli-decompressor) | |
| LZO-compressed Tar | `.tar.lzo` | [tar](https://crates.io/crates/tar) + [lzo](https://crates.io/crates/lzo) | lzop file format |
| ZIP | `.zip`, `.apk`, `.jar`, `.war`, `.ear` | [zip](https://crates.io/crates/zip) | Supports AES password-protected archives (`--password`); APK/JAR/WAR/EAR are treated as ZIP |
| 7-Zip | `.7z` | [sevenz-rust2](https://crates.io/crates/sevenz-rust2) | Supports password-protected archives (`--password`) |
| RAR | `.rar` | [rars](https://crates.io/crates/rars) | Decode-only |
| Cabinet | `.cab` | [cab](https://crates.io/crates/cab) | Windows Cabinet files; directory separators are normalized to `/` |
| Unix Archive | `.ar`, `.a` | [ar](https://crates.io/crates/ar) | Common, BSD, and GNU variants |
| CPIO | `.cpio` | [cpio](https://crates.io/crates/cpio) | `newc` (SVR4) format only |
| ISO 9660 | `.iso` | [iso9660-rs](https://crates.io/crates/iso9660-rs) | Primary volume names; Joliet/Rock Ridge are detected but not used for filenames |
| XAR | `.xar` | [xara](https://crates.io/crates/xara) | macOS `.pkg` payload; symlinks are skipped |
| LHA/LZH | `.lha`, `.lzh` | [delharc](https://crates.io/crates/delharc) | Level 0-3 headers; common compression methods |
| Debian package | `.deb` | [ar](https://crates.io/crates/ar) + [tar](https://crates.io/crates/tar) | Extracts the `data.tar.*` payload |
| SquashFS | `.squashfs`, `.sqfs`, `.sfs`, `.snap` | [backhand](https://crates.io/crates/backhand) | gzip/xz/zstd/lz4 compressed images |
| RPM package | `.rpm` | [rpm](https://crates.io/crates/rpm) | gzip/bzip2/xz/zstd payloads |
| POSIX pax | `.pax` | [tar](https://crates.io/crates/tar) | Treated as a tar archive |
| Unix compress | `.tar.Z`, `.taz`, `.Z` | [unarc-rs](https://crates.io/crates/unarc-rs) | LZW (Unix compress) stream; `.tar.Z` extracts the tar contents |
| ACE | `.ace` | [unarc-rs](https://crates.io/crates/unarc-rs) | Decode-only |
| ARC/PAK | `.arc`, `.pak` | [unarc-rs](https://crates.io/crates/unarc-rs) | Decode-only; PAK uses the same reader |
| ARJ | `.arj` | [unarc-rs](https://crates.io/crates/unarc-rs) | Decode-only; supports `--password` for encrypted archives |
| ZOO | `.zoo` | [unarc-rs](https://crates.io/crates/unarc-rs) | Decode-only |

## Single-Stream Compression Formats

These formats contain exactly one compressed file. `untar` decompresses the input and writes the result to a file with the compression extension removed (e.g., `file.txt.gz` becomes `file.txt`).

| Format | Extension | Crate | Notes |
|--------|-----------|-------|-------|
| Gzip | `.gz` | [flate2](https://crates.io/crates/flate2) | |
| BZip2 | `.bz2` | [bzip2](https://crates.io/crates/bzip2) | |
| XZ | `.xz` | [liblzma](https://crates.io/crates/liblzma) | |
| LZIP | `.lz` | [lzma-rust2](https://crates.io/crates/lzma-rust2) | |
| Zstandard | `.zst` | [ruzstd](https://crates.io/crates/ruzstd) | |
| LZ4 | `.lz4` | [lz4_flex](https://crates.io/crates/lz4_flex) | |
| Brotli | `.br` | [brotli-decompressor](https://crates.io/crates/brotli-decompressor) | No fixed magic; extension or `--format` required |
| LZMA | `.lzma` | [lzma-rs](https://crates.io/crates/lzma-rs) | No fixed magic; extension or `--format` required |
| LZO/lzop | `.lzo` | [lzo](https://crates.io/crates/lzo) | |

## Magic Detection Notes

Most formats are detected by their magic number, so renaming or omitting the extension still works. The following formats have no reliable magic signature and can only be identified by their extension or via `--format`:

- `.br` (Brotli): the Brotli stream header is a variable-size window field, not a fixed magic number.
- `.lzma` (raw LZMA): the 13-byte header contains compression parameters, not a magic number, and the constraints are too weak for robust detection.

## Common Options

All formats support the following shared options:

- `-d, --directory <DIR>`: Output directory
- `-q, --quiet`: Suppress progress output
- `-l, --list`: List archive contents instead of extracting
- `--on-exists <POLICY>`: How to handle existing files (`ask`, `error`, `overwrite`, `skip`, `rename`)
- `--rename-suffix <SUFFIX>`: Suffix for renamed files (default `.1`)
- `--strip-components <N>`: Strip the first N leading path components
- `--password <PASSWORD>`: Password for ZIP and 7-Zip archives
- `--format <FORMAT>`: Force a format instead of auto-detecting
- `--max-total-size <SIZE>`: Maximum total bytes to extract (default `10GB`)
- `--max-entry-size <SIZE>`: Maximum bytes per extracted file (default `1GB`)
- `--max-entry-count <N>`: Maximum number of entries to extract (default `10000`)
- `--max-compression-ratio <N>`: Maximum allowed compression ratio (default `100`)
- `--max-recursion-depth <N>`: Maximum nested-archive depth for `.deb` package payloads (default `3`)
- `--allow-unsafe`: Skip security warnings and continue extraction when limits are exceeded
- `--crack`: Try to crack the archive password with a dictionary attack
- `--wordlist <FILE>`: Use an external wordlist file instead of the built-in SecLists dictionary
- `--extract-hash`: Print a john/hashcat-compatible hash and exit

## Path Safety

All archive paths are normalized and rejected if they contain absolute paths or `..` components, preventing Zip Slip / path traversal attacks.
