// From leveldb impelment.

use bytes::{Buf, BufMut};

#[derive(Debug, thiserror::Error)]
pub enum BloomError {
    #[error("crc32 not match")]
    Crc32NotMatch,
}

pub struct Bloom {
    bits: Vec<u8>,
    k: usize,
}

fn hash(data: &[u8]) -> u32 {
    crate::utils::hash::hash(data, 0xbc9f1d34)
}

impl Bloom {
    pub fn new(entries: usize, bits_per_key: usize) -> Self {
        let k = ((bits_per_key as f64 * 0.69).round() as usize)
            .min(30)
            .max(1);

        let bit_len = (bits_per_key * entries).max(64);
        let byte_len = (bit_len + 7) / 8;
        let _bit_len = byte_len * 8;

        let bits = vec![0; byte_len];

        Bloom { bits, k }
    }

    pub fn insert(&mut self, key: impl AsRef<[u8]>) {
        let mut h = hash(key.as_ref());
        let delta = (h >> 17) | (h << 15);

        for _i in 0..self.k {
            let bit_pos = h % ((self.bits.len() * 8) as u32);
            self.bits[bit_pos as usize / 8] |= 1 << (bit_pos % 8);
            h = h.wrapping_add(delta);
        }
    }

    pub fn may_contain(&self, key: impl AsRef<[u8]>) -> bool {
        let mut h = hash(key.as_ref());
        let delta = (h >> 17) | (h << 15);

        for _ in 0..self.k {
            let bit_pos = h % ((self.bits.len() * 8) as u32);
            if (self.bits[bit_pos as usize / 8] & (1 << (bit_pos % 8))) == 0 {
                return false;
            }
            h = h.wrapping_add(delta);
        }

        true
    }

    pub fn encode(&self, buf: &mut Vec<u8>) {
        let mut data = self.bits.clone();
        data.put_u8(self.k as u8);
        let checksum = crc32fast::hash(&data);

        buf.put_u32(checksum);
        buf.extend(data);
    }

    pub fn decode(buf: impl AsRef<[u8]>) -> Result<Self, BloomError> {
        let mut buf = buf.as_ref();

        let checksum = buf.get_u32();
        let data = &buf[..];

        if crc32fast::hash(data) != checksum {
            return Err(BloomError::Crc32NotMatch);
        }

        let k = data[data.len() - 1] as usize;
        let bits = data[..data.len() - 1].to_vec();

        Ok(Bloom { bits, k })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_contains() {
        let mut bloom = Bloom::new(2, 10);
        bloom.insert("hello");
        bloom.insert("world");

        assert!(bloom.may_contain("hello"));
        assert!(bloom.may_contain("world"));
        assert!(!bloom.may_contain("rust"));
    }

    #[test]
    fn encode_decode() {
        let mut bloom = Bloom::new(2, 10);
        bloom.insert("hello");
        bloom.insert("world");

        let mut buf = Vec::new();
        bloom.encode(&mut buf);

        let bloom = Bloom::decode(&buf).unwrap();
        assert!(bloom.may_contain("hello"));
        assert!(bloom.may_contain("world"));
        assert!(!bloom.may_contain("rust"));
    }
}
