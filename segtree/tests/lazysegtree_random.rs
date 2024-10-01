use rand::{rngs::ThreadRng, Rng};
use segtree::{lazy::LazySegtree, operation};

#[derive(Debug, Clone, Copy)]
enum Query<T> {
    OutputAll,
    UpdateRange(usize, usize, T),
    UpdateOne(usize, T),
}

fn update_range_test_solve_simple(data: &[i32], queries: &[Query<i32>]) -> Vec<Vec<i64>> {
    let mut res = vec![];
    let mut data = data.iter().map(|&x| x as i64).collect::<Vec<_>>();
    for &query in queries {
        match query {
            Query::OutputAll => {
                res.push(data.clone());
            }
            Query::UpdateOne(i, x) => {
                data[i] += x as i64;
            }
            Query::UpdateRange(l, r, x) => {
                data[l..r].iter_mut().for_each(|y| *y += x as i64);
            }
        }
    }
    res
}

fn update_range_test_solve1(data: &[i32], queries: &[Query<i32>]) -> Vec<Vec<i64>> {
    let n = data.len();
    let mut segtree = LazySegtree::from_iter_op(
        data.iter().map(|&x| x as i64),
        operation::min(),
        operation::range_add(),
    );
    let mut res = vec![];
    for &query in queries {
        match query {
            Query::OutputAll => {
                res.push(segtree.borrow_data()[..n].to_vec());
            }
            Query::UpdateOne(i, x) => {
                segtree.apply_range(i..=i, x as i64);
            }
            Query::UpdateRange(l, r, x) => {
                segtree.apply_range(l..r, x as i64);
            }
        }
    }
    res
}

fn update_range_test_once(rng: &mut ThreadRng) {
    let n = rng.gen_range(1..=500);
    let data = (0..n)
        .map(|_| rng.gen_range(-1_000_000_000..=1_000_000_000))
        .collect::<Vec<_>>();
    let q = rng.gen_range(1..=500);
    let queries = (0..q)
        .map(|_| {
            if rng.gen_bool(0.125) {
                Query::OutputAll
            } else if rng.gen_bool(0.5) {
                let l = rng.gen_range(0..n);
                let r = rng.gen_range(l + 1..=n);
                let x = rng.gen_range(-1_000_000_000..=1_000_000_000);
                Query::UpdateRange(l, r, x)
            } else {
                let i = rng.gen_range(0..n);
                let x = rng.gen_range(-1_000_000_000..=1_000_000_000);
                Query::UpdateOne(i, x)
            }
        })
        .collect::<Vec<_>>();
    let expected = update_range_test_solve_simple(&data, &queries);
    let result = update_range_test_solve1(&data, &queries);
    assert_eq!(expected, result);
}

#[test]
fn update_range_test() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        update_range_test_once(&mut rng);
    }
}
