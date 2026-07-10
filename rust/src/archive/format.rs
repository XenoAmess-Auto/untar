use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    TarGz,
    TarXz,
    TarBz2,
    TarLzma,
    TarLz,
    TarZst,
    TarLz4,
    TarBr,
    Tar,
    Zip,
    SevenZ,
    Rar,
    Cab,
    Ar,
    Cpio,
    Iso,
    Xar,
    Lha,
    Lzh,
    Deb,
    Squashfs,
    Rpm,
    TarLzo,
    TarZ,
    Z,
    Ace,
    Arc,
    Arj,
    Zoo,
    Gz,
    Bz2,
    Xz,
    Zst,
    Lz4,
    Br,
    Lzma,
    Lzo,
    Lz,
}

impl Format {
    /// Parse a format identifier from the CLI `--format` option.
    pub fn from_cli(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "tar.gz" | "tgz" => Ok(Format::TarGz),
            "tar.xz" | "txz" => Ok(Format::TarXz),
            "tar.bz2" | "tbz2" | "tbz" => Ok(Format::TarBz2),
            "tar.lzma" | "tlz" => Ok(Format::TarLzma),
            "tar.lz" => Ok(Format::TarLz),
            "tar.zst" | "tzst" => Ok(Format::TarZst),
            "tar.lz4" => Ok(Format::TarLz4),
            "tar.br" => Ok(Format::TarBr),
            "tar" => Ok(Format::Tar),
            "zip" | "apk" | "jar" | "war" | "ear" => Ok(Format::Zip),
            "7z" => Ok(Format::SevenZ),
            "rar" => Ok(Format::Rar),
            "cab" => Ok(Format::Cab),
            "ar" | "a" => Ok(Format::Ar),
            "cpio" => Ok(Format::Cpio),
            "iso" => Ok(Format::Iso),
            "xar" => Ok(Format::Xar),
            "lha" => Ok(Format::Lha),
            "lzh" => Ok(Format::Lzh),
            "deb" => Ok(Format::Deb),
            "squashfs" | "sqfs" | "sfs" | "snap" => Ok(Format::Squashfs),
            "rpm" => Ok(Format::Rpm),
            "tar.lzo" => Ok(Format::TarLzo),
            "tar.z" | "taz" => Ok(Format::TarZ),
            "z" => Ok(Format::Z),
            "ace" => Ok(Format::Ace),
            "arc" => Ok(Format::Arc),
            "arj" => Ok(Format::Arj),
            "zoo" => Ok(Format::Zoo),
            "pax" => Ok(Format::Tar),
            "gz" => Ok(Format::Gz),
            "bz2" => Ok(Format::Bz2),
            "xz" => Ok(Format::Xz),
            "zst" => Ok(Format::Zst),
            "lz4" => Ok(Format::Lz4),
            "br" => Ok(Format::Br),
            "lzma" => Ok(Format::Lzma),
            "lzo" => Ok(Format::Lzo),
            "lz" => Ok(Format::Lz),
            _ => Err(anyhow!("Unknown format: {s}")),
        }
    }

    /// Return the file extension(s) that this format is normally associated with.
    #[allow(dead_code)]
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Format::TarGz => &[".tar.gz", ".tgz"],
            Format::TarXz => &[".tar.xz", ".txz"],
            Format::TarBz2 => &[".tar.bz2", ".tbz2", ".tbz"],
            Format::TarLzma => &[".tar.lzma", ".tlz"],
            Format::TarLz => &[".tar.lz", ".tlz"],
            Format::TarZst => &[".tar.zst", ".tzst"],
            Format::TarLz4 => &[".tar.lz4"],
            Format::TarBr => &[".tar.br"],
            Format::Tar => &[".tar"],
            Format::Zip => &[".zip", ".apk", ".jar", ".war", ".ear"],
            Format::SevenZ => &[".7z"],
            Format::Rar => &[".rar"],
            Format::Cab => &[".cab"],
            Format::Ar => &[".ar", ".a"],
            Format::Cpio => &[".cpio"],
            Format::Iso => &[".iso"],
            Format::Xar => &[".xar"],
            Format::Lha => &[".lha"],
            Format::Lzh => &[".lzh"],
            Format::Deb => &[".deb"],
            Format::Squashfs => &[".squashfs", ".sqfs", ".sfs", ".snap"],
            Format::Rpm => &[".rpm"],
            Format::TarLzo => &[".tar.lzo"],
            Format::TarZ => &[".tar.Z", ".taz"],
            Format::Z => &[".Z"],
            Format::Ace => &[".ace"],
            Format::Arc => &[".arc"],
            Format::Arj => &[".arj"],
            Format::Zoo => &[".zoo"],
            Format::Gz => &[".gz"],
            Format::Bz2 => &[".bz2"],
            Format::Xz => &[".xz"],
            Format::Zst => &[".zst"],
            Format::Lz4 => &[".lz4"],
            Format::Br => &[".br"],
            Format::Lzma => &[".lzma"],
            Format::Lzo => &[".lzo"],
            Format::Lz => &[".lz"],
        }
    }
}

