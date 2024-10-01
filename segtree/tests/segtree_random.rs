use rand::{rngs::ThreadRng, Rng};
use segtree::{operation, Segtree};

#[derive(Debug, Clone, Copy)]
enum Query<T> {
    Sum(usize, usize),
    Update(usize, T),
}

fn range_add_test_solve1(data: &[i32], queries: &[Query<i32>]) -> Vec<i64> {
    let mut segtree = data
        .iter()
        .map(|&x| x as i64)
        .collect::<Segtree<_, operation::Add<_>>>();
    let mut res = vec![];
    for &query in queries {
        match query {
            Query::Sum(l, r) => {
                res.push(segtree.query(l..r));
            }
            Query::Update(i, x) => {
                segtree.update(i, x as i64);
            }
        }
    }
    res
}

fn range_add_test_solve_simple(data: &[i32], queries: &[Query<i32>]) -> Vec<i64> {
    let mut res = vec![];
    let mut data = data.to_vec();
    for &query in queries {
        match query {
            Query::Sum(l, r) => {
                res.push(data[l..r].iter().map(|&x| x as i64).sum());
            }
            Query::Update(i, x) => {
                data[i] = x;
            }
        }
    }
    res
}

fn range_add_test_once(rng: &mut ThreadRng) {
    let n = rng.gen_range(1..=500);
    let data = (0..n)
        .map(|_| rng.gen_range(-1_000_000_000..=1_000_000_000))
        .collect::<Vec<_>>();
    let q = rng.gen_range(1..=500);
    let queries = (0..q)
        .map(|_| {
            if rng.gen_bool(0.5) {
                let l = rng.gen_range(0..n);
                let r = rng.gen_range(l..=n);
                Query::Sum(l, r)
            } else {
                let i = rng.gen_range(0..n);
                let x = rng.gen_range(-1_000_000_000..=1_000_000_000);
                Query::Update(i, x)
            }
        })
        .collect::<Vec<_>>();
    let expected = range_add_test_solve_simple(&data, &queries);
    let result = range_add_test_solve1(&data, &queries);
    assert_eq!(expected, result);
}

#[test]
fn range_add_test() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        range_add_test_once(&mut rng);
    }
}
