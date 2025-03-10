use std::ptr::NonNull;
use Color::*;
use Direction::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Left => Right,
            Right => Left,
        }
    }
}

pub struct Node<K> {
    key: K,
    color: Color,
    left: Option<Link<K>>,
    right: Option<Link<K>>,
    parent: Option<(Link<K>, Direction)>,
}

pub struct Link<K>(pub NonNull<Node<K>>);

impl<K> Clone for Link<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K> Copy for Link<K> {}

impl<K> PartialEq for Link<K> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K> Eq for Link<K> {}

impl<K> Node<K> {
    pub fn new(key: K) -> Self {
        Self {
            key,
            color: Red,
            left: None,
            right: None,
            parent: None,
        }
    }

    pub fn into_link(self) -> Link<K> {
        Link(NonNull::from(Box::leak(Box::new(self))))
    }

    pub fn child(&self, dir: Direction) -> Option<Link<K>> {
        match dir {
            Left => self.left,
            Right => self.right,
        }
    }

    pub fn set_child(&mut self, dir: Direction, child: Option<Link<K>>) {
        match dir {
            Left => self.left = child,
            Right => self.right = child,
        }
    }

    pub fn link(&mut self, dir: Direction, child: &mut Self) {
        self.set_child(dir, Some(Link::from_mut(child)));
        child.parent = Some((Link::from_mut(self), dir));
    }

    pub fn link_child(&mut self, dir: Direction, child: Option<Link<K>>) {
        self.set_child(dir, child);
        if let Some(child) = child {
            child.set_parent(Some((Link::from_mut(self), dir)));
        }
    }

    pub fn rot(&mut self, child: &mut Self, dir: Direction) {
        let gc = child.child(dir);
        child.link(dir, self);
        self.link_child(dir.opposite(), gc);
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut K, Option<&K>, Option<&K>),
    {
        let lk = self.left.map(|n| unsafe { &(*n.0.as_ptr()).key });
        let rk = self.right.map(|n| unsafe { &(*n.0.as_ptr()).key });
        f(&mut self.key, lk, rk);
    }

    pub fn rot_with_update<F>(&mut self, child: &mut Self, dir: Direction, mut f: F)
    where
        F: FnMut(&mut K, Option<&K>, Option<&K>),
    {
        self.rot(child, dir);
        self.update(&mut f);
        child.update(&mut f);
    }

    pub fn swap_key(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.key, &mut other.key);
    }

    pub fn into_key(self) -> K {
        self.key
    }
}

pub enum SearchResult<K> {
    Found(Link<K>),
    NotFound(Link<K>, Direction),
}

impl<K> Link<K> {
    pub fn from_mut(ptr: &mut Node<K>) -> Self {
        Link(NonNull::from(ptr))
    }

    pub fn search<F>(self, mut f: F) -> SearchResult<K>
    where
        F: FnMut(&K) -> std::cmp::Ordering,
    {
        use std::cmp::Ordering::*;
        let mut cur = self;
        loop {
            let node = unsafe { cur.0.as_ref() };
            match f(&node.key) {
                Less => match node.left {
                    Some(left) => cur = left,
                    None => return SearchResult::NotFound(cur, Left),
                },
                Greater => match node.right {
                    Some(right) => cur = right,
                    None => return SearchResult::NotFound(cur, Right),
                },
                Equal => return SearchResult::Found(cur),
            }
        }
    }

    pub fn as_ptr(self) -> *mut Node<K> {
        self.0.as_ptr()
    }

    fn as_mut<'a>(self) -> &'a mut Node<K> {
        unsafe { &mut *self.as_ptr() }
    }

    fn as_ref<'a>(self) -> &'a Node<K> {
        unsafe { &*self.as_ptr() }
    }

    pub fn child(self, dir: Direction) -> Option<Self> {
        let node = self.as_mut();
        match dir {
            Left => node.left,
            Right => node.right,
        }
    }

    pub fn set_child(self, dir: Direction, child: Option<Self>) {
        let node = self.as_mut();
        match dir {
            Left => node.left = child,
            Right => node.right = child,
        }
    }

    pub fn parent(self) -> Option<(Self, Direction)> {
        self.as_ref().parent
    }

    pub fn set_parent(self, parent: Option<(Self, Direction)>) {
        self.as_mut().parent = parent;
    }

