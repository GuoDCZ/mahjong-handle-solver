use std::cmp::Ord;
use std::collections::{BinaryHeap, HashSet};
use std::hash::Hash;

struct UniqueHeap<T>
where
    T: Ord + Hash + Clone,
{
    heap: BinaryHeap<T>,
    set: HashSet<T>,
}

impl<T> UniqueHeap<T>
where
    T: Ord + Hash + Clone,
{
    fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            set: HashSet::new(),
        }
    }

    fn push(&mut self, item: T) {
        if self.set.contains(&item) {
            return;
        }
        self.set.insert(item.clone());
        self.heap.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        // self.set.remove(&item);
        self.heap.pop()
    }
}

pub trait Growable {
    fn grow(&self) -> impl Iterator<Item = Self>;
}

pub struct GrowingHeap<T>
where
    T: Ord + Hash + Clone + Growable,
{
    uh: UniqueHeap<T>,
}

impl<T> GrowingHeap<T>
where
    T: Ord + Hash + Clone + Growable,
{
    pub fn new(seed: T) -> Self {
        let mut uh = UniqueHeap::new();
        uh.push(seed);
        Self { uh }
    }
}

impl<T> Iterator for GrowingHeap<T>
where
    T: Ord + Hash + Clone + Growable,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.uh.pop()?;
        for new_item in item.grow() {
            self.uh.push(new_item);
        }
        Some(item)
    }
}
