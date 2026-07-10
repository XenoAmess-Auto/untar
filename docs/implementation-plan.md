# Implementation Plan

## Overview

This plan covers four work items requested after the v1.1.0 release:

1. Update CLI help and project documentation to reflect all currently supported formats.
2. Add security guards against archive bombs, decompression bombs, and directory traversal/symlink attacks.
3. Complete password-protected archive support, including ARJ.
4. Add a built-in password cracker with dictionary attacks and `john`/`hashcat` integration.

## Goal

Make `untar` safer, better documented, and able to handle encrypted archives end-to-end (including a user-supplied fallback cracker), while keeping the CLI simple and cross-platform.

---

## Phase 1: Documentation and Help Updates

### Why
The current `--help` text and README still describe the project as a tar-only extractor, even though dozens of formats are now supported. This causes confusion and makes the tool harder to discover.

### Files to update

| File | Change |
|------|--------|
| `rust/src/cli.rs` | Rewrite the `about` doc comment to describe the broad format support. Add accepted values to the `--format` help. |
| `rust/Cargo.toml` | Update the `description` field. |
| `README.md` | Correct the feature list, remove the outdated "not supported" note for `.tar.lz`/`.tar.Z`/`.ace`/`.arc`/`.zoo`, update the module structure, and complete the dependency list. |
| `README_CN.md` | Add `--format` and `--password` to the options block. |
| `CLAUDE.md` | Refresh the project overview. |
| `nfpm.yaml` | Update the package description. |
| `docs/format-expansion-plan.md` | Update the exclusions section to reflect current support. |

### Tests to add

- `.tar` (plain tar)
- `.tar.xz` / `.txz`
- `.tar.bz2` / `.tbz2` / `.tbz`
- `.rar`
- `.lha` / `.lzh`
- `.arj` (added in Phase 3)

---

## Phase 2: Security Hardening

### Why
Currently there is uniform directory-traversal protection via `safe_output_path`, but no limits on output size, entry count, compression ratio, recursion depth, or symlink targets. A malicious archive can fill the disk or recurse indefinitely.

### Design decisions

- **Warn first, abort by default**: When a limit is exceeded, print a clear warning and ask the user whether to continue. This keeps the tool usable for legitimate edge cases (e.g., a very large log archive) without silently exposing the machine to attacks.
- **Non-TTY fallback**: If stdin is not a terminal, the tool cannot ask interactively. In that case it refuses to continue unless the user explicitly passed `--allow-unsafe`.
- **Configurable limits**: All limits are exposed as CLI options with sensible defaults.

### New CLI options

| Flag | Default | Purpose |
|------|---------|---------|
| `--max-total-size` | `10GB` | Maximum total bytes written for the entire extraction. |
| `--max-entry-size` | `1GB` | Maximum bytes for a single output file. |
| `--max-entry-count` | `10000` | Maximum number of entries extracted. |
| `--max-compression-ratio` | `100` | Maximum allowed uncompressed-to-compressed size ratio. |
| `--max-recursion-depth` | `3` | Maximum nested-archive depth (e.g., `.deb` inner tar). |
| `--allow-unsafe` | `false` | Skip warnings and continue extraction even when limits are exceeded. |

### Implementation tasks

1. Add `ExtractOptions` fields: `max_total_size`, `max_entry_size`, `max_entry_count`, `max_compression_ratio`, `max_recursion_depth`, `allow_unsafe`.
2. Add a `LimitedWriter` that wraps `io::copy` and tracks per-entry and total bytes written.
3. Add a warning/prompt helper that prints the limit being violated and asks for confirmation in TTY mode.
4. Add ratio pre-checks for ZIP, 7z, and RAR where compressed and uncompressed sizes are known.
5. Add recursion-depth tracking for nested archives (currently `.deb`; future nested tar support).
6. Harden symlink handling:
   - Intercept tar `Symlink` and `Hardlink` entries; reject hardlinks and validate symlink targets.
   - Validate symlink targets in `squashfs.rs` and `rpm.rs` before creating them.
7. Add tests:
   - Zip-slip variants (`../`, absolute paths, backslash on Windows).
   - Decompression bomb (tiny compressed size, huge uncompressed size).
   - Entry exceeding `--max-entry-size`.
   - Total output exceeding `--max-total-size`.
   - Malicious symlink target.

---

## Phase 3: Password Support and ARJ

**Status: completed**

### Why
Password-protected ZIP, 7z, and RAR are already supported, but the CLI help/docs do not advertise it well. ARJ is supported by the `unarc-rs` crate but is not exposed in the current format dispatcher.

