//! Stable sorting with one copy of the sorting code.
//!
//! `slice::sort_by` compiles a full copy of the sort for every closure it is given, which made
//! the many small sorts of the layouts the biggest block of the WebAssembly build after the
//! parsers. Here the order is found by one sort of indices that does not depend on the element
//! type, and the elements are then moved into it. Equal elements keep their order, as with
//! `sort_by`.

use std::cmp::Ordering;

/// The indices `0..count` in the order that `compare` puts them in, equal ones in their own
/// order.
fn ranked(count: usize, compare: &mut dyn FnMut(usize, usize) -> Ordering) -> Vec<usize> {
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_unstable_by(|a, b| compare(*a, *b).then(a.cmp(b)));
    order
}

/// Sorts `items` by `compare`; equal elements keep their order.
pub(crate) fn by<T>(items: &mut [T], mut compare: impl FnMut(&T, &T) -> Ordering) {
    let order = ranked(items.len(), &mut |a, b| compare(&items[a], &items[b]));
    // Element `i` of the result is element `order[i]` of the original, moved along its cycle.
    let mut done = vec![false; items.len()];
    for start in 0..items.len() {
        if done[start] {
            continue;
        }
        let mut at = start;
        while order[at] != start {
            items.swap(at, order[at]);
            done[at] = true;
            at = order[at];
        }
        done[at] = true;
    }
}

/// Sorts `items` by the key `key` gives each; equal keys keep their order.
pub(crate) fn by_key<T, K: Ord>(items: &mut [T], mut key: impl FnMut(&T) -> K) {
    by(items, |a, b| key(a).cmp(&key(b)));
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_sorts_like_the_standard_stable_sort() {
        let mut state = 12_345_u64;
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            (state >> 33) as usize
        };
        for length in [0, 1, 2, 3, 10, 100, 500] {
            // Few distinct keys, so that ties are common and their order shows.
            let items: Vec<(usize, usize)> = (0..length).map(|index| (next() % 7, index)).collect();
            let mut expected = items.clone();
            expected.sort_by_key(|item| item.0);
            let mut actual = items.clone();
            super::by_key(&mut actual, |item| item.0);
            assert_eq!(actual, expected, "length {length}");
            let mut descending = items.clone();
            super::by(&mut descending, |a, b| b.0.cmp(&a.0));
            let mut expected = items;
            expected.sort_by_key(|item| std::cmp::Reverse(item.0));
            assert_eq!(descending, expected, "length {length}");
        }
    }
}
