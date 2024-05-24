pub struct PersistentArray<T> {
    data: Box<[T]>,
}

impl<T> PersistentArray<T> {
    pub fn new() -> Self {
        todo!()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        todo!()
    }

    pub fn set(&self, index: usize, value: T) -> Self {
        todo!()
    }
}
