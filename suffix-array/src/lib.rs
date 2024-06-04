use std::ops::Deref;

pub struct SuffixArray<'a, T> {
    array: Vec<&'a [T]>
}

impl<'a, T> Deref for SuffixArray<'a, T> {
    type Target = [&'a [T]];

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<'a, T> SuffixArray<'a, T> {
    pub fn new_simple(array: &'a [T]) -> Self
    where
        T: Ord,
    {
        let mut suffixes = (0..array.len()).map(|i| &array[i..]).collect::<Vec<_>>();
        suffixes.sort_unstable();
        Self {
            array: suffixes,
        }
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
