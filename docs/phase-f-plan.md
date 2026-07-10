# Phase F 计划：更多格式与格式识别改进

目标：继续扩展 `untar` 支持的归档/压缩格式，并处理 `.br` / `.lzma` 的格式识别问题。

## 1. ZIP 别名（零新依赖）

将 `.apk` / `.jar` / `.war` / `.ear` 识别为 ZIP 归档的别名，直接复用现有 `zip` 提取逻辑。

- 修改 `rust/src/archive/format.rs`：
  - `Format::from_cli` 中增加 `"apk" | "jar" | "war" | "ear" => Ok(Format::Zip)`。
  - `Format::extensions` 中为 `Zip` 增加 `.apk`、`.jar`、`.war`、`.ear`。
- 修改 `rust/src/extract.rs`：在 `extract_extension` 中增加 `.apk`、`.jar`、`.war`、`.ear`。
- 测试：用现有 `create_zip` 生成 `test.apk` / `test.jar` 并断言解压结果。
- 更新 `docs/supported-formats.md` 和 `README.md`。

## 2. LZIP 支持（`.lz` / `.tar.lz` / `.tlz`）

使用 `lzma-rust2` 0.16（纯 Rust，Apache-2.0）提供 LZIP 读写能力。

- 新增依赖：`lzma-rust2 = "0.16"`（MSRV 1.85，当前环境 1.88 可编译）。
- 修改 `rust/src/archive/tar.rs`：新增 `extract_tar_lz<R>(reader, options)`，用 `LzipReader::new(reader)` 作为 tar 输入流。
- 修改 `rust/src/archive/stream.rs`：新增 `.lz` 单文件解压分支。
- 修改 `rust/src/archive/format.rs`：
  - 新增 `Format::TarLz` 和 `Format::Lz`。
  - 扩展名映射：`.tar.lz`、`.tlz`、`.lz`。
  - Magic 检测：文件头 `b"LZIP"`（偏移 0）。
- 修改 `rust/src/extract.rs`：新增 dispatch 分支。
- 测试：在 `compress_bytes` 增加 `Compression::Lz` 分支，用 `LzipWriter` 生成 fixture；新增 `extracts_tar_lz` / `extracts_lz_stream`。
- 更新文档。

## 3. 复古格式支持（`.tar.Z` / `.Z` / `.ace` / `.arc` / `.zoo`）

使用 `unarc-rs` 0.6 统一支持这些旧格式。注意：该依赖会 transitively 引入 `unrar_sys`，需要 C++ 工具链。

- 新增依赖：`unarc-rs = "0.6"`。
- 新建 `rust/src/archive/unarc.rs`：
  - 使用 `unarc_rs::unified::ArchiveFormat::open_path` / `detect_from_bytes` 打开归档。
  - 遍历 entries，复用 `safe_output_path`、`strip_path_components`、`resolve_conflict`、进度条。
  - `.tar.Z`：将单条目解压到临时 tar 文件，再调用现有 `tar::extract_tar`（复用 strip 等逻辑）。
  - `.Z` / `.ace` / `.arc` / `.zoo`：直接把条目写出到目标路径。
- 修改 `rust/src/archive/format.rs`：
  - 新增 `Format::TarZ` / `Format::Z` / `Format::Ace` / `Format::Arc` / `Format::Zoo`。
  - 扩展名映射：`.tar.Z`、`.taz`、`.Z`、`.ace`、`.arc`、`.zoo`。
  - Magic 检测：可直接使用 `ArchiveFormat::detect_from_bytes`。
- 修改 `rust/src/extract.rs`：新增 dispatch 分支。
- 二进制测试 fixture 策略：
  - `.tar.Z` / `.Z`：安装 `ncompress` 后自己生成，然后提交。
  - `.arc`：安装 `arc` 包后自己生成，然后提交。
  - `.ace` / `.zoo`：从 `unarc-rs` 官方测试集下载已有小型 fixture（MIT/Apache 许可）后提交。
  - 所有 fixture 控制在几十 KB 以内，**不需要 git-lfs**。
- 测试：对每个格式新增 `extracts_*` 测试，验证文件存在且内容正确。
- 更新文档。

## 4. `.br` / `.lzma` 的 Magic 探测

- **不实现。**
- 原因：Brotli 规范没有固定 magic；raw LZMA 的 13 字节头只是参数约束，误判率太高。
- 保持现有行为：依赖扩展名或 `--format` 覆盖。
- 在文档中说明原因。

## 验证与提交

每阶段完成后执行：

```bash
cd rust
cargo fmt --check
cargo clippy --release -- -D warnings
cargo test
```

全部通过后统一提交到仓库。
