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

fn vec_uninit_arr<T, const N: usize>(n: usize) -> Vec<[MaybeUninit<T>; N]> {
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

    fn make_bucket(arr: &'a [T]) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        debug_assert!(!arr.is_empty());
        let n = arr.len();
        let mut buf = (0..n).collect::<Vec<_>>();
        buf.sort_unstable_by(|&i, &j| arr[i].cmp(&arr[j]));
        let mut bucket = ManuallyDrop::new(vec_uninit(n));
        let mut bucket_range = Vec::with_capacity((n as f32).sqrt() as usize);
        let mut i = 0;
        bucket_range.push(0);
        let mut prev_aj = &arr[buf[0]];
        bucket[buf[0]].write(0);
        for (k, &j) in buf.iter().enumerate().skip(1) {
            if &arr[j] != prev_aj {
                i = k;
                bucket_range.push(i);
            }
            bucket[j].write(bucket_range.len() - 1);
            prev_aj = &arr[j];
        }
        bucket_range.push(n);

        let ptr = bucket.as_mut_ptr();
        let len = bucket.len();
        let capacity = bucket.capacity();
        unsafe {
            (Vec::from_raw_parts(ptr as _, len, capacity), bucket_range, buf)
        }
    }

    pub fn new_sa_is(array: &'a [T]) -> Self {
        if array.is_empty() {
            return Self {
                array: Vec::new(),
            };
        }
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
        eprintln!("{:?}", t);

        let (bucket, bucket_range, mut sa) = Self::make_bucket(array);
        sa.fill(usize::MAX);
        let mut bucket_head = bucket_range.clone();
        for (i, &ti) in t.iter().enumerate() {
            if let SuffixType::Lms = ti {
                sa[bucket_head[bucket[i]]] = i;
                bucket_head[bucket[i]] += 1;
            }
        }

        eprintln!("{:?}", sa);

        let mut bucket_tail = bucket_head;
        bucket_tail.clone_from_slice(&bucket_range);
        let bucket_tail_ref = &mut bucket_tail[1..];
        eprintln!("{:?}", bucket_tail_ref);
        for i in 0..n {
            let j = sa[i];
            if j >= n {
                continue;
            }
            if let SuffixType::Lms = t[j] {
                sa[i] = usize::MAX;
            }
            let k = j.wrapping_sub(1);
            if t.get(k).is_some_and(|&ti| ti == SuffixType::L) {
                bucket_tail_ref[bucket[k]] -= 1;
                sa[bucket_tail_ref[bucket[k]]] = k;
            }
        }
        eprintln!("{:?}", sa);

        let mut bucket_head = bucket_tail;
        let mut lms_substr_prev = usize::MAX;
        let mut lms_substr_rank = 0usize;
        let mut lms_substr = Vec::with_capacity(n / 2);
        bucket_head.clone_from_slice(&bucket_range);
        for i in (0..n).rev() {
            let j = sa[i];
            let k = j.wrapping_sub(1);
            if k < n {
                match t[k] {
                    SuffixType::S => {
                        sa[bucket_head[bucket[k]]] = k;
                        bucket_head[bucket[k]] += 1;
                    }
                    SuffixType::Lms => {
                        sa[bucket_head[bucket[k]]] = k;
                        bucket_head[bucket[k]] += 1;

                        if lms_substr_prev < n {
                            let mut i = k;
                            let mut j = lms_substr_prev;

                            while array[i] == array[j] {
                                i += 1;
                                j += 1;
                                match (t[i], t[j]) {
                                    (SuffixType::Lms, SuffixType::Lms) => {
                                        if array[i] == array[j] {
                                            lms_substr_rank -= 1;
                                        }
                                        break;
                                    }
                                    (SuffixType::Lms, _) | (_, SuffixType::Lms) => {
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        lms_substr.push(lms_substr_rank);
                        lms_substr_rank += 1;
                        lms_substr_prev = k;
                    }
                    _ => {}
                }
            }
        }
        sa[bucket_head[bucket[n - 1]]] = n - 1;
        eprintln!("{:?}", lms_substr);
        lms_substr.push(lms_substr_rank);
        eprintln!("{:?}", sa);
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

    #[test]
    fn test_make_bucket() {
        let arr = b"banana";
        let (bucket, bucket_range, buf) = SuffixArray::make_bucket(arr);

        assert_eq!(&bucket_range, &[0, 3, 4, 6]);
        assert_eq!(&bucket, &[1, 0, 2, 0, 2, 0]);
        assert_eq!(&buf.iter().map(|&i| arr[i]).collect::<Vec<_>>(), b"aaabnn");
    }

    #[test]
    fn new_sa_is_tmp() {
        let _ = SuffixArray::new_sa_is(b"abaababababaaba");
    }
}
