#[allow(dead_code)]
pub mod bimap {
    use std::{collections::HashMap, hash::Hash};

    pub struct BiMap<Left, Right> {
        left_to_right: HashMap<Left, Right>,
        right_to_left: HashMap<Right, Left>,
    }

    impl<Left, Right> BiMap<Left, Right>
    where
        Left: Eq + Hash + Clone,
        Right: Eq + Hash + Clone,
    {
        pub fn new() -> BiMap<Left, Right> {
            BiMap {
                left_to_right: HashMap::new(),
                right_to_left: HashMap::new(),
            }
        }

        pub fn insert(&mut self, left: Left, right: Right) {
            self.left_to_right.insert(left.clone(), right.clone());
            self.right_to_left.insert(right.clone(), left.clone());
        }

        pub fn get_with_left(&self, left: &Left) -> Option<&Right> {
            self.left_to_right.get(left)
        }

        pub fn get_with_right(&self, right: &Right) -> Option<&Left> {
            self.right_to_left.get(right)
        }
    }
}

#[allow(dead_code)]
pub mod pairmap {
    use std::collections::{HashMap, HashSet};
    use std::hash::Hash;

    pub struct PairMap<Left, Right> {
        left_to_right: HashMap<Left, HashSet<Right>>,
        right_to_left: HashMap<Right, HashSet<Left>>,
    }

    impl<Left, Right> PairMap<Left, Right>
    where
        Left: Eq + Hash + Clone,
        Right: Eq + Hash + Clone,
    {
        pub fn new() -> PairMap<Left, Right> {
            PairMap {
                left_to_right: HashMap::new(),
                right_to_left: HashMap::new(),
            }
        }

        pub fn insert(&mut self, left: Left, right: Right) {
            self.left_to_right
                .entry(left.clone())
                .or_insert(HashSet::new())
                .insert(right.clone());
            self.right_to_left
                .entry(right.clone())
                .or_insert(HashSet::new())
                .insert(left.clone());
        }

        pub fn get_with_left<'a>(&'a self, left: &Left) -> impl Iterator<Item = &'a Right> {
            self.left_to_right.get(left).into_iter().flatten()
        }

        pub fn get_with_right<'a>(&'a self, right: &Right) -> impl Iterator<Item = &'a Left> {
            self.right_to_left.get(right).into_iter().flatten()
        }

        pub fn delete_with_left(&mut self, left: &Left) {
            for right in self.left_to_right.get(left).into_iter().flatten() {
                self.right_to_left.get_mut(right).map(|ls| ls.remove(left));
            }
            self.left_to_right.remove(left);
        }

        pub fn delete_with_right(&mut self, right: &Right) {
            for left in self.right_to_left.get(right).into_iter().flatten() {
                self.left_to_right.get_mut(left).map(|rs| rs.remove(right));
            }
            self.right_to_left.remove(right);
        }
    }
}

pub mod fastqueue {
    use std::collections::{HashSet, VecDeque};

    pub struct FastQueue<T> {
        queue: VecDeque<T>,
        set: HashSet<T>,
    }

    impl<T: Eq + std::hash::Hash + Clone> FastQueue<T> {
        pub fn new() -> Self {
            FastQueue {
                queue: VecDeque::new(),
                set: HashSet::new(),
            }
        }

        pub fn push(&mut self, item: T) {
            if !self.set.contains(&item) {
                self.queue.push_back(item.clone());
                self.set.insert(item);
            }
        }

        pub fn pop(&mut self) -> Option<T> {
            self.queue.pop_front().map(|item| {
                self.set.remove(&item);
                item
            })
        }
    }
}
