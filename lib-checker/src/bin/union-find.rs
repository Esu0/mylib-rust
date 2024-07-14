#[allow(dead_code)]
mod union_find {
    use std::ops::{Add, Sub};

    pub struct UnionFind<T> {
        uf: Vec<usize>,
        size: Vec<usize>,
        query: Vec<T>,
    }

    pub trait Query {
        fn query(&self, other: &Self) -> Self;
    }

    pub trait RevQuery: Query {
        /// other.query(output) == self
        fn rev_query(&self, other: &Self) -> Self;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SumQuery<T>(pub T);

    impl<T: Add<Output = T> + Clone> Query for SumQuery<T> {
        fn query(&self, other: &Self) -> Self {
            Self(self.0.clone() + other.0.clone())
        }
    }

    impl<T: Add<Output = T> + Sub<Output = T> + Clone> RevQuery for SumQuery<T> {
        fn rev_query(&self, other: &Self) -> Self {
            Self(self.0.clone() - other.0.clone())
        }
    }

    impl<Q1: Query, Q2: Query> Query for (Q1, Q2) {
        fn query(&self, other: &Self) -> Self {
            (self.0.query(&other.0), self.1.query(&other.1))
        }
    }

    impl<Q1: RevQuery, Q2: RevQuery> RevQuery for (Q1, Q2) {
        fn rev_query(&self, other: &Self) -> Self {
            (self.0.rev_query(&other.0), self.1.rev_query(&other.1))
        }
    }

    impl Query for () {
        fn query(&self, _other: &Self) -> Self {}
    }

    impl<T> UnionFind<T> {
        pub fn new(data: Vec<T>) -> Self {
            let size = data.len();
            Self {
                uf: (0..size).collect(),
                size: vec![1; size],
                query: data,
            }
        }

        pub fn len(&self) -> usize {
            self.uf.len()
        }

        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }

    impl<T: Query> UnionFind<T> {
        pub fn unite(&mut self, i: usize, j: usize) {
            let root_i = self.find(i);
            let root_j = self.find(j);
            if root_i != root_j {
                let size_i = self.size[root_i];
                let size_j = self.size[root_j];
                if size_i > size_j {
                    self.uf[root_j] = root_i;
                    self.size[root_i] = size_i + size_j;
                    let new_data = self.query[root_i].query(&self.query[root_j]);
                    self.query[root_i] = new_data;
                } else {
                    self.uf[root_i] = root_j;
                    self.size[root_j] = size_i + size_j;
                    let new_data = self.query[root_j].query(&self.query[root_i]);
                    self.query[root_j] = new_data;
                }
            }
        }

        pub fn find(&self, i: usize) -> usize {
            self.find_query(i).0
        }

        pub fn query(&self, i: usize) -> &T {
            self.find_query(i).1
        }

        pub fn find_query(&self, i: usize) -> (usize, &T) {
            let parent = self.uf[i];
            if parent == i {
                (i, &self.query[i])
            } else {
                self.find_query(parent)
            }
        }
    }
}

macro_rules! scan_from {
    ($tokens:expr, $t:ty $(,)?) => {
        $tokens.next().unwrap().parse::<$t>().unwrap()
    };
    ($tokens:expr, $($t:ty),+ $(,)?) => {
        ($(scan_from!($tokens, $t)),+)
    };
}

fn main() {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap();
    let mut tokens = buf.split_whitespace();
    let (n, q) = scan_from!(tokens, usize, usize);

    let mut uf = union_find::UnionFind::new(vec![(); n]);
    for _ in 0..q {
        let (t, u, v) = scan_from!(tokens, u8, usize, usize);
        match t {
            0 => uf.unite(u, v),
            1 => {
                if uf.find(u) == uf.find(v) {
                    println!("1");
                } else {
                    println!("0");
                }
            }
            _ => unreachable!()
        }
    }
}