fn read_head(file_path: &Path, len: u64) -> Result<Vec<u8>> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path.display()))?;
    let mut buf = Vec::new();
    file.take(len).read_to_end(&mut buf)?;
    Ok(buf)
}

fn starts_with(buf: &[u8], prefix: &[u8]) -> bool {
    buf.len() >= prefix.len() && buf[..prefix.len()] == *prefix
}

fn is_tar(buf: &[u8]) -> bool {
    // tar magic is at offset 257: "ustar\0" followed by version "00" at 263.
    buf.len() >= 264 && buf[257..262] == *b"ustar" && buf[262] == 0 && buf[263..265] == *b"00"
}

fn is_iso(buf: &[u8]) -> bool {
    const ISO_VD_OFFSET: usize = 16 * 2048;
    buf.len() >= ISO_VD_OFFSET + 8
        && buf[ISO_VD_OFFSET + 1..ISO_VD_OFFSET + 6] == *b"CD001"
        && buf[ISO_VD_OFFSET + 6] == 1
}

fn is_lha(buf: &[u8]) -> Option<Format> {
    // LHA header: first byte is header length, then 5-byte method ID.
    if buf.len() < 6 {
        return None;
    }
    let header_len = buf[0] as usize;
    if header_len == 0 || buf.len() < 1 + header_len {
        return None;
    }
    let method = &buf[1..6];
    let lha_methods: [&[u8]; 11] = [
        b"-lh0-", b"-lh1-", b"-lh4-", b"-lh5-", b"-lh6-", b"-lh7-", b"-lhd-", b"-lhx-", b"-lz4-",
        b"-lzs-", b"-lz5-",
    ];
    if lha_methods.contains(&method) {
        Some(Format::Lha)
    } else {
        None
    }
}

fn is_deb(buf: &[u8]) -> bool {
    // .deb is an ar archive whose first member is named "debian-binary".
    if !starts_with(buf, b"!<arch>\n") || buf.len() < 8 + 60 {
        return false;
    }
    let header = &buf[8..8 + 60];
    let name = &header[..16];
    name.starts_with(b"debian-binary")
}

fn is_squashfs(buf: &[u8]) -> bool {
    // SquashFS magic is 0x73717368; little-endian on-disk bytes are "hsqs".
    // Big-endian images spell "sqsh".
    starts_with(buf, b"hsqs") || starts_with(buf, b"sqsh")
}

fn is_rpm(buf: &[u8]) -> bool {
    starts_with(buf, b"\xed\xab\xee\xdb")
}

fn is_lzop(buf: &[u8]) -> bool {
    // lzop file format magic.
    starts_with(buf, b"\x89LZO\x00\r\n\x1a\n")
}

