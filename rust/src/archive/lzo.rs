use std::io::{self, Read};

use anyhow::{anyhow, bail, Context, Result};

const LZOP_MAGIC: &[u8] = b"\x89LZO\x00\r\n\x1a\n";

const F_ADLER32_D: u32 = 0x0000_0001;
const F_ADLER32_C: u32 = 0x0000_0002;
const F_H_EXTRA_FIELD: u32 = 0x0000_0040;
const F_CRC32_D: u32 = 0x0000_0100;
const F_CRC32_C: u32 = 0x0000_0200;
const F_H_FILTER: u32 = 0x0000_0800;

const MAX_BLOCK_SIZE: usize = 64 * 1024 * 1024;

/// Streaming reader for a file compressed with the lzop format.
pub struct LzopReader<R: Read> {
    inner: R,
    flags: u32,
    block: Vec<u8>,
    block_pos: usize,
    block_len: usize,
    eof: bool,
}

impl<R: Read> LzopReader<R> {
    pub fn new(mut inner: R) -> Result<Self> {
        let mut magic = [0u8; LZOP_MAGIC.len()];
        inner
            .read_exact(&mut magic)
            .context("Failed to read lzop magic")?;
        if magic != LZOP_MAGIC {
            bail!("Invalid lzop magic");
        }

        let mut buf2 = [0u8; 2];
        inner.read_exact(&mut buf2).context("version")?;
        let version = u16::from_be_bytes(buf2);
        inner.read_exact(&mut buf2).context("lib_version")?;
        let _lib_version = u16::from_be_bytes(buf2);

        let mut version_needed = 0x0900u16;
        if version >= 0x0940 {
            inner.read_exact(&mut buf2).context("version_needed")?;
            version_needed = u16::from_be_bytes(buf2);
        }
        if version_needed < 0x0900 {
            bail!("Invalid lzop version_needed");
        }

        let mut method = [0u8; 1];
        inner.read_exact(&mut method).context("method")?;
        let method = method[0];
        if !(1..=3).contains(&method) {
            bail!("Unsupported lzop method: {method}");
        }

        let mut _level = 0u8;
        if version >= 0x0940 {
            inner
                .read_exact(std::slice::from_mut(&mut _level))
                .context("level")?;
        }

        let mut buf4 = [0u8; 4];
        inner.read_exact(&mut buf4).context("flags")?;
        let flags = u32::from_be_bytes(buf4);

        if flags & F_H_FILTER != 0 {
            inner.read_exact(&mut buf4).context("filter")?;
        }

        inner.read_exact(&mut buf4).context("mode")?;
        inner.read_exact(&mut buf4).context("mtime_low")?;
        if version >= 0x0940 {
            inner.read_exact(&mut buf4).context("mtime_high")?;
        }

        let mut name_len = [0u8; 1];
        inner.read_exact(&mut name_len).context("name_len")?;
        let name_len = name_len[0] as usize;
        if name_len > 0 {
            let mut name = vec![0u8; name_len];
            inner.read_exact(&mut name).context("name")?;
        }

        // Header checksum (adler32 or crc32 depending on F_H_CRC32).
        inner.read_exact(&mut buf4).context("header_checksum")?;

        if flags & F_H_EXTRA_FIELD != 0 {
            inner.read_exact(&mut buf4).context("extra_field_len")?;
            let extra_len = u32::from_be_bytes(buf4) as usize;
            if extra_len > 0 {
                let mut extra = vec![0u8; extra_len];
                inner.read_exact(&mut extra).context("extra_field")?;
            }
            inner
                .read_exact(&mut buf4)
                .context("extra_field_checksum")?;
        }

        Ok(Self {
            inner,
            flags,
            block: Vec::new(),
            block_pos: 0,
            block_len: 0,
            eof: false,
        })
    }

    fn fill_block(&mut self) -> Result<()> {
        if self.eof {
            self.block_len = 0;
            return Ok(());
        }

        let mut buf4 = [0u8; 4];
        self.inner.read_exact(&mut buf4).context("dst_len")?;
        let dst_len = u32::from_be_bytes(buf4) as usize;

        if dst_len == 0 {
            self.eof = true;
            self.block_len = 0;
            return Ok(());
        }
        if dst_len == 0xffff_ffff {
            bail!("Split lzop files are not supported");
        }
        if dst_len > MAX_BLOCK_SIZE {
            bail!("lzop block size too large");
        }

        self.inner.read_exact(&mut buf4).context("src_len")?;
        let src_len = u32::from_be_bytes(buf4) as usize;
        if src_len == 0 || src_len > dst_len {
            bail!("Invalid lzop compressed block size");
        }

        if self.flags & F_ADLER32_D != 0 {
            self.inner.read_exact(&mut buf4).context("d_adler32")?;
        }
        if self.flags & F_CRC32_D != 0 {
            self.inner.read_exact(&mut buf4).context("d_crc32")?;
        }
        if src_len < dst_len {
            if self.flags & F_ADLER32_C != 0 {
                self.inner.read_exact(&mut buf4).context("c_adler32")?;
            }
            if self.flags & F_CRC32_C != 0 {
                self.inner.read_exact(&mut buf4).context("c_crc32")?;
            }
        }

        let mut compressed = vec![0u8; src_len];
        self.inner
            .read_exact(&mut compressed)
            .context("compressed block")?;

        self.block.resize(dst_len, 0);
        if src_len < dst_len {
            let n = lzo::decompress_into(&compressed, &mut self.block)
                .map_err(|e| anyhow!("LZO decompression error: {e:?}"))?;
            if n != dst_len {
                bail!("LZO decompressed size mismatch");
            }
        } else {
            self.block.copy_from_slice(&compressed);
        }

        self.block_pos = 0;
        self.block_len = dst_len;
        Ok(())
    }
}

impl<R: Read> Read for LzopReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.block_pos >= self.block_len {
            self.fill_block()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        }
        if self.block_len == 0 {
            return Ok(0);
        }
        let available = self.block_len - self.block_pos;
        let n = buf.len().min(available);
        buf[..n].copy_from_slice(&self.block[self.block_pos..self.block_pos + n]);
        self.block_pos += n;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_lzop_magic_only_errors() {
        let data = b"\x89LZO\x00\r\n\x1a\n";
        let r = LzopReader::new(Cursor::new(&data[..]));
        // Header incomplete after magic.
        assert!(r.is_err());
    }
}