    pub fn link(self, dir: Direction, parent: Option<Self>) {
        let node = self.as_mut();
        if let Some(parent) = parent {
            unsafe {
                match dir {
                    Left => (*parent.as_ptr()).left = Some(self),
                    Right => (*parent.as_ptr()).right = Some(self),
                }
            }
            node.parent = Some((parent, dir));
        } else {
            node.parent = None;
        }
    }

    pub fn link_child(self, dir: Direction, child: Option<Self>) {
        let node = self.as_mut();
        match dir {
            Left => node.left = child,
            Right => node.right = child,
        };
        if let Some(child) = child {
            child.as_mut().parent = Some((self, dir));
        }
    }

    pub fn rot(self, child: Self, dir: Direction) {
        self.link_child(dir.opposite(), child.child(dir));
        child.link_child(dir, Some(self));
    }

    pub fn insert_leaf(self, dir: Direction, node: Self) -> Option<Self> {
        debug_assert!(self.child(dir).is_none());
        debug_assert_eq!(node.as_ref().color, Red);

        self.link_child(dir, Some(node));
        self.rebalance_red(dir, node)
    }

    pub fn remove(&mut self, node: Self) -> Link<K> {
        let mut node = node.as_mut();
        if let Some(leftlink) = node.left {
            if let Some(right) = node.right {
                let mut right = right.as_mut();
                while let Some(child) = right.left {
                    right = child.as_mut();
                }
                node.swap_key(right);
                node = right;
            } else {
                let left = leftlink.as_mut();
                node.swap_key(left);
                node.left = None;
                return leftlink;
            }
        }
        if let Some(rightlink) = node.right {
            let right = rightlink.as_mut();
            node.swap_key(right);
            node.right = None;
            return rightlink;
        }
        if let Some((parent, dir)) = node.parent {
            if node.color == Black {
                self.remove_black_leaf(parent, dir);
            } else {
                parent.set_child(dir, None);
            }
        }
        Link::from_mut(node)
    }

    pub fn destroy(self) -> Node<K> {
        unsafe { *Box::from_raw(self.0.as_ptr()) }
    }

    pub fn remove_black_leaf(&mut self, parent: Self, dir: Direction) {
        parent.set_child(dir, None);
        if let Some(new_root) = parent.rebalance_black(dir) {
            *self = new_root;
        }
    }

    fn color(this: Option<Self>) -> Color {
        this.map_or(Black, |n| n.as_ref().color)
    }

    pub fn rebalance_black(self, dir: Direction) -> Option<Self> {
        let mut parent = self.as_mut();
        let mut dir = dir;
        let mut ret = None;
        loop {
            //           ? <- parent
            //          / \
            // this -> B   ? <- sibling
            //            / \
            // sibch0 -> ?   ? <- sibch1

            // debug_assert_eq!(Self::color(this), Black);
            let mut sibling = unsafe { parent.child(dir.opposite()).unwrap_unchecked().as_mut() };
            if sibling.color == Red {
                debug_assert_eq!(parent.color, Black);
                //   B             R                B
                //  / \   rotate  / \  recoloring  / \
                // B   R    ->   B   B     ->     R   B
                //    / \       / \              / \
                //   B   B     B   B            B   B
                let gp = parent.parent;
                if let Some(gp) = gp {
                    gp.0.set_child(gp.1, Some(Link::from_mut(sibling)));
                } else {
                    ret = Some(Link::from_mut(sibling));
                }
                parent.rot(sibling, dir);
                sibling.parent = gp;
                sibling.color = Black;
                parent.color = Red;
                sibling = unsafe { parent.child(dir.opposite()).unwrap_unchecked().as_mut() };
            }

            //   B        ?        ?        R
            //  / \      / \      / \      / \
            // B   B    B   B    B   B    B   B
            //    / \      / \      / \      / \
            //   B   B    R   B    B   R    B   B

            debug_assert_eq!(sibling.color, Black);
            let mut sibch1 = sibling
                .child(dir.opposite())
                .map(|n| unsafe { &mut *n.as_ptr() });
            if sibch1.as_ref().map_or(true, |n| n.color == Black) {
                let sibch0 = sibling.child(dir).map(Link::as_mut);
                match sibch0 {
                    Some(sibch0) if sibch0.color == Red => {
                        //   ?          ?             ?
                        //  / \        / \           / \
                        // B   B  ->  B   R    ->   B   B
                        //    / \        / \           / \
                        //   R   B      B   B         B   R
                        //  / \            / \           / \
                        // B   B          B   B         B   B
                        sibling.rot(sibch0, dir.opposite());
                        sibling.color = Red;
                        sibch0.color = Black;
                        sibch1 = Some(sibling);
                        sibling = sibch0;
                    }
                    _ => {
                        if parent.color == Black {
                            //   B          B
                            //  / \        / \
                            // B   B  ->  B   R
                            //    / \        / \
                            //   B   B      B   B
                            sibling.color = Red;
                            if let Some(new_parent) = parent.parent {
                                parent = new_parent.0.as_mut();
                                dir = new_parent.1;
                                continue;
                            } else {
                                return None;
                            }
                        } else {
                            //   R          B
                            //  / \        / \
                            // B   B  ->  B   R
                            //    / \        / \
                            //   B   B      B   B
                            parent.color = Black;
                            sibling.color = Red;
                            return ret;
                        }
                    }
                }
            }
            let sibch1 = unsafe { sibch1.unwrap_unchecked() };
            debug_assert_eq!(sibch1.color, Red);
            //   ?          B          ?
            //  / \        / \        / \
            // B   B  ->  ?   R  ->  B   B
            //    / \    / \        / \
            //   B   R  B   B      B   B
            let gp = parent.parent;
            if let Some(gp) = gp {
                gp.0.set_child(gp.1, Some(Link::from_mut(sibling)));
            } else {
                ret = Some(Link::from_mut(sibling));
            }
            parent.rot(sibling, dir);
            sibling.parent = gp;
            sibling.color = parent.color;
            parent.color = Black;
            sibch1.color = Black;
            return ret;
        }
    }