fn is_lzip(buf: &[u8]) -> bool {
    starts_with(buf, b"LZIP")
}

fn is_arj(buf: &[u8]) -> bool {
    buf.len() >= 2 && buf[0] == 0x60 && buf[1] == 0xEA
}

fn is_ace(buf: &[u8]) -> bool {
    buf.len() >= 14 && &buf[7..14] == b"**ACE**"
}

fn is_arc(buf: &[u8]) -> bool {
    buf.len() >= 2 && buf[0] == 0x1A && buf[1] >= 1 && buf[1] <= 11
}

fn is_zoo(buf: &[u8]) -> bool {
    starts_with(buf, b"ZOO ")
}

fn compressed_tar_or_stream(magic_format: Format, ext: &str) -> Format {
    let ext_lower = ext.to_lowercase();
    if ext_lower.contains("tar") {
        match magic_format {
            Format::Gz => Format::TarGz,
            Format::Bz2 => Format::TarBz2,
            Format::Xz => Format::TarXz,
            Format::Zst => Format::TarZst,
            Format::Lz4 => Format::TarLz4,
            Format::Br => Format::TarBr,
            Format::Lzma => Format::TarLzma,
            Format::Lzo => Format::TarLzo,
            Format::Lz => Format::TarLz,
            _ => magic_format,
        }
    } else {
        magic_format
    }
}

/// Detect the archive/compression format of a file by inspecting its contents.
///
/// `ext_hint` is the file extension (including the leading dot), used to
/// disambiguate compressed tar archives from single-stream compressed files,
/// which share the same compression magic number.
pub fn detect_format(file_path: &Path, ext_hint: Option<&str>) -> Result<Format> {
    let buf = read_head(file_path, 16 * 2048 + 8)?;

    if buf.is_empty() {
        return Err(anyhow!("Empty file: {}", file_path.display()));
    }

    let ext = ext_hint.unwrap_or("");

    if starts_with(&buf, b"PK\x03\x04") || starts_with(&buf, b"PK\x05\x06") {
        return Ok(Format::Zip);
    }

    if starts_with(&buf, b"7z\xbc\xaf\x27\x1c") {
        return Ok(Format::SevenZ);
    }

    if starts_with(&buf, b"Rar!") {
        return Ok(Format::Rar);
    }

    if starts_with(&buf, b"MSCF") {
        return Ok(Format::Cab);
    }

    if is_deb(&buf) {
        return Ok(Format::Deb);
    }

    if starts_with(&buf, b"!<arch>\n") {
        return Ok(Format::Ar);
    }

    if starts_with(&buf, b"070701") || starts_with(&buf, b"070702") {
        return Ok(Format::Cpio);
    }

    if starts_with(&buf, b"xar!") {
        return Ok(Format::Xar);
    }

    if let Some(fmt) = is_lha(&buf) {
        return Ok(fmt);
    }

    if is_squashfs(&buf) {
        return Ok(Format::Squashfs);
    }

    if is_rpm(&buf) {
        return Ok(Format::Rpm);
    }

    if is_lzop(&buf) {
        return Ok(compressed_tar_or_stream(Format::Lzo, ext));
    }

    if is_lzip(&buf) {
        return Ok(compressed_tar_or_stream(Format::Lz, ext));
    }

    if is_ace(&buf) {
        return Ok(Format::Ace);
    }

    if is_arc(&buf) {
        return Ok(Format::Arc);
    }

    if is_arj(&buf) {
        return Ok(Format::Arj);
    }

    if is_zoo(&buf) {
        return Ok(Format::Zoo);
    }

    if starts_with(&buf, b"\x1f\x9d") {
        let ext_lower = ext.to_lowercase();
        if ext_lower == ".tar.z" || ext_lower == ".taz" {
            return Ok(Format::TarZ);
        }
        return Ok(Format::Z);
    }

    if is_iso(&buf) {
        return Ok(Format::Iso);
    }

    if is_tar(&buf) {
        return Ok(Format::Tar);
    }

    if starts_with(&buf, b"\x1f\x8b") {
        return Ok(compressed_tar_or_stream(Format::Gz, ext));
    }

    if starts_with(&buf, b"BZh") {
        return Ok(compressed_tar_or_stream(Format::Bz2, ext));
    }

    if starts_with(&buf, b"\xfd7zXZ\x00") {
        return Ok(compressed_tar_or_stream(Format::Xz, ext));
    }

    if buf.len() >= 4 && u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) == 0x28b52ffd {
        return Ok(compressed_tar_or_stream(Format::Zst, ext));
    }

    if buf.len() >= 4 && u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) == 0x184d2204 {
        return Ok(compressed_tar_or_stream(Format::Lz4, ext));
    }

    Err(anyhow!(
        "Could not detect format from contents: {}",
        file_path.display()
    ))
}

