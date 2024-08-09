use std::{
    iter::FusedIterator,
    ops::{Add, Div, Mul, RangeBounds, Rem, Sub},
};

pub trait Integer:
    Sized
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + Ord
    + Copy
{
    const MIN: Self;
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
}

macro_rules! impl_integer {
    ($($t:ty),*) => {
        $(
            impl Integer for $t {
                const MIN: Self = <$t>::MIN;
                const MAX: Self = <$t>::MAX;
                const ZERO: Self = 0;
                const ONE: Self = 1;
                const TWO: Self = 2;
            }
        )*
    };
}

impl_integer!(i8, i16, i32, i64, i128, isize);
impl_integer!(u8, u16, u32, u64, u128, usize);

/// `range`が`l..r`で、返り値を`i`とすると、
/// `(l..i).contains(j)`となる`j`において、`f(j)`が`true`となり、
/// `(i..r).contains(j)`となる`j`において、`f(j)`が`false`となる。
pub fn upper_bound<T: Integer>(range: impl RangeBounds<T>, mut f: impl FnMut(T) -> bool) -> T {
    let mut l = match range.start_bound() {
        std::ops::Bound::Included(&l) => l,
        std::ops::Bound::Excluded(&l) => l + T::ONE,
        std::ops::Bound::Unbounded => T::MIN / T::TWO,
    };
    let mut r = match range.end_bound() {
        std::ops::Bound::Included(&r) => r + T::ONE,
        std::ops::Bound::Excluded(&r) => r,
        std::ops::Bound::Unbounded => T::MAX / T::TWO,
    };
    while r - l > T::ONE {
        let m = l + (r - l) / T::TWO;
        if f(m) {
            l = m + T::ONE;
        } else {
            r = m;
        }
    }
    if f(l) {
        l + T::ONE
    } else {
        l
    }
}

/// `range`が`l..r`で、返り値を`i`とすると、
/// `(l..i).contains(j)`となる`j`において、`f(j)`が`false`となり、
/// `(i..r).contains(j)`となる`j`において、`f(j)`が`true`となる。
pub fn lower_bound<T: Integer>(range: impl RangeBounds<T>, mut f: impl FnMut(T) -> bool) -> T {
    upper_bound(range, |x| !f(x))
}

pub trait IteratorExt: Iterator {
    /// 累積和を求める
    fn cumulative_sum<T, F>(self, init: T, f: F) -> CumSum<Self, T, F>
    where
        Self: Sized,
        F: FnMut(&T, Self::Item) -> T,
    {
        CumSum {
            iter: self,
            sum: Some(init),
            f,
        }
    }
}

impl<I: Iterator> IteratorExt for I {}

pub struct CumSum<I, T, F> {
    iter: I,
    sum: Option<T>,
    f: F,
}

impl<I, T, F> Iterator for CumSum<I, T, F>
where
    I: Iterator,
    F: FnMut(&T, I::Item) -> T,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sum) = self.sum.as_mut() {
            if let Some(next) = self.iter.next() {
                let next_sum = (self.f)(sum, next);
                Some(std::mem::replace(sum, next_sum))
            } else {
                self.sum.take()
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        (lower + 1, upper.map(|x| x + 1))
    }
}

impl<I, T, F> ExactSizeIterator for CumSum<I, T, F>
where
    I: ExactSizeIterator,
    F: FnMut(&T, I::Item) -> T,
{
    fn len(&self) -> usize {
        self.iter.len() + 1
    }
}

impl<I, T, F> FusedIterator for CumSum<I, T, F>
where
    I: Iterator,
    F: FnMut(&T, I::Item) -> T,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_bound_test() {
        let v = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        assert_eq!(upper_bound(0..v.len(), |i| v[i] < 10), 4);
        assert_eq!(upper_bound(0..v.len(), |i| v[i] < 11), 4);
        assert_eq!(upper_bound(0..v.len(), |i| v[i] < 11), 4);
        assert_eq!(upper_bound(0..v.len(), |i| v[i] < 12), 5);
        assert_eq!(lower_bound(0..v.len(), |i| v[i] > 17), 7);
        assert_eq!(lower_bound(0..v.len(), |i| v[i] > 1), 0);
        assert_eq!(lower_bound(0..v.len(), |i| v[i] > 2), 1);
    }

    #[test]
    fn cum_sum_test() {
        let v = [1, 2, 3, 4, 5];
        let cum_sum = v.iter().cumulative_sum(0, |&sum, &x| sum + x).collect::<Vec<_>>();
        assert_eq!(cum_sum[..], [0, 1, 3, 6, 10, 15]);
    }
}
