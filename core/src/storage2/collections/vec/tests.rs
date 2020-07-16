// Copyright 2019-2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::Vec as StorageVec;
use crate::{
    env,
    storage2::{
        collections::vec::IndexOutOfBounds,
        traits::{
            KeyPtr,
            SpreadLayout,
        },
    },
};
use ink_primitives::Key;

#[test]
fn new_vec_works() {
    // `StorageVec::new`
    let vec = <StorageVec<i32>>::new();
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.get(0), None);
    assert!(vec.iter().next().is_none());
    // `StorageVec::default`
    let default = <StorageVec<i32> as Default>::default();
    assert!(default.is_empty());
    assert_eq!(default.len(), 0);
    assert_eq!(vec.get(0), None);
    assert!(default.iter().next().is_none());
    // `StorageVec::new` and `StorageVec::default` should be equal.
    assert_eq!(vec, default);
}

#[test]
fn from_iterator_works() {
    let some_primes = [1, 2, 3, 5, 7, 11, 13];
    assert_eq!(some_primes.iter().copied().collect::<StorageVec<_>>(), {
        let mut vec = StorageVec::new();
        for prime in &some_primes {
            vec.push(*prime)
        }
        vec
    });
}

#[test]
fn from_empty_iterator_works() {
    assert_eq!(
        [].iter().copied().collect::<StorageVec<i32>>(),
        StorageVec::new(),
    );
}

#[test]
fn first_last_of_empty() {
    let mut vec = <StorageVec<u8>>::new();
    assert_eq!(vec.first(), None);
    assert_eq!(vec.first_mut(), None);
    assert_eq!(vec.last(), None);
    assert_eq!(vec.last_mut(), None);
}

#[test]
fn push_pop_first_last_works() {
    /// Asserts conditions are met for the given storage vector.
    fn assert_vec<F, L>(vec: &StorageVec<u8>, len: u32, first: F, last: L)
    where
        F: Into<Option<u8>>,
        L: Into<Option<u8>>,
    {
        assert_eq!(vec.is_empty(), len == 0);
        assert_eq!(vec.len(), len);
        assert_eq!(vec.first().copied(), first.into());
        assert_eq!(vec.last().copied(), last.into());
    }

    let mut vec = StorageVec::new();
    assert_vec(&vec, 0, None, None);

    // Sequence of `push`
    vec.push(b'a');
    assert_vec(&vec, 1, b'a', b'a');
    vec.push(b'b');
    assert_vec(&vec, 2, b'a', b'b');
    vec.push(b'c');
    assert_vec(&vec, 3, b'a', b'c');
    vec.push(b'd');
    assert_vec(&vec, 4, b'a', b'd');

    // Sequence of `pop`
    assert_eq!(vec.pop(), Some(b'd'));
    assert_vec(&vec, 3, b'a', b'c');
    assert_eq!(vec.pop(), Some(b'c'));
    assert_vec(&vec, 2, b'a', b'b');
    assert_eq!(vec.pop(), Some(b'b'));
    assert_vec(&vec, 1, b'a', b'a');
    assert_eq!(vec.pop(), Some(b'a'));
    assert_vec(&vec, 0, None, None);

    // Pop from empty vector.
    assert_eq!(vec.pop(), None);
    assert_vec(&vec, 0, None, None);
}

#[test]
fn pop_drop_works() {
    let elems = [b'a', b'b', b'c', b'd'];
    let mut vec = vec_from_slice(&elems);
    assert_eq!(vec.pop_drop(), Some(()));
    assert_eq_slice(&vec, &elems[0..3]);
    assert_eq!(vec.pop_drop(), Some(()));
    assert_eq_slice(&vec, &elems[0..2]);
    assert_eq!(vec.pop_drop(), Some(()));
    assert_eq_slice(&vec, &elems[0..1]);
    assert_eq!(vec.pop_drop(), Some(()));
    assert_eq_slice(&vec, &[]);
    assert_eq!(vec.pop_drop(), None);
    assert_eq_slice(&vec, &[]);
}

#[test]
fn get_works() {
    let elems = [b'a', b'b', b'c', b'd'];
    let mut vec = vec_from_slice(&elems);
    for (n, mut expected) in elems.iter().copied().enumerate() {
        let n = n as u32;
        assert_eq!(vec.get(n), Some(&expected));
        assert_eq!(vec.get_mut(n), Some(&mut expected));
        assert_eq!(&vec[n], &expected);
        assert_eq!(&mut vec[n], &mut expected);
    }
    let len = vec.len();
    assert_eq!(vec.get(len), None);
    assert_eq!(vec.get_mut(len), None);
}

