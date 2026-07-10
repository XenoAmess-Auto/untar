# untar

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-latest%20stable-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/XenoAmess-Auto/untar/actions/workflows/ci.yml/badge.svg)](https://github.com/XenoAmess-Auto/untar/actions)
[![Release](https://github.com/XenoAmess-Auto/untar/actions/workflows/release.yml/badge.svg)](https://github.com/XenoAmess-Auto/untar/releases)

English | [中文版](README_CN.md)

A lightweight, fast command-line tool for extracting archives with support for a wide range of formats, including password-protected archives.

## Features

- 🚀 **Fast & Lightweight** - Written in Rust with optimized release builds
- 📦 **Multiple Formats** - Support for `.tar`, `.tar.gz`, `.tgz`, `.tar.xz`, `.txz`, `.tar.bz2`, `.tbz2`, `.tbz`, `.tar.lzma`, `.tlz`, `.tar.lz`, `.tar.zst`, `.tzst`, `.tar.lz4`, `.tar.br`, `.zip`, `.apk`, `.jar`, `.war`, `.ear`, `.7z`, `.rar`, `.cab`, `.ar`, `.a`, `.cpio`, `.iso`, `.xar`, `.lha`, `.lzh`, `.deb`, `.squashfs`, `.sqfs`, `.sfs`, `.snap`, `.rpm`, `.tar.Z`, `.taz`, `.Z`, `.ace`, `.arc`, `.zoo`, `.gz`, `.bz2`, `.xz`, `.lz`, `.zst`, `.lz4`, `.br`, `.lzma`
- 🖥️ **Cross-Platform** - Linux (x86_64, ARM64) and Windows (x86_64) support
- 🔧 **Simple Usage** - Intuitive command-line interface
- 💾 **Preserves Permissions** - Unix file permissions are preserved during extraction
- 📊 **Progress Display** - Shows extraction progress with file sizes by default (use `-q` to suppress)

## Installation

### Pre-built Binaries

Pre-built binaries from the [Releases](https://github.com/XenoAmess-Auto/untar/releases) page.

Available builds:
- `untar-x86_64-linux-musl.tar.gz` - Linux x86_64 (static musl)
- `untar-aarch64-linux-musl.tar.gz` - Linux ARM64 (static musl)
- `untar-x86_64-windows.zip` - Windows x86_64

### Linux Packages

Pre-built packages for common distributions are available on the [Releases](https://github.com/XenoAmess-Auto/untar/releases) page.

#### Debian/Ubuntu (.deb)

```bash
sudo apt install ./untar_*.deb
# or
sudo dpkg -i untar_*.deb
```

#### Fedora/RHEL/openSUSE (.rpm)

```bash
sudo rpm -i untar-*.rpm
# or
sudo dnf install ./untar-*.rpm
```

#### Alpine Linux (.apk)

```bash
sudo apk add --allow-untrusted untar-*.apk
```

#### Arch Linux (.pkg.tar.zst)

```bash
sudo pacman -U untar-*.pkg.tar.zst
```

### Docker

A minimal image is available. Build it locally:

```bash
docker build -t untar .
```

Extract an archive with a bind mount:

```bash
docker run --rm -v "$(pwd):/workdir" untar -d /workdir/out /workdir/archive.tar.gz
```

List archive contents:

```bash
docker run --rm -v "$(pwd):/workdir" untar --list /workdir/archive.tar.gz
```

### Windows (.msi / .zip)

Download the `.msi` installer or `.zip` archive from [Releases](https://github.com/XenoAmess-Auto/untar/releases).

To install the MSI silently:

```powershell
msiexec /i untar-*.msi /qn
```

### From Source (Rust)

```bash
# Clone the repository
git clone https://github.com/XenoAmess-Auto/untar.git
cd untar/rust

# Build release version
cargo build --release

# Install to /usr/local/bin (optional)
sudo cp target/release/untar /usr/local/bin/
```

## Usage

### Basic Usage

```bash
# Extract archive to current directory
untar archive.tar.gz

# Extract to specific directory
untar -d /path/to/output archive.tar.gz

# Show help
untar --help
```

### Supported Archive Formats

| Format | Extension | Description |
|--------|-----------|-------------|
| Tar | `.tar` | Uncompressed tar archive |
| Gzip-compressed Tar | `.tar.gz`, `.tgz` | Gzip-compressed tar archive |
| XZ-compressed Tar | `.tar.xz`, `.txz` | XZ-compressed tar archive |
| BZip2-compressed Tar | `.tar.bz2`, `.tbz2`, `.tbz` | BZip2-compressed tar archive |
| LZMA-compressed Tar | `.tar.lzma`, `.tlz` | LZMA-compressed tar archive |
| Zstandard-compressed Tar | `.tar.zst`, `.tzst` | Zstandard-compressed tar archive |
| LZ4-compressed Tar | `.tar.lz4` | LZ4-compressed tar archive |
| Brotli-compressed Tar | `.tar.br` | Brotli-compressed tar archive |
| LZO-compressed Tar | `.tar.lzo` | LZO/lzop-compressed tar archive |
| ZIP | `.zip`, `.apk`, `.jar`, `.war`, `.ear` | ZIP archive (including AES password-protected) |
| 7-Zip | `.7z` | 7z archive (including password-protected) |
| RAR | `.rar` | RAR archive (decode-only) |
| Cabinet | `.cab` | Windows Cabinet archive |
| Unix Archive | `.ar`, `.a` | Unix `ar` archive (common/static libraries) |
| CPIO | `.cpio` | CPIO newc (SVR4) archive |
| ISO 9660 | `.iso` | ISO 9660 CD/DVD image (primary volume names) |
| XAR | `.xar` | XAR archive / macOS installer package payload |
| LHA/LZH | `.lha`, `.lzh` | LHA/LZH archive |
| Debian package | `.deb` | Debian/Ubuntu package (extracts `data.tar.*`) |
| SquashFS | `.squashfs`, `.sqfs`, `.sfs`, `.snap` | SquashFS filesystem image |
| RPM package | `.rpm` | RPM package (Red Hat/Fedora/openSUSE) |
| POSIX pax | `.pax` | POSIX pax archive (handled as tar) |
| LZIP | `.tar.lz`, `.tlz` | LZIP-compressed tar archive |
| Unix compress | `.tar.Z`, `.taz` | Unix compress (LZW) compressed tar archive |
| ACE | `.ace` | ACE archive (decode-only) |
| ARC/PAK | `.arc`, `.pak` | ARC/PAK archive (decode-only) |
| ARJ | `.arj` | ARJ archive (decode-only, password support via `--password`) |
| ZOO | `.zoo` | ZOO archive (decode-only) |
| Gzip stream | `.gz` | Single gzip-compressed file |
| BZip2 stream | `.bz2` | Single bzip2-compressed file |
| XZ stream | `.xz` | Single XZ-compressed file |
| Zstandard stream | `.zst` | Single zstandard-compressed file |
| LZ4 stream | `.lz4` | Single LZ4-compressed file |
| Brotli stream | `.br` | Single brotli-compressed file (extension only) |
| LZMA stream | `.lzma` | Single LZMA-compressed file (extension only) |
| LZO stream | `.lzo` | Single LZO/lzop-compressed file |

`untar` will automatically detect the actual format from the file's magic number if the extension is missing or incorrect. You can also force a format with `--format <FORMAT>`. See `--help` for the full list of accepted format names.

### Command Line Options

```
Usage: untar [OPTIONS] [FILE] [PATTERNS]...

Arguments:
  [FILE]       Archive file to extract or list
  [PATTERNS]   Only extract entries whose path starts with one of these patterns

Options:
  -d, --directory <DIR>        Output directory
  -q, --quiet                 Quiet mode (suppress output)
  -l, --list                  List archive contents instead of extracting
  -p, --password <PASSWORD>   Password for encrypted archives
      --on-exists <POLICY>    How to handle existing files [default: ask]
                               (ask, error, overwrite, skip, rename)
      --rename-suffix <SUFFIX>  Suffix used when renaming existing files [default: .1]
      --strip-components <N>   Strip the first N leading path components
      --format <FORMAT>        Force a format instead of auto-detecting
      --max-total-size <SIZE>   Maximum total bytes to extract [default: 10GB]
      --max-entry-size <SIZE>   Maximum bytes per extracted file [default: 1GB]
      --max-entry-count <N>     Maximum number of entries to extract [default: 10000]
      --max-compression-ratio <N>  Maximum compression ratio allowed [default: 100]
      --max-recursion-depth <N>    Maximum nested-archive depth [default: 3]
      --allow-unsafe             Skip security warnings and continue extraction
      --crack                    Try to crack the archive password with a dictionary attack
      --wordlist <FILE>          External wordlist file (default: built-in SecLists dictionary)
      --extract-hash             Print a john/hashcat-compatible hash and exit
  -h, --help                    Print help
  -V, --version                 Print version
```

### Examples

```bash
# Extract a tar.gz file (with progress output)
untar myproject.tar.gz

# Extract quietly (suppress output)
untar -q myproject.tar.gz

# Extract to a specific directory
untar -d ./extracted backup.tar.xz

# Extract a ZIP file quietly
untar -q archive.zip

# Crack a password-protected archive with the built-in dictionary
untar -d ./extracted --crack secret.zip

# Crack with a custom wordlist
untar -d ./extracted --crack --wordlist passwords.txt secret.7z

# Export a hash for john/hashcat
untar --extract-hash secret.zip
```

## Building

### Requirements

- Latest stable Rust toolchain

### Build Commands

```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# Run tests
cargo test
```

## Project Structure

```
untar/
├── Dockerfile               # Multi-stage Docker image
├── .dockerignore            # Docker build context exclusions
├── rust/                    # Rust implementation
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # CLI entry point
│   │   ├── cli.rs           # Argument parsing
│   │   ├── extract.rs       # Extraction orchestration & path safety
│   │   └── archive/         # Archive format implementations
│   │       ├── mod.rs
│   │       ├── ar.rs
│   │       ├── cab.rs
│   │       ├── cpio.rs
│   │       ├── iso.rs
│   │       ├── lha.rs
│   │       ├── rar.rs
│   │       ├── sevenz.rs
│   │       ├── stream.rs
│   │       ├── tar.rs
│   │       ├── xar.rs
│   │       └── zip.rs
│   └── tests/
│       └── integration.rs   # End-to-end CLI tests
├── .github/
│   ├── workflows/           # CI/CD workflows
│   │   ├── ci.yml           # Build and test
│   │   ├── release.yml      # Release builds (multi-platform)
│   │   └── auto-merge.yml   # Dependabot auto-merge
│   └── dependabot.yml       # Automated dependency updates
├── docs/                    # Additional documentation
│   ├── development-plan.md
│   ├── format-expansion-plan.md
│   └── supported-formats.md
└── LICENSE, README.md       # Documentation
```

## Dependencies

- [tar](https://crates.io/crates/tar) 0.4 - Tar archive handling
- [flate2](https://crates.io/crates/flate2) 1.1 - GZip compression support
- [liblzma](https://crates.io/crates/liblzma) 0.4 - XZ compression support
- [bzip2](https://crates.io/crates/bzip2) 0.6 - BZip2 compression support
- [lzma-rs](https://crates.io/crates/lzma-rs) 0.3 - LZMA decompression
- [lzma-rust2](https://crates.io/crates/lzma-rust2) 0.16 - LZIP decompression
- [ruzstd](https://crates.io/crates/ruzstd) 0.8 - Zstandard decompression
- [lz4_flex](https://crates.io/crates/lz4_flex) 0.13 - LZ4 decompression
- [brotli-decompressor](https://crates.io/crates/brotli-decompressor) 5 - Brotli decompression
- [zip](https://crates.io/crates/zip) 8 - ZIP archive support
- [sevenz-rust2](https://crates.io/crates/sevenz-rust2) 0.20 - 7z archive support
- [rars](https://crates.io/crates/rars) 0.4 - RAR archive support
- [cab](https://crates.io/crates/cab) 0.6 - Windows Cabinet support
- [ar](https://crates.io/crates/ar) 0.9 - Unix archive support
- [cpio](https://crates.io/crates/cpio) 0.4 - CPIO archive support
- [iso9660-rs](https://crates.io/crates/iso9660-rs) 1.0 - ISO 9660 image support
- [xara](https://crates.io/crates/xara) 0.3 - XAR archive support
- [delharc](https://crates.io/crates/delharc) 0.6 - LHA/LZH archive support
- [backhand](https://crates.io/crates/backhand) 0.25 - SquashFS image support
- [rpm](https://crates.io/crates/rpm) 0.25 - RPM package support
- [unarc-rs](https://crates.io/crates/unarc-rs) 0.6 - Legacy formats (ACE, ARC, ZOO, Unix compress)
- [lzo](https://crates.io/crates/lzo) 0.1 - LZO/lzop decompression
- [clap](https://crates.io/crates/clap) 4.6 - Command-line argument parsing
- [anyhow](https://crates.io/crates/anyhow) 1.0 - Error handling
- [indicatif](https://crates.io/crates/indicatif) 0.17 - Progress display

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Author

- **XenoAmess** - [GitHub](https://github.com/XenoAmess-Auto)

## Contributing

Contributions are welcome! This project uses:
- **Dependabot** for automated dependency updates
- **Rebase merging** for clean linear history (Squash and Merge commit are disabled)

Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request
