# Multi-Archive Extraction Plan

## Goal

Support extracting multiple archive files in a single `untar` invocation, and provide an `--auto-dir` option that extracts each archive into its own stem-named subdirectory.

## CLI Changes

```text
untar [OPTIONS] [FILES]...

Arguments:
  [FILES]    Archive file(s) to extract or list

Options:
  -d, --directory <DIR>        Output directory
      --auto-dir              Extract each archive into a subdirectory named after its stem
      --pattern <PATTERN>     Pattern for filtering entries inside archives (repeatable)
  ... (existing options remain unchanged)
```

- `FILES` changes from a single `Option<String>` to a positional vector (`num_args = 1..`).
- `PATTERNS` is removed from positional arguments; use `--pattern` instead.
- `--auto-dir` is added as a boolean flag.

## Behavior Rules

1. **Output directory**: `-d <DIR>` still defines the base output directory (default current directory).
2. **Auto directory**: When `--auto-dir` is set, each archive is extracted into `<base_dir>/<archive_stem>/`. The stem is derived by stripping the longest recognized archive extension from the file name:
   - `a.zip` -> `a`
   - `a.tar.gz` -> `a`
   - `a.tgz` -> `a`
   - `a.apk` -> `a`
   - Files without recognized extensions use the full file name.
3. **Single archive**: `--auto-dir` also works for a single archive (`untar a.zip --auto-dir` extracts to `a/`).
4. **Patterns**: `--pattern` applies to all archives in the command. It replaces the previous positional `PATTERNS`.
5. **Global options**: `--format` and `--password` apply to all archives in the command (assumes same type/password).
6. **Hash / crack**: `--extract-hash` and `--crack` are applied per file sequentially.
7. **Limits**: Security limits (`--max-total-size`, `--max-entry-size`, `--max-entry-count`, `--max-compression-ratio`) are computed **per archive**, not globally. A fresh `LimitTracker` is created for each file.
8. **Error handling**: If one archive fails, print the error and continue with the remaining archives. Exit with a non-zero status if any archive failed.

## Implementation Steps

1. **`rust/src/cli.rs`**
   - Change `file: Option<String>` to `files: Vec<String>` with `num_args = 1..`.
   - Change `patterns` from positional `Vec<String>` to `#[arg(long = "pattern", value_name = "PATTERN", num_args = 1..)]`.
   - Add `#[arg(long)] auto_dir: bool`.
   - Update doc comments for `files`, `patterns`, and `auto_dir`.
   - Remove or update `Args::archive_file()` method.

2. **`rust/src/extract.rs`**
   - Add a public helper `archive_stem(path: &Path) -> String` that uses the existing `extract_extension` to strip recognized archive extensions and return the stem.

3. **`rust/src/main.rs`**
   - Error if `files` is empty.
   - Loop over `args.files`.
   - For each file:
     - Compute the final `output_dir` based on `--auto-dir`.
     - Create a fresh `LimitTracker`.
     - Build `ExtractOptions` with the per-archive output directory and limits.
     - If `--extract-hash`: resolve format and call `cracker::extract_hash`.
     - Else if `--crack`: call `cracker::crack_archive`, then extract if a password is found.
     - Else: call `extract::extract_archive`.
   - Collect errors and exit non-zero if any occurred.

4. **Tests (`rust/tests/integration.rs`)**
   - Add tests for:
     - Multiple archives extracted to the same directory.
     - Multiple archives with `--auto-dir`.
     - Single archive with `--auto-dir`.
     - `--pattern` filtering with multiple archives.
     - Per-archive limits (e.g., `--max-entry-count=1` should not affect the second archive).

5. **Documentation**
   - Update `README.md` and `README_CN.md`:
     - Usage line: `untar [OPTIONS] [FILES]...`
     - Add `--auto-dir` and `--pattern` to the options table.
     - Add examples for multiple files and `--auto-dir`.

6. **Validation**
   - Run `cargo fmt --check`.
   - Run `cargo clippy --release -- -D warnings`.
   - Run `cargo test`.

7. **Commit and push**.

## Edge Cases

- If two archives have the same stem with `--auto-dir`, they will extract into the same subdirectory (files may merge or conflict). Use `--on-exists` for file-level conflicts.
- If `--auto-dir` is used with `--list`, the prefix directory is ignored for listing (only meaningful for extraction).
- `--format` is global; it is not possible to mix different forced formats in one invocation.
- `--pattern` is global; the same pattern is applied to every archive.