#[test]
#[should_panic(expected = "index out of bounds: the len is 3 but the index is 3")]
fn index_out_of_bounds_works() {
    let test_values = [b'a', b'b', b'c'];
    let vec = vec_from_slice(&test_values);
    let _ = &vec[test_values.len() as u32];
}

#[test]
#[should_panic(expected = "index out of bounds: the len is 3 but the index is 3")]
fn index_mut_out_of_bounds_works() {
    let test_values = [b'a', b'b', b'c'];
    let mut vec = vec_from_slice(&test_values);
    let _ = &mut vec[test_values.len() as u32];
}

#[test]
fn iter_next_works() {
    let elems = [b'a', b'b', b'c', b'd'];
    let vec = vec_from_slice(&elems);
    // Test iterator over `&T`:
    let mut iter = vec.iter();
    assert_eq!(iter.count(), 4);
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some(&b'a'));
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some(&b'b'));
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.count(), 2);
    assert_eq!(iter.next(), Some(&b'c'));
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some(&b'd'));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.count(), 0);
    assert_eq!(iter.next(), None);
    // Test iterator over `&mut T`:
    let mut vec = vec;
    let mut iter = vec.iter_mut();
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some(&mut b'a'));
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some(&mut b'b'));
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.next(), Some(&mut b'c'));
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some(&mut b'd'));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.count(), 0);
}

#[test]
fn iter_nth_works() {
    let elems = [b'a', b'b', b'c', b'd'];
    let vec = vec_from_slice(&elems);
    // Test iterator over `&T`:
    let mut iter = vec.iter();
    assert_eq!(iter.count(), 4);
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.nth(1), Some(&b'b'));
    assert_eq!(iter.count(), 2);
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.nth(1), Some(&b'd'));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.count(), 0);
    assert_eq!(iter.nth(1), None);
    // Test iterator over `&mut T`:
    let mut vec = vec;
    let mut iter = vec.iter_mut();
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.nth(1), Some(&mut b'b'));
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.nth(1), Some(&mut b'd'));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.nth(1), None);
    assert_eq!(iter.count(), 0);
}

#[test]
fn iter_next_back_works() {
    let elems = [b'a', b'b', b'c', b'd'];
    let vec = vec_from_slice(&elems);
    // Test iterator over `&T`:
    let mut iter = vec.iter().rev();
    assert_eq!(iter.clone().count(), 4);
    assert_eq!(iter.next(), Some(&b'd'));
    assert_eq!(iter.next(), Some(&b'c'));
    assert_eq!(iter.clone().count(), 2);
    assert_eq!(iter.next(), Some(&b'b'));
    assert_eq!(iter.next(), Some(&b'a'));
    assert_eq!(iter.clone().count(), 0);
    assert_eq!(iter.next(), None);
    // Test iterator over `&mut T`:
    let mut vec = vec;
    let mut iter = vec.iter_mut().rev();
    assert_eq!(iter.next(), Some(&mut b'd'));
    assert_eq!(iter.next(), Some(&mut b'c'));
    assert_eq!(iter.next(), Some(&mut b'b'));
    assert_eq!(iter.next(), Some(&mut b'a'));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.count(), 0);
}

#[test]
fn iter_nth_back_works() {
    let elems = [b'a', b'b', b'c', b'd'];
    let vec = vec_from_slice(&elems);
    // Test iterator over `&T`:
    let mut iter = vec.iter().rev();
    assert_eq!(iter.clone().count(), 4);
    assert_eq!(iter.nth(1), Some(&b'c'));
    assert_eq!(iter.clone().count(), 2);
    assert_eq!(iter.nth(1), Some(&b'a'));
    assert_eq!(iter.clone().count(), 0);
    assert_eq!(iter.nth(1), None);
    // Test iterator over `&mut T`:
    let mut vec = vec;
    let mut iter = vec.iter_mut().rev();
    assert_eq!(iter.nth(1), Some(&mut b'c'));
    assert_eq!(iter.nth(1), Some(&mut b'a'));
    assert_eq!(iter.nth(1), None);
    assert_eq!(iter.count(), 0);
}

/// Asserts that the the given ordered storage vector elements are equal to the
/// ordered elements of the given slice.
fn assert_eq_slice(vec: &StorageVec<u8>, slice: &[u8]) {
    assert_eq!(vec.len() as usize, slice.len());
    assert!(vec.iter().zip(slice.iter()).all(|(lhs, rhs)| *lhs == *rhs))
}

