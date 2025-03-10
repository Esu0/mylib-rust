mod red_black_tree;
use red_black_tree::{Link, Node, SearchResult};
pub struct RedBlackTreeSet<T> {
    root: Option<Link<T>>,
}

impl<T: Ord> RedBlackTreeSet<T> {
    pub const fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, value: T) -> bool {
        if let Some(root) = self.root {
            let result = root.search(|x| value.cmp(x));
            match result {
                SearchResult::Found(_) => false,
                SearchResult::NotFound(node, dir) => {
                    let new_node = Node::new(value).into_link();
                    if let Some(new_root) = node.insert_leaf(dir, new_node) {
                        self.root = Some(new_root);
                    }
                    true
                }
            }
        } else {
            self.root = Some(Node::new(value).into_link());
            true
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        if let Some(root) = self.root {
            matches!(root.search(|x| value.cmp(x)), SearchResult::Found(_))
        } else {
            false
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<T>
    where
        T: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut root = self.root?;
        let result = root.search(|x| key.cmp(x.borrow()));
        match result {
            SearchResult::Found(node) => {
                let node = root.remove(node);
                if node == root {
                    self.root = None;
                } else {
                    self.root = Some(root);
                }
                Some(node.destroy().into_key())
            }
            SearchResult::NotFound(_, _) => None,
        }
    }

    #[cfg(test)]
    fn check(&self) {
        if let Some(root) = self.root {
            root.check()
        }
    }
}

impl<T: Ord> Default for RedBlackTreeSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_random() {
        use rand::prelude::SliceRandom;
        let mut set = RedBlackTreeSet::new();
        let mut rng = rand::rng();
        let mut perm = std::array::from_fn::<_, 20, _>(|i| i);
        perm.shuffle(&mut rng);

        // eprintln!("will insert: {:?}", &perm);
        for &pi in &perm {
            set.insert(pi);
            // eprintln!("{}", set.root.unwrap().dump());
        }
        assert!(perm.iter().all(|&pi| set.contains(&pi)));
    }

    #[test]
    fn test_insert_ascend() {
        let mut set = RedBlackTreeSet::new();
        for i in 0..20usize {
            assert!(set.insert(i));
        }
        for i in 0..20usize {
            assert!(set.contains(&i));
        }
    }

    #[test]
    fn test_remove_random() {
        use rand::prelude::SliceRandom;
        let mut set = RedBlackTreeSet::new();
        let mut rng = rand::rng();
        let mut perm = std::array::from_fn::<_, 50, _>(|i| i);
        perm.shuffle(&mut rng);

        for &pi in &perm {
            assert!(set.insert(pi));
            set.check();
        }

        perm.shuffle(&mut rng);

        for &pi in &perm {
            eprintln!("remove {}", pi);
            if set.remove(&pi) != Some(pi) {
                eprintln!("failed to remove {}", pi);
                eprintln!("tree:");
                if let Some(root) = set.root {
                    eprintln!("{}", root.dump());
                } else {
                    eprintln!("(empty)");
                }
            }
            set.check();
        }
    }
}