    pub fn rebalance_red(self, dir: Direction, child: Self) -> Option<Self> {
        let (mut this, mut child) = (self.as_mut(), child.as_mut());
        let mut dir = dir;
        loop {
            if this.color == Black {
                return None;
            }

            if let Some(p) = this.parent {
                let parent = p.0.as_mut();
                if let Some(sib) = parent.child(p.1.opposite()) {
                    let sibling = sib.as_mut();
                    if sibling.color == Red {
                        parent.color = Red;
                        sibling.color = Black;
                        this.color = Black;
                        if let Some(gp) = parent.parent {
                            child = parent;
                            this = gp.0.as_mut();
                            dir = gp.1;
                            continue;
                        } else {
                            return None;
                        }
                    }
                }
                if dir != p.1 {
                    dir = dir.opposite();
                    this.rot(child, dir);
                    std::mem::swap(&mut this, &mut child);
                }
                let gp = parent.parent;
                parent.rot(this, p.1.opposite());
                this.color = Black;
                parent.color = Red;
                this.parent = gp;
                if let Some(gp) = gp {
                    gp.0.set_child(gp.1, Some(Link::from_mut(this)));
                    return None;
                } else {
                    return Some(Link::from_mut(this));
                }
            }
            this.color = Black;
            return None;
        }
    }

    #[cfg(test)]
    pub fn dump(self) -> tests::Dump<K> {
        tests::Dump(self)
    }

    #[cfg(test)]
    pub fn check(self) {
        self.check_rec(None);
    }

    #[cfg(test)]
    pub fn check_rec(self, parent: Option<(Self, Direction)>) -> usize {
        // let node = unsafe { self.0.as_ref() };
        if parent != self.parent() {
            panic!("parent mismatch");
        }
        let mut h = 0;
        if let Some(left) = self.child(Left) {
            h = left.check_rec(Some((self, Left)));
        }
        if let Some(right) = self.child(Right) {
            if h != right.check_rec(Some((self, Right))) {
                panic!("red-black-tree condition violated");
            }
        }

        if Self::color(Some(self)) == Black {
            h += 1;
        }
        h
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    pub struct Dump<K>(pub Link<K>);

    impl<K: std::fmt::Display> Dump<K> {
        pub fn fmt_with_indent(
            &self,
            f: &mut std::fmt::Formatter<'_>,
            indent: usize,
        ) -> std::fmt::Result {
            if indent > 20 {
                panic!();
            }
            let node = unsafe { self.0 .0.as_ref() };
            if let Some(right) = node.right {
                Dump(right).fmt_with_indent(f, indent + 1)?;
            }
            let col = match node.color {
                Red => 'R',
                Black => 'B',
            };
            writeln!(f, "{:3$}{} {}", "", col, node.key, indent * 4)?;
            if let Some(left) = node.left {
                Dump(left).fmt_with_indent(f, indent + 1)?;
            }
            Ok(())
        }
    }
    impl<K: std::fmt::Display> std::fmt::Display for Dump<K> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.fmt_with_indent(f, 0)
        }
    }
}
