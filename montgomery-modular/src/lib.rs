pub struct Montgomery<const R: u64> {
    n: u32,
    n_prime: u32,
    r2: u32,
}

pub fn gcd_ext(a: i64, b: i64) -> (i64, i64, i64) {
    use std::cmp::Ordering::*;
    let (mut r0, mut x0) = match a.cmp(&0) {
        Less => (-a, -1),
        Equal => return (b, 0, 1),
        Greater => (a, 1),
    };
    let (mut r1, mut x1) = match b.cmp(&0) {
        Less => (-b, -1),
        Equal => return (a, 1, 0),
        Greater => (b, 1),
    };
    while r1 != 0 {
        let q = r0 / r1;
        let r = r0 % r1;
        r0 = r1;
        r1 = r;
        let x = x0 - q * x1;
        x0 = x1;
        x1 = x;
    }
    (r0, x0, (r0 - a * x0) / b)
}

impl<const R: u64> Montgomery<R> {
    const R_IS_POWER_OF_TWO: bool = R.is_power_of_two();
    const MASK: u64 = R - 1;
    const SHIFT: u32 = R.trailing_zeros();

    fn compute_n_prime(n: u32) -> u32 {
        if Self::R_IS_POWER_OF_TWO {
            assert_eq!(n & 1, 1, "n and r must be coprime.");
            assert!((n as u64) < R, "n must be less than R.");
            let mut b = 2;
            let mut result = 1;
            let mut r = n as u64;
            let mut n = (n as u64) << 1;
            while b < R {
                if r & b == 0 {
                    result |= b;
                    r += n;
                }
                n <<= 1;
                b <<= 1;
            }
            result as u32
        } else {
            panic!("R is not power of two");
            // let (g, x, _) = gcd_ext(-(n as i64), R as i64);
            // assert_eq!(g, 1, "n and R must be coprime.");
            // x as u32
        }
    }

    pub fn new(n: u32) -> Self {
        Self {
            n,
            n_prime: Self::compute_n_prime(n),
            r2: {
                let n = n as u64;
                ((R % n) * (R % n) % n) as u32
            },
        }
    }

    fn rem_r(a: u64) -> u64 {
        a & Self::MASK
    }

    fn div_r(a: u64) -> u64 {
        a >> Self::SHIFT
    }

    /// Compute `(a * R^(-1)) % n`
    /// when `0 <= a < n * R`
    pub fn reduce(&self, a: u64) -> u32 {
        let n = self.n as u64;
        let t = Self::div_r(a + Self::rem_r(a.wrapping_mul(self.n_prime as u64)) * n);
        (if t >= n { t - n } else { t }) as u32
    }

    /// Compute `(a * b) % n`
    /// when `0 <= a < n` and `0 <= b < n`
    pub fn multiply(&self, a: u32, b: u32) -> u32 {
        let c = self.reduce(a as u64 * b as u64);
        self.reduce(c as u64 * self.r2 as u64)
    }

    /// Compute `(a * R) % n`
    /// when `0 <= a < n`
    pub fn multiply_r(&self, a: u32) -> u32 {
        self.reduce(a as u64 * self.r2 as u64)
    }

    /// Compute `a^exp % n`
    pub fn pow(&self, a: u32, mut exp: u64) -> u32 {
        let mut result = 1;
        let mut base = self.multiply_r(a); // a * R
        while exp > 0 {
            if exp & 1 == 1 {
                result = self.reduce(result as u64 * base as u64);
            }
            base = self.reduce(base as u64 * base as u64);
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
        let m = Montgomery::<8>::new(5);
        assert_eq!(m.n_prime, 3);
        assert_eq!(m.r2, 4);
    }

    #[test]
    fn multiply_test() {
        let m = Montgomery::<256>::new(167);
        assert_eq!(m.multiply(123, 45), 123 * 45 % 167);
    }

    #[test]
    #[should_panic]
    fn new_test_panic() {
        Montgomery::<256>::new(150);
    }

    #[test]
    fn multiply_random() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        const R: u64 = u32::MAX as u64 + 1;
        let n = rng.gen_range(1..(R / 4)) as u32 * 2 + 1;
        let m = Montgomery::<R>::new(n);
        for _ in 0..10000 {
            let a = rng.gen_range(0..n);
            let b = rng.gen_range(0..n);
            assert_eq!(m.multiply(a, b) as u64, a as u64 * b as u64 % n as u64);
        }
    }

    #[test]
    fn pow_test() {
        let m = Montgomery::<256>::new(101);
        assert_eq!(m.pow(2, 10), 2u32.pow(10) % 101);
        assert_eq!(m.pow(43, 5), 43u32.pow(5) % 101);
    }
}
