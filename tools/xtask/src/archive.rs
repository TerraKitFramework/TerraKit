//! Write and inspect ZIP archives using the ZIP32 stored-entry format.
//!
//! Release packages contain native libraries, a header, and metadata. Stored
//! entries avoid external compression tools; they are valid but uncompressed ZIPs.

use std::collections::BTreeMap;
use std::io::{Error, ErrorKind, Result};

use crate::digest::crc32;

fn invalid(message: &str) -> Error {
    Error::new(ErrorKind::InvalidData, message.to_owned())
}

fn number(value: usize, label: &str) -> Result<u32> {
    u32::try_from(value).map_err(|_| invalid(label))
}

fn u16_number(value: usize, label: &str) -> Result<u16> {
    u16::try_from(value).map_err(|_| invalid(label))
}

fn le16(out: &mut Vec<u8>, n: u16) {
    out.extend_from_slice(&n.to_le_bytes());
}

fn le32(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}

fn read_u16(data: &[u8], at: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        data.get(at..at + 2)
            .ok_or_else(|| invalid("truncated ZIP"))?
            .try_into()
            .expect("two bytes"),
    ))
}

fn read_u32(data: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        data.get(at..at + 4)
            .ok_or_else(|| invalid("truncated ZIP"))?
            .try_into()
            .expect("four bytes"),
    ))
}

/// Creates a ZIP with CRC-checked stored entries; paths are UTF-8 and relative.
pub fn write(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, bytes) in files {
        if name.starts_with('/') || name.contains("..") || name.contains('\\') {
            return Err(invalid("invalid ZIP entry path"));
        }
        let name_bytes = name.as_bytes();
        let path_len = u16_number(name_bytes.len(), "ZIP path is too long")?;
        let size = number(bytes.len(), "ZIP entry exceeds ZIP32 size")?;
        let offset = number(out.len(), "ZIP exceeds ZIP32 size")?;
        let crc = crc32(bytes);

        le32(&mut out, 0x0403_4b50);
        le16(&mut out, 20); // ZIP version required
        le16(&mut out, 0x0800); // UTF-8 names
        le16(&mut out, 0); // Stored (no compression)
        le16(&mut out, 0);
        le16(&mut out, 0);
        le32(&mut out, crc);
        le32(&mut out, size);
        le32(&mut out, size);
        le16(&mut out, path_len);
        le16(&mut out, 0);
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(bytes);

        le32(&mut central, 0x0201_4b50);
        le16(&mut central, 20); // Creator version
        le16(&mut central, 20); // Required version
        le16(&mut central, 0x0800);
        le16(&mut central, 0);
        le16(&mut central, 0);
        le16(&mut central, 0);
        le32(&mut central, crc);
        le32(&mut central, size);
        le32(&mut central, size);
        le16(&mut central, path_len);
        le16(&mut central, 0);
        le16(&mut central, 0);
        le16(&mut central, 0);
        le16(&mut central, 0);
        le32(&mut central, 0);
        le32(&mut central, offset);
        central.extend_from_slice(name_bytes);
    }
    let central_offset = number(out.len(), "ZIP exceeds ZIP32 size")?;
    let central_size = number(central.len(), "ZIP central directory exceeds ZIP32 size")?;
    let count = u16_number(files.len(), "too many ZIP entries")?;
    out.extend_from_slice(&central);
    le32(&mut out, 0x0605_4b50);
    le16(&mut out, 0);
    le16(&mut out, 0);
    le16(&mut out, count);
    le16(&mut out, count);
    le32(&mut out, central_size);
    le32(&mut out, central_offset);
    le16(&mut out, 0);
    Ok(out)
}

/// Reads stored entries and rejects corrupt data, duplicates, and unsupported formats.
pub fn read(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut out = BTreeMap::new();
    let mut at = 0_usize;
    while read_u32(bytes, at)? == 0x0403_4b50 {
        let flags = read_u16(bytes, at + 6)?;
        if flags != 0x0800 || read_u16(bytes, at + 8)? != 0 {
            return Err(invalid("unsupported ZIP flags or compression"));
        }
        let crc = read_u32(bytes, at + 14)?;
        let size = read_u32(bytes, at + 18)? as usize;
        if read_u32(bytes, at + 22)? as usize != size {
            return Err(invalid("ZIP entry sizes disagree"));
        }
        let name_len = read_u16(bytes, at + 26)? as usize;
        let extra_len = read_u16(bytes, at + 28)? as usize;
        let name_start = at
            .checked_add(30)
            .ok_or_else(|| invalid("ZIP offset overflow"))?;
        let data_start = name_start
            .checked_add(name_len)
            .and_then(|n| n.checked_add(extra_len))
            .ok_or_else(|| invalid("ZIP offset overflow"))?;
        let end = data_start
            .checked_add(size)
            .ok_or_else(|| invalid("ZIP offset overflow"))?;
        let name = std::str::from_utf8(
            bytes
                .get(name_start..name_start + name_len)
                .ok_or_else(|| invalid("truncated ZIP name"))?,
        )
        .map_err(|_| invalid("non-UTF8 ZIP name"))?;
        if !name.contains('/')
            || name.starts_with('/')
            || name.contains("..")
            || name.contains('\\')
        {
            return Err(invalid("invalid ZIP entry path"));
        }
        let data = bytes
            .get(data_start..end)
            .ok_or_else(|| invalid("truncated ZIP entry"))?;
        if crc32(data) != crc || out.insert(name.to_owned(), data.to_vec()).is_some() {
            return Err(invalid("duplicate or corrupt ZIP entry"));
        }
        at = end;
    }
    if read_u32(bytes, at)? != 0x0201_4b50 {
        return Err(invalid("missing ZIP central directory"));
    }
    // Validate central directory and end marker, not only local file headers.
    let central_start = at;
    for (name, data) in &out {
        if read_u32(bytes, at)? != 0x0201_4b50 {
            return Err(invalid("missing ZIP central entry"));
        }
        let name_len = read_u16(bytes, at + 28)? as usize;
        let extra_len = read_u16(bytes, at + 30)? as usize;
        let comment_len = read_u16(bytes, at + 32)? as usize;
        let filename = bytes
            .get(at + 46..at + 46 + name_len)
            .ok_or_else(|| invalid("truncated ZIP central name"))?;
        if filename != name.as_bytes() || read_u32(bytes, at + 16)? != crc32(data) {
            return Err(invalid("ZIP central directory mismatch"));
        }
        at = at
            .checked_add(46 + name_len + extra_len + comment_len)
            .ok_or_else(|| invalid("ZIP offset overflow"))?;
    }
    if read_u32(bytes, at)? != 0x0605_4b50 || read_u16(bytes, at + 10)? as usize != out.len() {
        return Err(invalid("invalid ZIP end record"));
    }
    if read_u32(bytes, at + 12)? as usize != at - central_start
        || read_u32(bytes, at + 16)? as usize != central_start
        || read_u16(bytes, at + 20)? != 0
        || at + 22 != bytes.len()
    {
        return Err(invalid("invalid ZIP central directory size"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_corruption() {
        let mut input = BTreeMap::new();
        input.insert("root/include/terrakit.h".to_string(), b"header".to_vec());
        input.insert("root/lib/libterrakit.a".to_string(), vec![0, 1, 2, 255]);
        let zip = write(&input).unwrap();
        assert_eq!(read(&zip).unwrap(), input);
        let mut damaged = zip;
        damaged[58] ^= 0xff;
        assert!(read(&damaged).is_err());
    }
}
