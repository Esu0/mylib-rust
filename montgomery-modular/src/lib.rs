use number::traits::{Arithmetic, BitArithmetic, ExistsBiggerInt, Integer, WrappingOps};

pub struct Montgomery<T: Arithmetic<BitWidthType = u32> + BitArithmetic + ExistsBiggerInt> {
    n: T,
    n_prime: T,
    mask: T::Twice,
    r_log2: u32,
    r2: T,
}

impl<T> Montgomery<T>
where
    T: Arithmetic<BitWidthType = u32> + BitArithmetic + ExistsBiggerInt + Ord + Copy,
    T::Twice: Arithmetic<BitWidthType = u32> + BitArithmetic + Copy + WrappingOps,
{
    fn compute_n_prime(n: T, r: T::Twice) -> T {
        let one = T::one();
        let two = T::Twice::one() << 1;
        if n & one != one {
            panic!("n and r must be coprime");
        }
        if n.cast_to_twice() >= r {
            panic!("n must be less than R.");
        }
        let mut b = two;
        let mut result = T::Twice::one();
        let mut rr = n.cast_to_twice();
        let mut n = rr << 1;
        while b < r {
            if rr & b == T::Twice::zero() {
                result |= b;
                rr += n;
            }
            n <<= 1;
            b <<= 1;
        }
        T::cast_from_twice(result)
    }

    pub fn new(n: T, r_log2: u32) -> Self {
        let r = T::Twice::one() << r_log2;
        Self {
            n,
            n_prime: Self::compute_n_prime(n, r),
            r_log2,
            mask: r - T::Twice::one(),
            r2: T::cast_from_twice({
                let n = n.cast_to_twice();
                let r = r % n;
                r * r % n
            }),
        }
    }

    fn rem_r(&self, a: T::Twice) -> T::Twice {
        a & self.mask
    }

    fn div_r(&self, a: T::Twice) -> T::Twice {
        a >> self.r_log2
    }

    /// Compute `(a * R^(-1)) % n`
    /// when `0 <= a < n * R`
    pub fn reduce(&self, a: T::Twice) -> T {
        let n = self.n.cast_to_twice();
        let t = self.div_r(a + self.rem_r(a.wrapping_mul(self.n_prime.cast_to_twice())) * n);
        T::cast_from_twice(if t >= n { t - n } else { t })
    }

    /// Compute `(a * b) % n`
    /// when `0 <= a < n` and `0 <= b < n`
    pub fn multiply(&self, a: T, b: T) -> T {
        let c = self.reduce(a.cast_to_twice() * b.cast_to_twice());
        self.reduce(c.cast_to_twice() * self.r2.cast_to_twice())
    }

    /// Compute `(a * R) % n`
    /// when `0 <= a < n`
    pub fn multiply_r(&self, a: T) -> T {
        self.reduce(a.cast_to_twice() * self.r2.cast_to_twice())
    }

    /// Compute `a^exp % n`
    pub fn pow(&self, a: T, mut exp: u64) -> T {
        let mut result = T::one();
        let mut base = self.multiply_r(a); // a * R
        while exp > 0 {
            if exp & 1 == 1 {
                result = self.reduce(result.cast_to_twice() * base.cast_to_twice());
            }
            base = self.reduce(base.cast_to_twice() * base.cast_to_twice());
            exp >>= 1;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_test() {
        let m = Montgomery::new(5u32, 3);
        assert_eq!(m.n_prime, 3);
        assert_eq!(m.r2, 4);
    }

    #[test]
    fn multiply_test() {
        let m = Montgomery::new(167u32, 8);
        assert_eq!(m.multiply(123, 45), 123 * 45 % 167);
    }

    #[test]
    #[should_panic]
    fn new_test_panic() {
        Montgomery::new(150u32, 8);
    }

    #[test]
    fn multiply_random() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        const R: u64 = u32::MAX as u64 + 1;
        let n = rng.gen_range(1..(R / 4)) as u32 * 2 + 1;
        let m = Montgomery::new(n, 32);
        for _ in 0..10000 {
            let a = rng.gen_range(0..n);
            let b = rng.gen_range(0..n);
            assert_eq!(m.multiply(a, b) as u64, a as u64 * b as u64 % n as u64);
        }
    }

    #[test]
    fn pow_test() {
        let m = Montgomery::new(101, 8);
        assert_eq!(m.pow(2, 10), 2u32.pow(10) % 101);
        assert_eq!(m.pow(43, 5), 43u32.pow(5) % 101);
    }
}
