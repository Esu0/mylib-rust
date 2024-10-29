pub mod traits;

pub fn gcd<T: traits::Integer + Eq + Clone>(mut a: T, mut b: T) -> T {
    let zero = T::zero();
    if a == zero {
        return b;
    }
    while b != zero {
        let tmp = a % b.clone();
        a = b;
        b = tmp;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_test() {
        let a = 18u32;
        let b = 24u32;
        assert_eq!(gcd(a, b), 6);
    }
}