### What was done
- Added `Format::Arj` with `.arj` extension and magic detection (`0x60 0xEA`).
- Added `extract_arj` in `rust/src/archive/unarc.rs` and wired it into `extract.rs`.
- Passed `ArchiveOptions::with_password(...)` through the `unarc-rs` wrapper so encrypted ARJ works with `--password`.
- Added plain and password-protected ARJ integration tests using the `unarc-rs` test fixtures.
- Verified ZIP AES/ZipCrypto, 7z AES-256, and ARJ password extraction with existing tests. RAR password extraction is implemented via `rars`; no test fixture is available in the repo, so it is covered by code path only.

---

## Phase 4: Password Cracker

### Why
Users sometimes receive a password-protected archive but do not know the password. A built-in recovery mode provides a fallback without forcing the user to install external tools first.

### Scope

- **Pure Rust**: dictionary attacks for ZIP, 7z, and RAR using the crates already in `Cargo.toml`.
- **External tools**: export hashes for `john`/`hashcat` when the encryption is strong (ZIP AES, 7z, RAR5).
- **Weak formats only**: ZIP ZipCrypto is fast to crack in pure Rust. Strong AES formats require external GPU tools.
- **ACE/ARC/ZOO encrypted**: out of scope because no Rust libraries reliably implement their encryption.

### Built-in wordlist

- **Source**: `SecLists/Passwords/Common-Credentials/xato-net-10-million-passwords.txt` (5,189,454 unique passwords, 46.3 MB raw).
- **Storage**: compress with zstd to ~10-15 MB as `rust/assets/default_wordlist.txt.zst` and embed with `include_bytes!`. This keeps the binary and Git repository manageable.
- **Runtime**: decompress line-by-line using `ruzstd` (already a dependency) to avoid loading the full 46 MB into memory at once.
- **Override**: `--wordlist <FILE>` always takes precedence over the built-in list.

### New CLI options

| Flag | Purpose |
|------|---------|
| `--crack` | Immediately try to crack a password-protected archive. |
| `--wordlist <FILE>` | Use an external dictionary instead of the built-in one. |
| `--extract-hash` | Print a `john`/`hashcat`-compatible hash and exit, without attempting to crack. |

### Behavior

1. If extraction fails due to a wrong password and stdin is a TTY, prompt: `Password incorrect. Try to crack? [y/N]`.
2. If `--crack` is given, skip the prompt and start cracking immediately.
3. In non-TTY mode, cracking is skipped unless `--crack` is explicitly provided.
4. For each candidate password:
   - ZIP: use `zip::ZipArchive::by_index_decrypt()`.
   - 7z: use `sevenz_rust2::ArchiveReader` with `Password`.
   - RAR: use `rars::Archive::extract_to(password, ...)`.
5. For ZIP ZipCrypto, verify the candidate by decompressing the first file and checking CRC to avoid the 1/256 false-positive issue.
6. When a candidate succeeds, continue extraction with that password.
7. If no candidate succeeds, print a message suggesting `--extract-hash` plus `john`/`hashcat`.

### External-tool integration

1. Detect whether `john` or `hashcat` is installed at runtime (`Command::new("john").arg("--version")`, etc.).
2. `--extract-hash` behavior by format:
   - ZIP: run `zip2john <archive>` and print the resulting hash line.
   - RAR: run `rar2john <archive>` and print the hash line.
   - 7z: run `7z2john.pl <archive>` and print the hash line.
3. If the external tool is not installed, print a clear error and instructions.
4. Add a `crack` subcommand or flag that shells out to `john`/`hashcat` directly with the built-in or user-supplied wordlist for strong formats.

### Tests to add

- Crack a known-password ZIP (ZipCrypto) with the built-in wordlist.
- Crack a known-password ZIP (AES) with a small custom wordlist.
- Export hash for a ZIP/7z/RAR and verify the output format.
- Non-TTY behavior: `--crack` works, no prompt.

---

## Risks and Notes

1. **Binary size**: The built-in compressed wordlist will add ~10-15 MB to the binary and release artifacts. This is acceptable for the requested feature but should be documented.
2. **Performance**: Strong encrypted formats (7z, RAR5, ZIP AES) will be slow to dictionary-attack in pure Rust. The cracker should always point users to `john`/`hashcat` for these cases.
3. **Legal/ethical**: The cracker is intended for users to recover their own archives. The built-in wordlist is from a public, MIT-licensed source (SecLists). The project documentation should state that the tool must only be used on files the user owns or has permission to access.
4. **ARJ encryption**: `unarc-rs` exposes `ArjEncryption` variants (`Garble`, `Gost40`, `Gost256`, `Unknown`). The implementation will try the supplied password and report failure if the encryption variant is unsupported or the password is wrong.
5. **CI**: After each phase, run `cargo fmt --check`, `cargo clippy --release -- -D warnings`, and `cargo test` in `rust/` before committing and pushing.

---

## Execution Order

1. Phase 1: Documentation and help updates.
2. Phase 2: Security hardening.
3. Phase 3: Password support + ARJ.
4. Phase 4: Password cracker.

Each phase ends with lint, tests, commit, and push.
