# untar

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)

A lightweight, fast Linux command-line tool for extracting tar archives with support for multiple compression formats.

## Features

- 🚀 **Fast & Lightweight** - Written in Rust with optimized release builds
- 📦 **Multiple Formats** - Support for `.tar`, `.tar.gz`, `.tgz`, `.tar.xz`, `.tar.bz2`, `.zip`
- 🔧 **Simple Usage** - Intuitive command-line interface
- 💾 **Preserves Permissions** - Unix file permissions are preserved during extraction
- 📊 **Progress Display** - Verbose mode shows extraction progress with file sizes

## Installation

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
  -v, --verbose          Show progress (enabled by default)
  -h, --help             Show help

Arguments:
  FILE                   Archive file to extract
```

### Examples

```bash
# Extract a tar.gz file
untar myproject.tar.gz

# Extract to a specific directory with verbose output
untar -d ./extracted -v backup.tar.xz

# Extract a ZIP file
untar archive.zip
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
├── rust/           # Main Rust implementation (actively maintained)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── java/           # Java reference implementation
│   ├── pom.xml
│   └── src/
└── .github/        # GitHub Actions workflows
```

## Dependencies

- [tar](https://crates.io/crates/tar) - Tar archive handling
- [flate2](https://crates.io/crates/flate2) - GZip compression support
- [xz2](https://crates.io/crates/xz2) - XZ compression support
- [bzip2](https://crates.io/crates/bzip2) - BZip2 compression support
- [zip](https://crates.io/crates/zip) - ZIP archive support
- [clap](https://crates.io/crates/clap) - Command-line argument parsing
- [anyhow](https://crates.io/crates/anyhow) - Error handling

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Author

- **XenoAmess** - [GitHub](https://github.com/XenoAmess-bot)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request
