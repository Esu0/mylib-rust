pub mod scc;

pub trait Graph {
    type Context;
    fn init_context(&self) -> Self::Context;
    fn adjacent_vertices(&self, ctx: &mut Self::Context, u: usize) -> Option<usize>;
}

pub trait WeightedGraph: Graph {
    type W;

    fn adjacent_vertices_with_weight(
        &self,
        ctx: &mut Self::Context,
        u: usize,
    ) -> Option<(usize, &Self::W)>;
}

impl Graph for AdjacencyList {
    type Context = usize;

    fn init_context(&self) -> Self::Context {
        0
    }

    fn adjacent_vertices(&self, ctx: &mut Self::Context, u: usize) -> Option<usize> {
        self.adj[u].get(*ctx).map(|v| {
            *ctx += 1;
            *v
        })
    }
}

impl<W> Graph for AdjacencyListWeighted<W> {
    type Context = usize;

    fn init_context(&self) -> Self::Context {
        0
    }

    fn adjacent_vertices(&self, ctx: &mut Self::Context, u: usize) -> Option<usize> {
        self.adj[u].get(*ctx).map(|(v, _)| {
            *ctx += 1;
            *v
        })
    }
}

impl<W> WeightedGraph for AdjacencyListWeighted<W> {
    type W = W;

    fn adjacent_vertices_with_weight(
        &self,
        ctx: &mut Self::Context,
        u: usize,
    ) -> Option<(usize, &Self::W)> {
        self.adj[u].get(*ctx).map(|(v, w)| {
            *ctx += 1;
            (*v, w)
        })
    }
}

pub struct AdjacencyList {
    adj: Box<[Vec<usize>]>,
}

pub struct Edges<'a> {
    g: &'a AdjacencyList,
    u: usize,
    i: usize,
}

impl AdjacencyList {
    pub fn new(n: usize) -> Self {
        Self {
            adj: std::iter::repeat_with(Vec::new).take(n).collect(),
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        self.adj[u].push(v);
    }

    pub fn add_edge_undirected(&mut self, u: usize, v: usize) {
        if u == v {
            self.add_edge(u, u);
        } else {
            self.add_edge(u, v);
            self.add_edge(v, u);
        }
    }

    pub fn nvertices(&self) -> usize {
        self.adj.len()
    }

    pub fn edges(&self) -> Edges {
        Edges { g: self, u: 0, i: 0 }
    }
}

impl Iterator for Edges<'_> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while self.u < self.g.adj.len() {
            if self.i < self.g.adj[self.u].len() {
                let v = self.g.adj[self.u][self.i];
                self.i += 1;
                return Some((self.u, v));
            }
            self.u += 1;
        }
        None
    }
}

pub struct AdjacencyListWeighted<W> {
    adj: Box<[Vec<(usize, W)>]>,
}

pub struct WeightedEdges<'a, W> {
    g: &'a AdjacencyListWeighted<W>,
    u: usize,
    i: usize,
}

impl<'a, W> Iterator for WeightedEdges<'a, W> {
    type Item = (usize, usize, &'a W);

    fn next(&mut self) -> Option<Self::Item> {
        while self.u < self.g.adj.len() {
            if self.i < self.g.adj[self.u].len() {
                let (v, w) = &self.g.adj[self.u][self.i];
                self.i += 1;
                return Some((self.u, *v, w));
            }
            self.u += 1;
        }
        None
    }
}

impl<W> AdjacencyListWeighted<W> {
    pub fn new(n: usize) -> Self {
        Self {
            adj: std::iter::repeat_with(Vec::new).take(n).collect(),
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize, weight: W) {
        self.adj[u].push((v, weight));
    }

    pub fn add_edge_undirected(&mut self, u: usize, v: usize, weight: W)
    where
        W: Clone,
    {
        if u == v {
            self.add_edge(u, u, weight);
        } else {
            self.add_edge(u, v, weight.clone());
            self.add_edge(v, u, weight);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bfs_test() {
        let mut g = AdjacencyList::new(10);
        g.add_edge(0, 1);
        g.add_edge(0, 2);
        g.add_edge(1, 3);
        g.add_edge(2, 4);
        g.add_edge(3, 5);
        g.add_edge(4, 5);
        g.add_edge(5, 6);
        g.add_edge(6, 7);
        g.add_edge(6, 8);
        g.add_edge(7, 9);
        g.add_edge(8, 9);

        let mut queue = std::collections::VecDeque::from([0]);
        let mut dist = [usize::MAX; 10];
        dist[0] = 0;

        while let Some(u) = queue.pop_front() {
            let mut ctx = g.init_context();
            let d = dist[u];
            while let Some(v) = g.adjacent_vertices(&mut ctx, u) {
                if dist[v] == usize::MAX {
                    dist[v] = d + 1;
                    queue.push_back(v);
                }
            }
        }

        assert_eq!(dist, [0, 1, 1, 2, 2, 3, 4, 5, 5, 6]);
    }
}