/// Resolve the format from a CLI override, contents, or file extension.
pub fn resolve_format(file_path: &Path, cli_format: Option<&str>, ext: &str) -> Result<Format> {
    if let Some(s) = cli_format {
        return Format::from_cli(s);
    }

    match detect_format(file_path, Some(ext)) {
        Ok(fmt) => Ok(fmt),
        Err(_) => {
            format_from_extension(ext).with_context(|| format!("Unsupported archive format: {ext}"))
        }
    }
}

fn format_from_extension(ext: &str) -> Result<Format> {
    match ext.to_lowercase().as_str() {
        ".tar.gz" | ".tgz" => Ok(Format::TarGz),
        ".tar.xz" | ".txz" => Ok(Format::TarXz),
        ".tar.bz2" | ".tbz2" | ".tbz" => Ok(Format::TarBz2),
        ".tar.lzma" | ".tlz" => Ok(Format::TarLzma),
        ".tar.lz" => Ok(Format::TarLz),
        ".tar.zst" | ".tzst" => Ok(Format::TarZst),
        ".tar.lz4" => Ok(Format::TarLz4),
        ".tar.br" => Ok(Format::TarBr),
        ".tar" => Ok(Format::Tar),
        ".zip" => Ok(Format::Zip),
        ".7z" => Ok(Format::SevenZ),
        ".rar" => Ok(Format::Rar),
        ".cab" => Ok(Format::Cab),
        ".ar" | ".a" => Ok(Format::Ar),
        ".cpio" => Ok(Format::Cpio),
        ".iso" => Ok(Format::Iso),
        ".xar" => Ok(Format::Xar),
        ".lha" => Ok(Format::Lha),
        ".lzh" => Ok(Format::Lzh),
        ".deb" => Ok(Format::Deb),
        ".squashfs" | ".sqfs" | ".sfs" | ".snap" => Ok(Format::Squashfs),
        ".rpm" => Ok(Format::Rpm),
        ".tar.lzo" => Ok(Format::TarLzo),
        ".tar.z" | ".taz" => Ok(Format::TarZ),
        ".z" => Ok(Format::Z),
        ".ace" => Ok(Format::Ace),
        ".arc" => Ok(Format::Arc),
        ".arj" => Ok(Format::Arj),
        ".zoo" => Ok(Format::Zoo),
        ".pax" => Ok(Format::Tar),
        ".gz" => Ok(Format::Gz),
        ".bz2" => Ok(Format::Bz2),
        ".xz" => Ok(Format::Xz),
        ".zst" => Ok(Format::Zst),
        ".lz4" => Ok(Format::Lz4),
        ".br" => Ok(Format::Br),
        ".lzma" => Ok(Format::Lzma),
        ".lzo" => Ok(Format::Lzo),
        ".lz" => Ok(Format::Lz),
        _ => Err(anyhow!("Unknown extension: {ext}")),
    }
}
