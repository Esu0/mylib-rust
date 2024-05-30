use node::Root;

pub struct BTree<K, V> {
    root: Option<Root<K, V>>,
}

impl<K, V> BTree<K, V> {
    pub const fn new() -> Self {
        Self { root: None }
    }
}
mod node;
