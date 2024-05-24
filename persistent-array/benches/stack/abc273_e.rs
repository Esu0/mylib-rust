use std::collections::HashMap;

use persistent_array::stack::PersistentStackPool;
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub enum Query {
    Add(u32),
    Delete,
    Save(u32),
    Load(u32),
}

pub trait Solver {
    fn solve(&self, queries: &[Query], output: &mut Vec<i32>);
}

pub struct WithPersistentStack;

impl Solver for WithPersistentStack {
    fn solve(&self, queries: &[Query], output: &mut Vec<i32>) {
        let pool = PersistentStackPool::new(queries.len());
        let mut stack = pool.get_empty_stack();
        let mut note = HashMap::new();
        for &query in queries {
            match query {
                Query::Add(x) => {
                    stack = stack.push(x);
                }
                Query::Delete => {
                    stack = stack.pop();
                }
                Query::Save(x) => {
                    note.insert(x, stack);
                }
                Query::Load(x) => {
                    stack = note[&x];
                }
            }
            output.push(stack.top().copied().map_or(-1, |x| x as i32));
        }
    }
}

pub fn benchmark_case() -> Vec<Query> {
    let q = 500_000usize;
    let mut rng = rand::thread_rng();
    let mut saved = Vec::with_capacity(q / 4);
    (0..q)
        .map(|_| {
            match if saved.is_empty() {
                rng.gen_range(0..3)
            } else {
                rng.gen_range(0..4)
            } {
                0 => Query::Add(rng.gen_range(1..=1_000_000_000)),
                1 => Query::Delete,
                2 => {
                    let y = rng.gen_range(1..=1_000_000_000);
                    saved.push(y);
                    Query::Save(y)
                }
                3 => {
                    let y = saved[rng.gen_range(0..saved.len())];
                    Query::Load(y)
                }
                _ => unreachable!(),
            }
        })
        .collect()
}
