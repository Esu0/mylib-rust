use std::ops::Deref;

pub struct ZArray {
    data: Vec<usize>,
}

impl Deref for ZArray {
    type Target = [usize];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl ZArray {
    pub fn new<T: Eq>(slice: &[T]) -> Self {
        let mut z = Vec::with_capacity(slice.len() - 1);
        let (mut l, mut r) = (0, 1);
        while z.len() < slice.len() - 1 {
            let i = z.len() + 1;
            if z.get(i - l - 1).is_some_and(|&x| x + i < r) {
                z.push(z[i - l - 1]);
            } else {
                l = i;
                r = i.max(r);
                while slice.get(r).is_some_and(|x| x == &slice[r - l]) {
                    r += 1;
                }
                z.push(r - l);
            }
        }
        Self {
            data: z,
        }
    }

    pub fn into_vec(self) -> Vec<usize> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn z_array_new() {
        let z = ZArray::new(b"ababaababaabababc");
        assert_eq!(&z.data, &[0, 3, 0, 1, 10, 0, 3, 0, 1, 5, 0, 4, 0, 2, 0, 0]);

        let z = ZArray::new(b"ooooooooo");
        assert_eq!(&z.data, &[8, 7, 6, 5, 4, 3, 2, 1]);

        let z = ZArray::new(b"x");
        assert_eq!(&z.data, &[]);
    }
}
