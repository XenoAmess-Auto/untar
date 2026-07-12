# untar

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-latest%20stable-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/XenoAmess-Auto/untar/actions/workflows/ci.yml/badge.svg)](https://github.com/XenoAmess-Auto/untar/actions)
[![Release](https://github.com/XenoAmess-Auto/untar/actions/workflows/release.yml/badge.svg)](https://github.com/XenoAmess-Auto/untar/releases)

English | [中文版](README_CN.md)

A lightweight, fast command-line tool for extracting archives with support for a wide range of formats, including password-protected archives.

## Features

- 🚀 **Fast & Lightweight** - Written in Rust with optimized release builds
- 📦 **Multiple Formats** - Support for `.tar`, `.tar.gz`, `.tgz`, `.tar.xz`, `.txz`, `.tar.bz2`, `.tbz2`, `.tbz`, `.tar.lzma`, `.tlz`, `.tar.lz`, `.tar.zst`, `.tzst`, `.tar.lz4`, `.tar.br`, `.tar.lzo`, `.zip`, `.apk`, `.jar`, `.war`, `.ear`, `.7z`, `.rar`, `.cab`, `.ar`, `.a`, `.cpio`, `.iso`, `.xar`, `.lha`, `.lzh`, `.deb`, `.squashfs`, `.sqfs`, `.sfs`, `.snap`, `.rpm`, `.tar.Z`, `.taz`, `.Z`, `.ace`, `.arc`, `.pak`, `.zoo`, `.gz`, `.bz2`, `.xz`, `.lz`, `.zst`, `.lz4`, `.br`, `.lzma`, `.lzo`, `.pax`
- 🖥️ **Cross-Platform** - Linux (x86_64, ARM64) and Windows (x86_64) support
- 🔧 **Simple Usage** - Intuitive command-line interface
- 💾 **Preserves Permissions** - Unix file permissions are preserved during extraction where the archive format stores them (e.g., tar, ar, cpio, rpm, squashfs, zip)
- 📊 **Progress Display** - Shows extraction progress with file sizes by default (use `-q` to suppress)

## Installation

### Pre-built Binaries

Pre-built binaries from the [Releases](https://github.com/XenoAmess-Auto/untar/releases) page.

Available builds:
- `untar-x86_64-unknown-linux-musl.tar.gz` - Linux x86_64 (static musl)
- `untar-aarch64-unknown-linux-musl.tar.gz` - Linux ARM64 (static musl)
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
# Extract archive(s) to current directory
untar archive.tar.gz
untar archive1.tar.gz archive2.zip

# Extract to specific directory
untar -d /path/to/output archive.tar.gz

# Extract each archive into its own stem-named folder
untar -d /path/to/output --auto-dir archive1.tar.gz archive2.zip
# creates /path/to/output/archive1/ and /path/to/output/archive2/

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
| LZMA-compressed Tar | `.tar.lzma`, `.tlz` | LZMA-compressed tar archive (`.tlz` with non-LZIP header) |
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
| LZIP | `.tar.lz`, `.tlz` | LZIP-compressed tar archive (`.tlz` with LZIP header) |
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
Usage: untar [OPTIONS] [FILES]...

Arguments:
  [FILES]      Archive file(s) to extract or list

Options:
  -d, --directory <DIR>        Output directory
      --auto-dir              Extract each archive into its own stem-named subdirectory
      --pattern <PATTERN>     Only extract entries whose path matches one of these patterns
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
      --max-recursion-depth <N>    Maximum nested-archive depth for .deb package payloads [default: 3]
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

# Extract multiple archives to the same directory
untar myproject.tar.gz assets.zip -d ./extracted

# Extract each archive into its own folder
untar myproject.tar.gz assets.zip --auto-dir -d ./extracted

# Extract quietly (suppress output)
untar -q myproject.tar.gz

# Extract a ZIP file quietly
untar -q archive.zip

# Extract only matching entries from an archive
untar -d ./extracted --pattern 'docs/' archive.tar.gz

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
│   │       ├── deb.rs
│   │       ├── iso.rs
│   │       ├── lha.rs
│   │       ├── lzo.rs
│   │       ├── rar.rs
│   │       ├── rpm.rs
│   │       ├── sevenz.rs
│   │       ├── squashfs.rs
│   │       ├── stream.rs
│   │       ├── tar.rs
│   │       ├── unarc.rs
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
