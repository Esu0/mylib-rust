use super::Graph;

pub fn scc<G: Graph>(graph: &G, n: usize) -> Vec<usize> {
    let mut ids = vec![0; n];
    let mut ord = vec![usize::MAX; n];
    let mut low = vec![usize::MAX; n];
    let mut stack = Vec::new();
    let mut cur_id = 0;
    let mut cur_ord = 0;
    for u in 0..n {
        if ord[u] == usize::MAX {
            scc_dfs(graph, u, &mut cur_id, &mut ids, &mut cur_ord, &mut ord, &mut low, &mut stack);
        }
    }
    ids
}

fn scc_dfs<G: Graph>(
    graph: &G,
    u: usize,
    cur_id: &mut usize,
    ids: &mut [usize],
    cur_ord: &mut usize,
    ord: &mut [usize],
    low: &mut [usize],
    stack: &mut Vec<usize>,
) {
    let mut ctx = graph.init_context();
    stack.push(u);
    ord[u] = *cur_ord;
    low[u] = *cur_ord;
    *cur_ord += 1;
    while let Some(v) = graph.adjacent_vertices(&mut ctx, u) {
        if ord[v] == usize::MAX {
            scc_dfs(graph, v, cur_id, ids, cur_ord, ord, low, stack);
            low[u] = low[u].min(low[v]);
        } else {
            low[u] = low[u].min(ord[v]);
        }
    }
    if low[u] == ord[u] {
        while let Some(v) = stack.pop() {
            ids[v] = *cur_id;
            if v == u {
                break;
            }
        }
        *cur_id += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scc_test() {
        let mut g = super::super::AdjacencyList::new(10);
        g.add_edge(0, 1);
        g.add_edge(0, 2);
        g.add_edge(1, 4);
        g.add_edge(2, 0);
        g.add_edge(2, 6);
        g.add_edge(2, 8);
        g.add_edge(3, 1);
        g.add_edge(3, 5);
        g.add_edge(4, 3);
        g.add_edge(4, 7);
        g.add_edge(5, 3);
        g.add_edge(6, 7);
        g.add_edge(6, 8);
        g.add_edge(7, 3);
        g.add_edge(7, 9);
        g.add_edge(8, 6);
        g.add_edge(8, 7);

        let result = scc(&g, 10);

        eprintln!("{:?}", result);
        assert_eq!(result[0], result[2]);
        assert_eq!(result[1], result[3]);
        assert_eq!(result[1], result[4]);
        assert_eq!(result[1], result[5]);
        assert_eq!(result[1], result[7]);
        assert_eq!(result[6], result[8]);

        let list = [0, 1, 6, 9];
        for (i, &u) in list.iter().enumerate() {
            for &v in &list[i + 1..] {
                assert_ne!(result[u], result[v]);
            }
        }
    }
}
