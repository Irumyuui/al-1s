// From leveldb impelment.

pub fn hash(data: &[u8], seed: u32) -> u32 {
    const MOD: u32 = 0xc6a4a793;
    let size = data.len() as u32;
    let mut h = seed ^ (size.wrapping_mul(MOD));

    let mut i = 0;
    while i + 4 <= data.len() {
        let w = u32::from_le_bytes(data[i..i + 4].try_into().unwrap());
        h = h.wrapping_add(w).wrapping_mul(MOD);
        h ^= h >> 16;
        i += 4;
    }

    let rest = &data[i..];

    match rest.len() {
        3 => {
            h = h
                .wrapping_add((rest[2] as u32) << 16)
                .wrapping_add((rest[1] as u32) << 8)
                .wrapping_add(rest[0] as u32);
            h = h.wrapping_mul(MOD);
            h ^= h >> 24;
        }
        2 => {
            h = h
                .wrapping_add((rest[1] as u32) << 8)
                .wrapping_add(rest[0] as u32);
            h = h.wrapping_mul(MOD);
            h ^= h >> 24;
        }
        1 => {
            h = h.wrapping_add(rest[0] as u32);
            h = h.wrapping_mul(MOD);
            h ^= h >> 24;
        }
        _ => {}
    }

    h
}

#[cfg(test)]
mod tests {
    use super::hash;

    #[test]
    fn zero_data() {
        let data = [];
        let seed = 0xbc9f1d34;

        let result = hash(&data, seed);

        assert_eq!(result, seed);
    }

    #[test]
    fn one_data() {
        let data = [42];
        let seed = 0xbc9f1d34;

        let result = hash(&data, seed);

        assert_eq!(result, 0x7B0E_9D78);
    }

    #[test]
    fn long_data() {
        let data = [
            0x6d, 0x75, 0x91, 0x6d, 0x20, 0x80, 0x12, 0x73, 0xb5, 0x8d, 0x90, 0x29, 0xc5, 0x2a,
            0x80, 0x5f, 0xf, 0x7d, 0xbd, 0xf5, 0x6e, 0x33, 0x91, 0x30, 0x62, 0xc, 0x6, 0x11, 0xa2,
            0x6f, 0xcb, 0x4b, 0x50, 0x9f, 0x48, 0x5f, 0xdf, 0x5c, 0xe7, 0x41, 0x6f, 0xb8, 0x44,
            0x7d, 0x68, 0x84, 0xb6, 0x7e, 0x5, 0xdd, 0x66, 0xef, 0x74, 0xba, 0xf8, 0xf5, 0xb0,
            0x9d, 0x25, 0xdc, 0x50, 0x8b, 0x45, 0xb8, 0x0, 0xef, 0x89, 0xb5, 0x5, 0x42, 0xd3, 0x36,
            0x8f, 0x37, 0xdc, 0x18, 0x25, 0x74, 0x3b, 0x3f, 0x67, 0xc7, 0x37, 0x58, 0x5, 0x76,
            0x6e, 0xc8, 0x4d, 0x18, 0xd2, 0x5b, 0xb2, 0x30, 0xb4, 0x0, 0x13, 0xd0, 0x88, 0x63,
        ];
        let seed = 0xbc9f1d34;

        let result = hash(&data, seed);
        assert_eq!(result, 0x_5400_451F);
    }
}
