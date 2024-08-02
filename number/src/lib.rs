use traits::Signed;

pub mod traits;

pub fn gcd_ext<T: Signed>(a: T, b: T) -> (T, T, T) {
    let mut r0 = a;
    let mut r1 = b;
    let mut x0 = T::ONE;
    let mut x1 = T::ZERO;
    while r1 != T::ZERO {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_ext_test() {
        assert_eq!(gcd_ext(10, 6), (2, -1, 2));
        assert_eq!(gcd_ext(6, 10), (2, 2, -1));
        assert_eq!(gcd_ext(10, 5), (5, 0, 1));
        assert_eq!(gcd_ext(5, 10), (5, 1, 0));
        assert_eq!(gcd_ext(10, 3), (1, 1, -3));
        assert_eq!(gcd_ext(3, 10), (1, -3, 1));
    }
}