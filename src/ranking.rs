use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub(crate) trait Ranked {
    fn score(&self) -> f64;
    fn id(&self) -> &str;
}

struct Item<T>(T);

impl<T: Ranked> PartialEq for Item<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<T: Ranked> Eq for Item<T> {}

impl<T: Ranked> PartialOrd for Item<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ranked> Ord for Item<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .0
            .score()
            .total_cmp(&self.0.score())
            .then_with(|| self.0.id().cmp(other.0.id()))
    }
}

pub(crate) struct TopK<T: Ranked> {
    limit: usize,
    heap: BinaryHeap<Item<T>>,
}

impl<T: Ranked> TopK<T> {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            heap: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        if self.limit == 0 {
            return;
        }
        let item = Item(value);
        if self.heap.len() < self.limit {
            self.heap.push(item);
        } else if let Some(mut worst) = self.heap.peek_mut() {
            if item < *worst {
                *worst = item;
            }
        }
    }

    pub fn finish(self) -> Vec<T> {
        self.heap
            .into_sorted_vec()
            .into_iter()
            .map(|item| item.0)
            .collect()
    }
}

impl Ranked for crate::search::SearchResult<'_> {
    fn score(&self) -> f64 {
        self.score
    }
    fn id(&self) -> &str {
        &self.entry.id
    }
}

impl Ranked for crate::matching::MatchResult<'_> {
    fn score(&self) -> f64 {
        self.score
    }
    fn id(&self) -> &str {
        &self.entry.id
    }
}

impl Ranked for crate::knowledge::RelationSuggestion {
    fn score(&self) -> f64 {
        self.score
    }
    fn id(&self) -> &str {
        &self.target_id
    }
}
