//! Forward-only joins over ordered document IDs. Callers must request increasing IDs.
//! The first range starts at the first candidate. After a gap, the original bounded
//! skip counts and pre-seek checkpoint preserve cooperative cancellation behavior.
use super::{shared_ref, DocumentMap, IndexedEntry, Shared};
use std::{collections::BTreeMap, sync::Arc};

pub(super) struct DocumentCursor<'a> {
    documents: &'a DocumentMap,
    rest: std::collections::btree_map::Range<'a, Arc<str>, Shared<IndexedEntry>>,
    next: Option<(&'a Arc<str>, &'a Shared<IndexedEntry>)>,
}

impl<'a> DocumentCursor<'a> {
    pub(super) fn new(documents: &'a DocumentMap, first: &str) -> Self {
        let mut rest = documents
            .range::<str, _>((std::ops::Bound::Included(first), std::ops::Bound::Unbounded));
        let next = rest.next();
        Self {
            documents,
            rest,
            next,
        }
    }

    pub(super) fn get_checked<E>(
        &mut self,
        id: &Arc<str>,
        checkpoint: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<(&'a Arc<str>, &'a IndexedEntry)>, E> {
        let mut skipped = 0;
        while let Some((key, document)) = self.next {
            let order = if Arc::ptr_eq(key, id) {
                std::cmp::Ordering::Equal
            } else {
                key.as_ref().cmp(id.as_ref())
            };
            match order {
                std::cmp::Ordering::Equal => {
                    self.next = self.rest.next();
                    return Ok(Some((key, shared_ref(document))));
                }
                std::cmp::Ordering::Greater => return Ok(None),
                std::cmp::Ordering::Less => {
                    skipped += 1;
                    if skipped == 8 {
                        checkpoint()?;
                        self.rest = self.documents.range::<str, _>((
                            std::ops::Bound::Included(id.as_ref()),
                            std::ops::Bound::Unbounded,
                        ));
                    }
                    self.next = self.rest.next();
                }
            }
        }
        Ok(None)
    }
}

pub(super) struct TermCursor<'a> {
    postings: &'a BTreeMap<Arc<str>, usize>,
    rest: std::collections::btree_map::Range<'a, Arc<str>, usize>,
    next: Option<(&'a Arc<str>, &'a usize)>,
}

impl<'a> TermCursor<'a> {
    #[cfg(test)]
    pub(super) fn next_id(&self) -> Option<&str> {
        self.next.map(|(id, _)| id.as_ref())
    }

    pub(super) fn new(postings: &'a BTreeMap<Arc<str>, usize>, first: &str) -> Self {
        let mut rest = postings
            .range::<str, _>((std::ops::Bound::Included(first), std::ops::Bound::Unbounded));
        let next = rest.next();
        Self {
            postings,
            rest,
            next,
        }
    }

    pub(super) fn frequency<E>(
        &mut self,
        id: &Arc<str>,
        checkpoint: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        let mut skipped = 0;
        while let Some((key, frequency)) = self.next {
            let order = if Arc::ptr_eq(key, id) {
                std::cmp::Ordering::Equal
            } else {
                key.as_ref().cmp(id.as_ref())
            };
            match order {
                std::cmp::Ordering::Equal => {
                    let frequency = *frequency;
                    self.next = self.rest.next();
                    return Ok(Some(frequency));
                }
                std::cmp::Ordering::Greater => return Ok(None),
                std::cmp::Ordering::Less => {
                    skipped += 1;
                    if skipped == 32 {
                        checkpoint()?;
                        self.rest = self.postings.range::<str, _>((
                            std::ops::Bound::Included(id.as_ref()),
                            std::ops::Bound::Unbounded,
                        ));
                    }
                    self.next = self.rest.next();
                }
            }
        }
        Ok(None)
    }
}
