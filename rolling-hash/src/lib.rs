use std::ops::RangeBounds;

pub struct RollingHasher {
    modulo: u64,
    /// `exponents[i] = base^(i + 1) % modulo`
    exponents: Vec<u64>,
    hash: Vec<u64>,
}

const MODULO_DEFAULT: u64 = (1 << 61) - 1;

impl RollingHasher {
    pub fn new(base: u64, data: impl IntoIterator<Item = u64>) -> Self {
        let mut e = 1u64;
        let modulo = MODULO_DEFAULT;
        let base = base as u128;
        let mut tmp = 0u64;
        let hash = data.into_iter().map(|x| {
            tmp = ((tmp as u128 * base + x as u128) % modulo as u128) as u64;
            tmp
        }).collect::<Vec<_>>();
        let exponents = std::iter::repeat_with(|| {
            e = (e as u128 * base % modulo as u128) as u64;
            e
        }).take(hash.len() - 1).collect();
        Self {
            modulo,
            exponents,
            hash,
        }
    }

    pub fn hash(&self, range: impl RangeBounds<usize>) -> u64 {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&x) => x,
            std::ops::Bound::Excluded(&x) => x + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&x) => x + 1,
            std::ops::Bound::Excluded(&x) => x,
            std::ops::Bound::Unbounded => self.hash.len(),
        };
        let mut ret = self.hash[end - 1] as i128;
        let modulo = self.modulo as i128;
        if start > 0 {
            ret = (ret - self.hash[start - 1] as i128 * self.exponents[end - start - 1] as i128).rem_euclid(modulo);
        }
        ret as _
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data = [2, 3, 4, 5, 2, 3, 4, 5, 2, 4, 3, 5];
        let rh = RollingHasher::new(1009, data.iter().copied());
        assert_eq!(rh.hash(0..4), rh.hash(4..8));
        assert_eq!(rh.hash(0..3), rh.hash(4..7));
        assert_eq!(rh.hash(1..4), rh.hash(5..8));
        assert_eq!(rh.hash(0..5), rh.hash(4..9));
        println!("{:?}: {}", &data[0..3], rh.hash(0..3));
        println!("{:?}: {}", &data[8..11], rh.hash(8..11));
    }
}