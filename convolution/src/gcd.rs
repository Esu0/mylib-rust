use std::ops::Deref;

#[derive(Debug, PartialEq, Eq)]
pub struct ZetaDiv<'a, T> {
    data: &'a mut [T],
}

unsafe fn borrow_two<T>(a: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    debug_assert!(i != j);
    debug_assert!(i < a.len());
    debug_assert!(j < a.len());
    let ptr = a.as_mut_ptr();
    let a = &mut *ptr.add(i);
    let b = &mut *ptr.add(j);
    (a, b)
}

impl<'a, T> Deref for ZetaDiv<'a, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> ZetaDiv<'a, T> {
    pub fn from_slice(data: &'a mut [T]) -> Self {
        Self { data }
    }

    pub fn new<F>(data: &'a mut [T], mut add: F) -> Self
    where
        F: FnMut(&mut T, &T),
    {
        let mut rt = (data.len() as f64).sqrt() as usize;
        while rt * rt <= data.len() {
            rt += 1;
        }
        rt -= 1;
        // is_prime[i] = true if i + 1 is prime
        let mut is_prime = vec![true; data.len()];
        for i in 2..=rt {
            if is_prime[i - 1] {
                let mut j = data.len() / i - 1;
                while {
                    let k = (j + 1) * i - 1;
                    is_prime[k] = false;
                    unsafe {
                        let (d, s) = borrow_two(data, j, k);
                        add(d, s);
                    }
                    j > 0
                } {
                    j -= 1;
                }
            }
        }
        for i in rt + 1..=data.len() {
            if is_prime[i - 1] {
                let mut j = data.len() / i - 1;
                while {
                    let k = (j + 1) * i - 1;
                    unsafe {
                        let (d, s) = borrow_two(data, j, k);
                        add(d, s);
                    }
                    j > 0
                } {
                    j -= 1;
                }
            }
        }
        Self { data }
    }

    pub fn hadamard<P>(&mut self, other: &Self, mut prod: P)
    where
        P: FnMut(&mut T, &T),
    {
        if self.data.len() != other.data.len() {
            panic!("Length mismatch");
        }
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            prod(a, b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// returns (gcd, x, y) such that a * x + b * y = gcd
    fn gcd_ext(a: u64, b: u64) -> (i64, i64, i64) {
        if b == 0 {
            (a as i64, 1, 0)
        } else {
            // b * x + (a % b) * y = gcd
            let (gcd, x, y) = gcd_ext(b, a % b);
            let d = a / b;
            // b * x - d * b * y + (a % b) * y + d * b * y = b * (x - d * y) + a * y = gcd
            (gcd, y, x - d as i64 * y)
        }
    }

    fn gcd(a: u64, b: u64) -> u64 {
        gcd_ext(a, b).0 as u64
    }

    #[test]
    fn zetadiv_transform() {
        let data = [1, 3, 2, 0, 4, 2, 2, 6, 8, 7];
        let mut borrowed = data;
        let mut zetadiv = ZetaDiv::new(&mut borrowed, |a, b| *a += *b);
        assert_eq!(zetadiv[..], [35, 18, 12, 6, 11, 2, 2, 6, 8, 7]);
        let mut cloned_data = zetadiv.to_vec();
        let cloned = ZetaDiv::from_slice(&mut cloned_data);
        zetadiv.hadamard(&cloned, |a, b| *a *= *b);

        let mut gcd_conv = [0; 10];
        for i in 1..=10 {
            for j in 1..=10 {
                let g = gcd(i, j);
                gcd_conv[g as usize - 1] += data[i as usize - 1] * data[j as usize - 1];
            }
        }

        let gcd_conv_zeta = ZetaDiv::new(&mut gcd_conv, |a, b| *a += *b);
        assert_eq!(zetadiv[..], gcd_conv_zeta[..]);
    }
}
