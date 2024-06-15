#![allow(clippy::missing_transmute_annotations)]
use std::{fmt::{self, Debug, Display}, mem::MaybeUninit, ops::{Add, AddAssign, Sub, SubAssign}};

/// N * 64 bit unsigned integer
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixedUInt<const N: usize> {
    data: [u64; N],
}

#[allow(non_camel_case_types)]
pub type u256 = FixedUInt<4>;

impl<const N: usize> Default for FixedUInt<N> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<const N: usize> FixedUInt<N> {
    const fn assert_zero_size() {
        assert!(N > 0, "size must be greater than 0");
    }

    pub const fn from_u64(x: u64) -> Self {
        Self::assert_zero_size();
        let mut ret = Self { data: [0; N] };
        ret.data[0] = x;
        ret
    }

    pub const fn zero() -> Self {
        Self::assert_zero_size();
        Self { data: [0; N] }
    }

    pub const fn one() -> Self {
        Self::from_u64(1)
    }

    const fn div10(mut self) -> (Self, u64) {
        let mut i = N - 1;
        let mut rem = self.data[i] % 10;
        self.data[i] /= 10;
        const BASE_REM: u64 = u64::MAX % 10 + 1;
        const BASE_DIV: u64 = u64::MAX / 10;
        while i > 0 {
            i -= 1;
            let r = self.data[i] % 10 + rem * BASE_REM;
            self.data[i] /= 10;
            self.data[i] += rem * BASE_DIV + r / 10;
            rem = r % 10;
        }
        (self, rem)
    }

    /// n < 64
    const fn lshift(mut self, n: u32) -> (Self, bool) {
        if n == 0 {
            return (self, false);
        }
        let amount_of_rshift = 64 - n;
        let mut i = 1;
        let mut d = self.data[0] >> amount_of_rshift;
        self.data[0] <<= n;
        while i < N {
            let next_d = self.data[i] >> amount_of_rshift;
            self.data[i] = (self.data[i] << n) | d;
            d = next_d;
            i += 1;
        }
        (self, d != 0)
    }

    const fn mul10(self) -> (Self, bool) {
        let (n2, flg1) = self.lshift(1);
        let (n8, flg2) = self.lshift(3);
        let (n10, flg3) = n2.overflowing_add(n8);
        (n10, flg1 || flg2 || flg3)
    }

    const fn mul_u64(self, rhs: u64) -> (u64, Self) {
        let mut carry = 0u64;
        let mut data: [MaybeUninit<u64>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut i = 0;
        while i < N {
            let mul = self.data[i] as u128 * rhs as u128 + carry as u128;
            data[i] = MaybeUninit::new(mul as u64);
            carry = (mul >> 64) as u64;
            i += 1;
        }
        let ptr = &data as *const _ as *const [u64; N];
        (carry, Self {
            data: unsafe { ptr.read() },
        })
    }

    pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let mut carry = 0;
        let mut data: [MaybeUninit<u64>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut i = 0;
        while i < N {
            let (mut sum, mut c) = self.data[i].overflowing_add(rhs.data[i]);
            if c {
                data[i] = MaybeUninit::new(sum + carry);
            } else {
                (sum, c) = sum.overflowing_add(carry);
                data[i] = MaybeUninit::new(sum);
            }
            carry = c as u64;
            i += 1;
        }
        let ptr = &data as *const _ as *const [u64; N];
        (Self {
            data: unsafe { ptr.read() },
        }, carry != 0)
    }

    const fn max() -> Self {
        Self { data: [u64::MAX; N] }
    }

    pub const MAX: Self = Self::max();

    pub const fn add_const(self, rhs: Self) -> Self {
        self.overflowing_add(rhs).0
    }

    pub const fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        let mut borrow = 0;
        let mut data: [MaybeUninit<u64>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut i = 0;
        while i < N {
            let (mut diff, mut b) = self.data[i].overflowing_sub(rhs.data[i]);
            if b {
                data[i] = MaybeUninit::new(diff - borrow);
            } else {
                (diff, b) = diff.overflowing_sub(borrow);
                data[i] = MaybeUninit::new(diff);
            }
            borrow = b as u64;
            i += 1;
        }
        let ptr = &data as *const _ as *const [u64; N];
        (Self {
            data: unsafe { ptr.read() },
        }, borrow != 0)
    }

    pub const fn sub_const(self, rhs: Self) -> Self {
        self.overflowing_sub(rhs).0
    }
}

impl<const N: usize> Add for FixedUInt<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.overflowing_add(rhs).0
    }
}

