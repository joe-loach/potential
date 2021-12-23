use std::marker::PhantomData;

pub struct Index<T>(usize, PhantomData<T>);

impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}
impl<T> Copy for Index<T> {}

impl<T> PartialEq for Index<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub struct Store<T> {
    items: Vec<T>,
}

impl<T> Store<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) -> Index<T> {
        self.items.push(item);
        Index(self.items.len() - 1, PhantomData)
    }

    pub fn get(&self, index: &Index<T>) -> &T {
        // Indexes can only be created by the push method, so it must exist
        self.items.get(index.0).unwrap()
    }

    /// Must uphold that all indexes to the store are deleted
    pub unsafe fn clear(&mut self) {
        self.items.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }
}

impl<T> Default for Store<T> {
    fn default() -> Self {
        Self::new()
    }
}