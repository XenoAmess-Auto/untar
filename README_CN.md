# untar

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-latest%20stable-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/XenoAmess-Auto/untar/actions/workflows/ci.yml/badge.svg)](https://github.com/XenoAmess-Auto/untar/actions)
[![Release](https://github.com/XenoAmess-Auto/untar/actions/workflows/release.yml/badge.svg)](https://github.com/XenoAmess-Auto/untar/releases)

[English](README.md) | 中文版

一个轻量级、快速的命令行工具，用于解压 tar 归档文件，支持多种压缩格式。

## 功能特性

- 🚀 **快速轻量** - 使用 Rust 编写，经过优化的 Release 构建
- 📦 **多种格式** - 支持 `.tar`、`.tar.gz`、`.tgz`、`.tar.xz`、`.tar.bz2`、`.zip`
- 🖥️ **跨平台** - 支持 Linux (x86_64、ARM64) 和 Windows (x86_64)
- 🔧 **简单易用** - 直观的命令行界面
- 💾 **保留权限** - 在解压过程中保留 Unix 文件权限
- 📊 **进度显示** - 默认显示解压进度和文件大小（使用 `-q` 静默模式可关闭）

## 安装

### 预构建二进制文件

从 [Releases](https://github.com/XenoAmess-Auto/untar/releases) 页面下载预构建的二进制文件。

可用构建版本：
- `untar-x86_64-linux-musl.tar.gz` - Linux x86_64 (静态 musl)
- `untar-aarch64-linux-musl.tar.gz` - Linux ARM64 (静态 musl)
- `untar-x86_64-windows.zip` - Windows x86_64

### Linux 安装包

常见发行版的预构建安装包可在 [Releases](https://github.com/XenoAmess-Auto/untar/releases) 页面下载。

#### Debian/Ubuntu (.deb)

```bash
sudo apt install ./untar_*.deb
# 或
sudo dpkg -i untar_*.deb
```

#### Fedora/RHEL/openSUSE (.rpm)

```bash
sudo rpm -i untar-*.rpm
# 或
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

### Windows (.msi / .zip)

从 [Releases](https://github.com/XenoAmess-Auto/untar/releases) 页面下载 `.msi` 安装程序或 `.zip` 压缩包。

静默安装 MSI：

```powershell
msiexec /i untar-*.msi /qn
```

### 从源码编译 (Rust)

```bash
# 克隆仓库
git clone https://github.com/XenoAmess-Auto/untar.git
cd untar/rust

# 构建 Release 版本
cargo build --release

# 安装到 /usr/local/bin（可选）
sudo cp target/release/untar /usr/local/bin/
```

## 使用方法

### 基本用法

```bash
# 解压归档到当前目录
untar archive.tar.gz

# 解压到指定目录
untar -d /path/to/output archive.tar.gz

# 显示帮助
untar --help
```

### 支持的归档格式

| 格式 | 扩展名 | 描述 |
|------|--------|------|
| Tar | `.tar` | 未压缩的 tar 归档 |
| Gzip | `.tar.gz`、`.tgz` | Gzip 压缩的 tar 归档 |
| XZ | `.tar.xz` | XZ 压缩的 tar 归档 |
| BZip2 | `.tar.bz2` | BZip2 压缩的 tar 归档 |
| Zip | `.zip` | ZIP 归档 |

### 命令行选项

```
Usage: untar [OPTIONS] [FILE]

Arguments:
  [FILE]  要解压的归档文件

Options:
  -d, --directory <DIR>  解压文件到指定目录（默认：当前目录）
  -q, --quiet            静默模式（不显示进度）
  -h, --help             显示帮助
  -V, --version          显示版本
```

### 示例

```bash
# 解压 tar.gz 文件（显示进度）
untar myproject.tar.gz

# 静默解压（不显示输出）
untar -q myproject.tar.gz

# 解压到指定目录
untar -d ./extracted backup.tar.xz

# 静默解压 ZIP 文件
untar -q archive.zip
```

## 构建

### 环境要求

- 最新 stable Rust 工具链

### 构建命令

```bash
# 开发构建
cargo build

# 优化 Release 构建
cargo build --release

# 运行测试
cargo test
```

## 项目结构

```
untar/
├── rust/                    # Rust 实现
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # CLI 入口
│   │   ├── cli.rs           # 参数解析
│   │   ├── extract.rs       # 解压编排与路径安全
│   │   └── archive/         # 归档格式实现
│   │       ├── mod.rs
│   │       ├── tar.rs
│   │       └── zip.rs
│   └── tests/
│       └── integration.rs   # 端到端 CLI 测试
├── .github/
│   ├── workflows/           # CI/CD 工作流
│   │   ├── ci.yml           # 构建和测试
│   │   ├── release.yml      # 多平台 Release 构建
│   │   └── auto-merge.yml   # Dependabot 自动合并
│   └── dependabot.yml       # 自动依赖更新
└── LICENSE、README.md       # 文档
```

## 依赖

- [tar](https://crates.io/crates/tar) 0.4 - Tar 归档处理
- [flate2](https://crates.io/crates/flate2) 1.1 - GZip 压缩支持
- [xz2](https://crates.io/crates/xz2) 0.1 - XZ 压缩支持
- [bzip2](https://crates.io/crates/bzip2) 0.6 - BZip2 压缩支持
- [zip](https://crates.io/crates/zip) 7 - ZIP 归档支持
- [clap](https://crates.io/crates/clap) 4.5 - 命令行参数解析
- [anyhow](https://crates.io/crates/anyhow) 1.0 - 错误处理

## 许可证

本项目采用 Apache License 2.0 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 作者

- **XenoAmess** - [GitHub](https://github.com/XenoAmess-Auto)

## 贡献

欢迎贡献！本项目使用：
- **Dependabot** 进行自动依赖更新
- **Rebase 合并** 保持简洁的线性历史（Squash 和 Merge commit 已禁用）

欢迎提交 Pull Request。

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request
