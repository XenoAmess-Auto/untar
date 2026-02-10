# untar

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/XenoAmess-bot/untar/actions/workflows/ci.yml/badge.svg)](https://github.com/XenoAmess-bot/untar/actions)
[![Release](https://github.com/XenoAmess-bot/untar/actions/workflows/release.yml/badge.svg)](https://github.com/XenoAmess-bot/untar/releases)

English | [中文版](README_CN.md)

A lightweight, fast command-line tool for extracting tar archives with support for multiple compression formats.

## Features

- 🚀 **Fast & Lightweight** - Written in Rust with optimized release builds
- 📦 **Multiple Formats** - Support for `.tar`, `.tar.gz`, `.tgz`, `.tar.xz`, `.tar.bz2`, `.zip`
- 🖥️ **Cross-Platform** - Linux (x86_64, ARM64) and Windows (x86_64) support
- 🔧 **Simple Usage** - Intuitive command-line interface
- 💾 **Preserves Permissions** - Unix file permissions are preserved during extraction
- 📊 **Progress Display** - Shows extraction progress with file sizes by default (use `-q` to suppress)

## Installation

### Pre-built Binaries

Download pre-built binaries from the [Releases](https://github.com/XenoAmess-bot/untar/releases) page.

Available builds:
- `untar-x86_64-linux-gnu` - Linux x86_64 (glibc)
- `untar-x86_64-linux-musl` - Linux x86_64 (static musl)
- `untar-aarch64-linux-musl` - Linux ARM64 (static musl)
- `untar-x86_64-windows` - Windows x86_64

### Debian/Ubuntu (.deb package)

Download the `.deb` package from [Releases](https://github.com/XenoAmess-bot/untar/releases) and install:

```bash
sudo dpkg -i untar_*.deb
```

Or install dependencies automatically:
```bash
sudo apt install ./untar_*.deb
```

### From Source (Rust)

```bash
# Clone the repository
git clone https://github.com/XenoAmess-bot/untar.git
cd untar/rust

# Build release version
cargo build --release

# Install to /usr/local/bin (optional)
sudo cp target/release/untar /usr/local/bin/
```

### Java Version (Reference Only)

The Java version is preserved for reference and uses Quarkus + GraalVM for native compilation. It is no longer actively maintained.

```bash
cd untar/java
mvn package -Pnative
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
| Gzip | `.tar.gz`, `.tgz` | Gzip-compressed tar archive |
| XZ | `.tar.xz` | XZ-compressed tar archive |
| BZip2 | `.tar.bz2` | BZip2-compressed tar archive |
| Zip | `.zip` | ZIP archive |

### Command Line Options

```
Usage: untar [OPTIONS] FILE

Options:
  -d, --directory DIR    Extract files into DIR (default: current directory)
  -q, --quiet            Suppress output (no progress display)
  -h, --help             Show help

Arguments:
  FILE                   Archive file to extract
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
```

## Building

### Requirements

- Rust 1.70+ (for Rust version)
- Java 17 + Maven (for Java version, reference only)

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
├── rust/                    # Main Rust implementation (actively maintained)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── java/                    # Java reference implementation
│   ├── pom.xml
│   └── src/
├── .github/
│   ├── workflows/           # CI/CD workflows
│   │   ├── ci.yml           # Build and test
│   │   ├── release.yml      # Release builds (multi-platform)
│   │   └── build-deb.yml    # Debian package builder
│   └── dependabot.yml       # Automated dependency updates
└── LICENSE, README.md       # Documentation
```

## Dependencies

- [tar](https://crates.io/crates/tar) 0.4 - Tar archive handling
- [flate2](https://crates.io/crates/flate2) 1.1 - GZip compression support
- [xz2](https://crates.io/crates/xz2) 0.1 - XZ compression support
- [bzip2](https://crates.io/crates/bzip2) 0.6 - BZip2 compression support
- [zip](https://crates.io/crates/zip) 7 - ZIP archive support
- [clap](https://crates.io/crates/clap) 4.5 - Command-line argument parsing
- [anyhow](https://crates.io/crates/anyhow) 1.0 - Error handling

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Author

- **XenoAmess** - [GitHub](https://github.com/XenoAmess-bot)

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
