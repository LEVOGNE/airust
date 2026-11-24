//! Query-local IDF memoization keyed by live, nonzero interned-term addresses.
//! N is a power of two (8 or 256). A collision recomputes the weight; it never
//! changes a score. Zero marks an empty slot. Promotion retains array order.
#[derive(Clone, Copy, Default)]
struct QueryWeight {
    key: usize,
    idf: f64,
}

pub(super) struct QueryWeights<const N: usize = 256> {
    slots: [QueryWeight; N],
}

impl<const N: usize> Default for QueryWeights<N> {
    fn default() -> Self {
        Self {
            slots: [QueryWeight::default(); N],
        }
    }
}

impl<const N: usize> QueryWeights<N> {
    pub(super) fn transfer_into<const M: usize>(self, weights: &mut QueryWeights<M>) {
        for slot in self.slots {
            if slot.key != 0 {
                weights.get(slot.key, || slot.idf);
            }
        }
    }

    pub(super) fn get(&mut self, key: usize, compute: impl FnOnce() -> f64) -> f64 {
        let slot = &mut self.slots[((key >> 4) ^ (key >> 12)) & (N - 1)];
        if slot.key != key {
            slot.idf = compute();
            slot.key = key;
        }
        slot.idf
    }
}