/// Creates a storage vector from the given slice.
fn vec_from_slice(slice: &[u8]) -> StorageVec<u8> {
    slice.iter().copied().collect::<StorageVec<u8>>()
}

#[test]
fn swap_works() {
    let elems = [b'a', b'b', b'c', b'd'];
    let mut vec = vec_from_slice(&elems);

    // Swap at same position is a no-op.
    for index in 0..elems.len() as u32 {
        vec.swap(index, index);
        assert_eq_slice(&vec, &elems);
    }

    // Swap first and second
    vec.swap(0, 1);
    assert_eq_slice(&vec, &[b'b', b'a', b'c', b'd']);
    // Swap third and last
    vec.swap(2, 3);
    assert_eq_slice(&vec, &[b'b', b'a', b'd', b'c']);
    // Swap first and last
    vec.swap(0, 3);
    assert_eq_slice(&vec, &[b'c', b'a', b'd', b'b']);
}

#[test]
#[should_panic]
fn swap_one_invalid_index() {
    let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);
    vec.swap(0, vec.len());
}

#[test]
#[should_panic]
fn swap_both_invalid_indices() {
    let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);
    vec.swap(vec.len(), vec.len());
}

#[test]
fn swap_remove_works() {
    let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);

    // Swap remove first element.
    assert_eq!(vec.swap_remove(0), Some(b'a'));
    assert_eq_slice(&vec, &[b'd', b'b', b'c']);
    // Swap remove middle element.
    assert_eq!(vec.swap_remove(1), Some(b'b'));
    assert_eq_slice(&vec, &[b'd', b'c']);
    // Swap remove last element.
    assert_eq!(vec.swap_remove(1), Some(b'c'));
    assert_eq_slice(&vec, &[b'd']);
    // Swap remove only element.
    assert_eq!(vec.swap_remove(0), Some(b'd'));
    assert_eq_slice(&vec, &[]);
    // Swap remove from empty vector.
    assert_eq!(vec.swap_remove(0), None);
    assert_eq_slice(&vec, &[]);
}

#[test]
fn swap_remove_drop_works() {
    let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);

    // Swap remove first element.
    assert_eq!(vec.swap_remove_drop(0), Some(()));
    assert_eq_slice(&vec, &[b'd', b'b', b'c']);
    // Swap remove middle element.
    assert_eq!(vec.swap_remove_drop(1), Some(()));
    assert_eq_slice(&vec, &[b'd', b'c']);
    // Swap remove last element.
    assert_eq!(vec.swap_remove_drop(1), Some(()));
    assert_eq_slice(&vec, &[b'd']);
    // Swap remove only element.
    assert_eq!(vec.swap_remove_drop(0), Some(()));
    assert_eq_slice(&vec, &[]);
    // Swap remove from empty vector.
    assert_eq!(vec.swap_remove_drop(0), None);
    assert_eq_slice(&vec, &[]);
}

#[test]
fn spread_layout_push_pull_works() -> env::Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let vec1 = vec_from_slice(&[b'a', b'b', b'c', b'd']);
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&vec1, &mut KeyPtr::from(root_key));
        // Load the pushed storage vector into another instance and check that
        // both instances are equal:
        let vec2 =
            <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(vec1, vec2);
        Ok(())
    })
}

#[test]
#[should_panic(expected = "encountered empty storage cell")]
fn spread_layout_clear_works() {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let vec1 = vec_from_slice(&[b'a', b'b', b'c', b'd']);
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&vec1, &mut KeyPtr::from(root_key));
        // It has already been asserted that a valid instance can be pulled
        // from contract storage after a push to the same storage region.
        //
        // Now clear the associated storage from `vec1` and check whether
        // loading another instance from this storage will panic since the
        // vector's length property cannot read a value:
        SpreadLayout::clear_spread(&vec1, &mut KeyPtr::from(root_key));
        let _ =
            <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        Ok(())
    })
    .unwrap()
}

#[test]
fn set_works() {
    let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);
    let _ = vec.set(0, b'x').unwrap();
    let expected = vec_from_slice(&[b'x', b'b', b'c', b'd']);
    assert_eq!(vec, expected);
}

#[test]
fn set_fails_when_index_oob() {
    let mut vec = vec_from_slice(&[b'a']);
    let res = vec.set(1, b'x');
    assert_eq!(res, Err(IndexOutOfBounds));
}

#[test]
fn clear_works_on_filled_vec() {
    let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);
    vec.clear();
    assert!(vec.is_empty());
}

#[test]
fn clear_works_on_empty_vec() {
    let mut vec = vec_from_slice(&[]);
    vec.clear();
    assert!(vec.is_empty());
}