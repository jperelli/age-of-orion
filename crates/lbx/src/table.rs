use crate::{cp437, u16_at, Error, Result};

/// A type-5 data entry: `u16 count, u16 record_width`, then `count` fixed-width records.
#[derive(Debug, Clone)]
pub struct Table<'a> {
    pub width: usize,
    records: Vec<&'a [u8]>,
}

impl<'a> Table<'a> {
    pub fn parse(entry: &'a [u8]) -> Result<Self> {
        let count = u16_at(entry, 0).ok_or(Error::Truncated("table header"))? as usize;
        let width = u16_at(entry, 2).ok_or(Error::Truncated("table header"))? as usize;
        let records = (0..count)
            .map(|i| entry.get(4 + i * width..4 + (i + 1) * width))
            .collect::<Option<Vec<_>>>()
            .ok_or(Error::Truncated("table records"))?;
        Ok(Table { width, records })
    }

    pub fn records(&self) -> &[&'a [u8]] {
        &self.records
    }

    /// Records as NUL-terminated CP437 strings.
    pub fn strings(&self) -> Vec<String> {
        self.records.iter().map(|r| decode_cp437(r)).collect()
    }
}

/// Decodes a NUL-terminated CP437 byte string.
pub fn decode_cp437(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| {
            if b < 0x80 {
                b as char
            } else {
                cp437::HIGH[(b - 0x80) as usize]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_fixed_width_strings() {
        let mut e = vec![2, 0, 6, 0];
        e.extend(b"Sol\0\0\0");
        e.extend([0x80, b'x', 0, 0, 0, 0]);
        let t = Table::parse(&e).unwrap();
        assert_eq!(t.width, 6);
        assert_eq!(t.strings(), vec!["Sol".to_string(), "Çx".to_string()]);
    }
}
