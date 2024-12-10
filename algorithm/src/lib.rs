use std::{iter::FusedIterator, num::NonZeroUsize};

pub trait IteratorExt: Iterator {
    fn run_length(mut self) -> RunLength<Self, Self::Item>
    where
        Self: Sized,
        Self::Item: Eq,
    {
        RunLength {
            prev: self.next(),
            iter: self,
        }
    }
}

pub struct RunLength<I, T> {
    iter: I,
    prev: Option<T>,
}

impl<I, T> Iterator for RunLength<I, T>
where
    I: Iterator<Item = T>,
    T: Eq,
{
    type Item = (NonZeroUsize, T);
    fn next(&mut self) -> Option<Self::Item> {
        let prev = self.prev.as_ref()?;
        let mut count = 1usize;
        let mut elem = self.iter.next();
        while elem.as_ref() == Some(prev) {
            count = count.checked_add(1).expect("counter overflowed");
            elem = self.iter.next();
        }
        Some(unsafe { (NonZeroUsize::new_unchecked(count), std::mem::replace(&mut self.prev, elem).unwrap_unchecked()) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<I, T> FusedIterator for RunLength<I, T>
where
    I: Iterator<Item = T>,
    T: Eq,
{}

impl<T: Iterator> IteratorExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_length_test() {
        let data = [3, 3, 1, 4, 4, 1, 1, 1, 5];
        let compressed = data.into_iter().run_length().map(|(c, v)| (c.get(), v)).collect::<Vec<_>>();
        assert_eq!(compressed[..], [(2, 3), (1, 1), (2, 4), (3, 1), (1, 5)]);

        let mut empty = std::iter::empty::<i32>().run_length();
        assert_eq!(empty.next(), None);
        assert_eq!(empty.next(), None);
        assert_eq!(empty.next(), None);

        let mut iter = std::iter::repeat(999).take(100).run_length();
        assert_eq!(iter.next(), Some((NonZeroUsize::new(100).unwrap(), 999)));
        assert_eq!(iter.next(), None);
    }
}
