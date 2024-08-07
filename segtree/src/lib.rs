pub mod operation;
use operation::Operator;
use std::iter;

pub struct Segtree<T, OP> {
    len: usize,
    data: Box<[T]>,
    op: OP,
}

impl<T, OP> Segtree<T, OP> {
    fn new_empty(op: OP) -> Self {
        Self {
            len: 0,
            data: Box::new([]),
            op,
        }
    }
}

impl<T, OP: Operator<Query = T>> Segtree<T, OP> {
    fn eval(mut self) -> Self {
        for i in (0..self.len - 1).rev() {
            self.data[i] = self.op.op(&self.data[i * 2], &self.data[i * 2 + 1]);
        }
        self
    }

    pub fn from_iter_op<I: IntoIterator<Item = T>>(iter: I, op: OP) -> Self {
        let iter = iter.into_iter();
        let (size_min, size_max) = iter.size_hint();
        if size_max == Some(0) {
            Self::new_empty(op)
        } else {
            assert_ne!(size_min, 0);
            let half_len_min = size_min.next_power_of_two();
            let half_len_max = size_max.map(usize::next_power_of_two);
            let uninit = if Some(half_len_min) == half_len_max {
                let half_len = half_len_min;
                let data = iter::repeat_with(|| OP::IDENT).take(half_len).chain(iter).collect();

                Self {
                    len: half_len,
                    data,
                    op,
                }
            } else {
                let data = iter.collect::<Vec<_>>();
                let half_len = data.len().next_power_of_two();
                let data = iter::repeat_with(|| OP::IDENT).take(half_len).chain(data).collect();
                Self {
                    len: half_len,
                    data,
                    op,
                }
            };
            uninit.eval()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
}