impl<const N: usize> AddAssign for FixedUInt<N> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<const N: usize> Sub for FixedUInt<N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.overflowing_sub(rhs).0
    }
}

impl<const N: usize> SubAssign for FixedUInt<N> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
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
                let (next, rem) = n.div10();
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
            let (next, rem) = n.div10();
            assert_eq!(c, b'0' + rem as u8);
            n = next;
        }

        let mut n = u256 { data: [3929000416921158139, 14534595274299777775, 17937483865664372264, 13373398346035241992] };
        let ans = b"83946181965915183772757055874729772429600189856348213023901615996796071477755";
        for &c in ans.iter().rev() {
            let (next, rem) = n.div10();
            assert_eq!(c, b'0' + rem as u8);
            n = next;
        }

        let mut n = u256::MAX;
        let ans = b"115792089237316195423570985008687907853269984665640564039457584007913129639935";
        for &c in ans.iter().rev() {
            let (next, rem) = n.div10();
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

    #[test]
    fn mul10() {
        let mut n = u256 { data: [1, 0, 0, 0] };
        n = n.mul10().0;
        assert_eq!(n, u256 { data: [10, 0, 0, 0] });
        n = n.mul10().0;
        assert_eq!(n, u256 { data: [100, 0, 0, 0] });
        n = n.mul10().0;
        assert_eq!(n, u256 { data: [1000, 0, 0, 0] });
        for _ in 0..20 {
            n = n.mul10().0;
        }
        assert_eq!(n.to_string(), "100000000000000000000000");
        for _ in 0..14 {
            n = n.mul10().0;
        }
        assert_eq!(n.to_string(), "10000000000000000000000000000000000000");
        n = u256::MAX;
        assert!(n.mul10().1);
        n = n.div10().0 + u256::from_u64(1);
        assert_eq!(n.data, [0x9999_9999_9999_999a, 0x9999_9999_9999_9999, 0x9999_9999_9999_9999, 0x1999_9999_9999_9999]);
        let (n, flg) = n.mul10();
        assert!(flg);
        assert_eq!(n.data, [4, 0, 0, 0]);
    }

    #[test]
    #[should_panic]
    fn zero_size_int() {
        let _ = FixedUInt::<0>::zero();
    }

    #[test]
    fn mul_u64() {
        // calculate 4389069123540196268 * 1764497475677533644 * 18129258545924030117 * 14030216130287452147
        let n = u256::from_u64(4389069123540196268);
        let (carry, n) = n.mul_u64(1764497475677533644);
        assert_eq!(carry, 0);
        assert_eq!(n.data, [0x6aa4_bcc9_5806_a910, 0x05d3_8966_0529_274f, 0, 0]);
        let (carry, n) = n.mul_u64(18129258545924030117);
        assert_eq!(carry, 0);
        assert_eq!(n.data, [0x96fa_a104_6d7c_d750, 0xe4ed_54d3_16aa_19e9, 0x05b9_ddb1_4b12_ea1f, 0]);
        let (carry, n) = n.mul_u64(14030216130287452147);
        assert_eq!(carry, 0);
        assert_eq!(n.data, [0xc350_3c52_cc8d_10f0, 0x241b_e060_734b_a730, 0xaa09_fa62_de2c_a222, 0x045a_e85d_9c64_ca24]);
    }
}
