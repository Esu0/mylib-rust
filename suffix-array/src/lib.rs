use std::{mem::{ManuallyDrop, MaybeUninit}, ops::Deref};

pub struct SuffixArray<'a, T> {
    array: Vec<&'a [T]>
}

impl<'a, T> Deref for SuffixArray<'a, T> {
    type Target = [&'a [T]];

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

fn vec_uninit<T>(n: usize) -> Vec<MaybeUninit<T>> {
    let mut v = Vec::with_capacity(n);
    unsafe {
        v.set_len(n);
    }
    v
}

impl<'a, T: Ord> SuffixArray<'a, T> {
    pub fn new_simple(array: &'a [T]) -> Self {
        let mut suffixes = (0..array.len()).map(|i| &array[i..]).collect::<Vec<_>>();
        suffixes.sort_unstable();
        Self {
            array: suffixes,
        }
    }

    fn make_bucket(arr: &'a [T]) -> (Vec<usize>, Vec<usize>) {
        debug_assert!(!arr.is_empty());
        let mut buf = (0..arr.len()).collect::<Vec<_>>();
        buf.sort_unstable_by(|&i, &j| arr[i].cmp(&arr[j]));
        let mut bucket = ManuallyDrop::new(vec_uninit(arr.len()));
        let mut i = 0;
        let mut prev_aj = &arr[buf[0]];
        bucket[buf[0]].write(i);
        for (k, &j) in buf.iter().enumerate().skip(1) {
            if &arr[j] != prev_aj {
                i = k;
            }
            bucket[j].write(i);
            prev_aj = &arr[j];
        }
        let ptr = bucket.as_mut_ptr();
        let len = bucket.len();
        let capacity = bucket.capacity();
        unsafe {
            (Vec::from_raw_parts(ptr as _, len, capacity), buf)
        }
    }

    pub fn new_sa_is(array: &'a [T]) -> Self {
        if array.is_empty() {
            return Self {
                array: Vec::new(),
            };
        }
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum SuffixType {
            S,
            L,
            Lms,
        }

        let n = array.len();
        let mut t = Vec::with_capacity(n);
        for (i, w) in array.windows(2).enumerate() {
            let [a, b] = w else {
                unsafe {
                    std::hint::unreachable_unchecked()
                }
            };
            use std::cmp::Ordering::*;
            match a.cmp(b) {
                Less => while {
                    t.push(SuffixType::S);
                    t.len() <= i
                } {},
                Equal => (),
                Greater => while {
                    t.push(SuffixType::L);
                    t.len() <= i
                } {},
            }
        }
        t.push(SuffixType::S);
        let mut prev = t[0];
        for ti in &mut t[1..n] {
            if let [SuffixType::L, SuffixType::S] = [prev, *ti] {
                *ti = SuffixType::Lms;
            }
            prev = *ti;
        }
        let (bucket, buf) = Self::make_bucket(array);
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_simple() {
        let array = b"banana";
        let suffix_array = SuffixArray::new_simple(array);
        assert_eq!(suffix_array.len(), 6);
        assert_eq!(suffix_array[0], b"a");
        assert_eq!(suffix_array[1], b"ana");
        assert_eq!(suffix_array[2], b"anana");
        assert_eq!(suffix_array[3], b"banana");
        assert_eq!(suffix_array[4], b"na");
        assert_eq!(suffix_array[5], b"nana");
    }
}
