use std::{fmt::{self, Debug, Display}, mem::MaybeUninit, ops::Add};


#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[allow(non_camel_case_types)]
pub struct u256 {
    data: [u64; 4],
}

impl u256 {
    pub fn from_u64(x: u64) -> Self {
        Self { data: [x, 0, 0, 0] }
    }

    fn div_10(mut self) -> (u64, Self) {
        let mut rem = self.data[3] % 10;
        self.data[3] /= 10;
        const BASE_REM: u64 = u64::MAX % 10 + 1;
        const BASE_DIV: u64 = u64::MAX / 10;
        for d in self.data[..3].iter_mut().rev() {
            let r = *d % 10 + rem * BASE_REM;
            *d /= 10;
            *d += rem * BASE_DIV + r / 10;
            rem = r % 10;
        }
        (rem, self)
    }

    /// n < 64
    fn lshift(mut self, n: u32) -> (Self, bool) {
        if n == 0 {
            return (self, false);
        }
        let (d, ret) = self.data[3].overflowing_shl(n);
        self.data[3] = d;
        self.data[3] |= self.data[2] >> (64 - n);
        self.data[2] = (self.data[2] << n) | (self.data[1] >> (64 - n));
        self.data[1] = (self.data[1] << n) | (self.data[0] >> (64 - n));
        self.data[0] <<= n;
        (self, ret)
    }

    fn mul_10(mut self) -> (Self, bool) {
        let (n2, flg1) = self.lshift(1);
        let (n8, flg2) = self.lshift(3);
        let (n10, flg3) = n2.overflowing_add(n8);
        (n10, flg1 || flg2 || flg3)
    }

    const fn max() -> Self {
        Self { data: [u64::MAX; 4] }
    }

    pub fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let mut carry = 0;
        let mut data: [MaybeUninit<u64>; 4] = unsafe { MaybeUninit::uninit().assume_init() };
        for ((&a, &b), r) in self.data.iter().zip(&rhs.data).zip(&mut data) {
            let (mut sum, mut c) = a.overflowing_add(b);
            if c {
                r.write(sum + carry);
            } else {
                (sum, c) = sum.overflowing_add(carry);
                r.write(sum);
            }
            carry = c as u64;
        }
        (Self {
            data: unsafe { std::mem::transmute(data) },
        }, carry != 0)
    }
    pub const MAX: Self = Self::max();
}

impl Add for u256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut carry = 0;
        let mut data: [MaybeUninit<u64>; 4] = unsafe { MaybeUninit::uninit().assume_init() };
        for ((&a, &b), r) in self.data.iter().zip(&rhs.data).zip(&mut data) {
            let (mut sum, mut c) = a.overflowing_add(b);
            if c {
                r.write(sum + carry);
            } else {
                (sum, c) = sum.overflowing_add(carry);
                r.write(sum);
            }
            carry = c as u64;
        }
        Self {
            data: unsafe { std::mem::transmute(data) },
        }
    }
}

impl Debug for u256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl Display for u256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s: [MaybeUninit<u8>; 78] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut s_head = 78usize;
        let mut n = *self;
        if n.data == [0, 0, 0, 0] {
            s_head -= 1;
            s[s_head].write(b'0');
        } else {
            while {
                let (rem, next) = n.div_10();
                s_head -= 1;
                s[s_head].write(b'0' + rem as u8);
                n = next;
                n.data != [0, 0, 0, 0]
            } {}
        }
        let s = unsafe { std::mem::transmute::<_, &[u8]>(&s[s_head..]) };
        unsafe {
            write!(f, "{}", std::str::from_utf8_unchecked(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div10() {
        let mut n = u256 { data: [0, 0, 0, 1] };
        let ans = b"6277101735386680763835789423207666416102355444464034512896";
        for &c in ans.iter().rev() {
            let (rem, next) = n.div_10();
            assert_eq!(c, b'0' + rem as u8);
            n = next;
        }

        let mut n = u256 { data: [3929000416921158139, 14534595274299777775, 17937483865664372264, 13373398346035241992] };
        let ans = b"83946181965915183772757055874729772429600189856348213023901615996796071477755";
        for &c in ans.iter().rev() {
            let (rem, next) = n.div_10();
            assert_eq!(c, b'0' + rem as u8);
            n = next;
        }

        let mut n = u256::MAX;
        let ans = b"115792089237316195423570985008687907853269984665640564039457584007913129639935";
        for &c in ans.iter().rev() {
            let (rem, next) = n.div_10();
            assert_eq!(c, b'0' + rem as u8);
            n = next;
        }
    }

    #[test]
    fn to_string() {
        let n = u256 { data: [0, 0, 0, 1] };
        assert_eq!(n.to_string(), "6277101735386680763835789423207666416102355444464034512896");
        let n = u256::from_u64(0);
        assert_eq!(n.to_string(), "0");
        let n = u256::MAX;
        assert_eq!(n.to_string(), "115792089237316195423570985008687907853269984665640564039457584007913129639935");
        let n = u256 { data: [3929000416921158139, 14534595274299777775, 17937483865664372264, 13373398346035241992] };
        assert_eq!(n.to_string(), "83946181965915183772757055874729772429600189856348213023901615996796071477755");
    }
}
