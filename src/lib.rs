//! red_memo is a simple, safe, pure rust library for memoization and dynamic programming.
//!
//! `Memoizer<K,V>` is the main cache type.  It can be initialized with an underlying
//! `std::collections::HashMap` with `new_hash()`, or with an underlying
//! `std::collections::BTreeMap` with `new_ord()`.
//!
//! Keys can be either ordered or hashed. The outer api is identical for both cases,
//!
//! The Debug trait is required for keys and values in order to make error messages intelligible.
//! 
//! The Clone trait is required tor keys in order to fulfill the expectations a user has for a
//! memoizing cache.
//!
//! The Clone trait is required for values.  If Memoizer were to return immutable references to
//! cached values, as is typically done, then the cache would have to be immutably borrowd while
//! new values were being calculated from old ones, and the cache would not be updated with the new
//! values.
//!
//! If a value type cannot be made to implement Clone, or if it would be excessively costly to make
//! copies, consider using `std::rc::Rc`.
//!
//! ```
//!
//! use red_memo::Memoizer;
//!
//! fn fibonacci(mem: &mut Memoizer<usize, usize>, k: &usize) -> usize
//! {
//!     let k = *k;
//!     if k < 2 {
//!         k
//!     } else {
//!         mem.lookup(&(k - 1)) + mem.lookup(&(k - 2))
//!     }
//! }
//!
//! fn main()
//! {
//!     let mut fib_cache = Memoizer::new_ord(fibonacci);
//!     // since usize implements Hash+Eq as well, this could instead be
//!     // let mut fib_cache = Memoizer::new_hash(fibonacci);
//!     println!("fibonacci(20) = {}", fib_cache.lookup(&20));
//!     assert_eq!(fib_cache.lookup(&40), 102334155);
//! }
//!
//! ```
//!
//!

#![deny(missing_docs)]

use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Eq, Ord, PartialOrd, PartialEq, Debug, Copy, Clone)]
enum MemoVal<V> {
    InProgress,
    Finished(V),
}

trait MemoStruct<'a, K: 'a + Clone + Debug, V: 'a + Clone + Debug>: Debug {
    fn insert(&mut self, k: K, v: V) -> Result<(), V>;
    fn get(&self, k: &K) -> Option<V>;
    // TODO: remove get_mut?
    fn get_mut(&mut self, k: &K) -> Option<&mut V>;
    // TODO: Add iter() and into_iter() implementations somehow.
    // TODO: Make it possible to manually initialize the cache.  public `store()`?
}

impl<'a, K: 'a + Clone + Debug, V: 'a + Clone + Debug> MemoStruct<'a, K, V> for HashMap<K, V>
where
    K: Hash + Eq,
{
    fn insert(&mut self, k: K, v: V) -> Result<(), V> {
        use std::collections::hash_map::Entry::*;
        match HashMap::entry(self, k) {
            Vacant(ve) => {
                ve.insert(v);
                Ok(())
            }
            Occupied(mut oe) => {
                let oldv = oe.insert(v);
                Err(oldv)
            }
        }
    }
    fn get(&self, k: &K) -> Option<V> {
        self.get(k).map(|vref| vref.clone())
    }
    fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        HashMap::get_mut(self, k)
    }
}

impl<'a, K: 'a + Clone + Debug, V: 'a + Clone + Debug> MemoStruct<'a, K, V> for BTreeMap<K, V>
where
    K: Ord,
{
    fn insert(&mut self, k: K, v: V) -> Result<(), V> {
        use std::collections::btree_map::Entry::*;
        match self.entry(k) {
            Vacant(ve) => {
                ve.insert(v);
                Ok(())
            }
            Occupied(mut oe) => {
                let oldv = oe.insert(v);
                Err(oldv)
            }
        }
    }
    fn get(&self, k: &K) -> Option<V> {
        BTreeMap::get(self, k).map(|vref| vref.clone())
    }
    fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        BTreeMap::get_mut(self, k)
    }
}

/// Memoization cache for a recursive user function
pub struct Memoizer<'a, K: 'a, V: 'a + Clone + Debug> {
    cache: Box<dyn 'a + MemoStruct<'a, K, MemoVal<V>>>,
    user_function: Rc<dyn Fn(&mut Memoizer<K, V>, &K) -> V>,
    memo_predicate: Option<Box<dyn Fn(&K) -> bool>>,
}

impl<'a, K: 'a + Clone + Debug, V: 'a + Clone + Debug> Debug for Memoizer<'a, K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let memo_str = self
            .memo_predicate
            .as_ref()
            .map(|_| "*present*")
            .unwrap_or("*not present*");
        write!(
            f,
            "Memoizer {{ cache: {:?}, user_function: *unprintable*, memo_predicate: {} }}",
            self.cache, memo_str
        )
    }
}

impl<'a, K: 'a + Clone + Debug, V: 'a + Clone + Debug> Memoizer<'a, K, V> {
    /// Creates a Memoizer based on HashMap.
    pub fn new_hash<F>(user: F) -> Self
    where
        K: Hash + Eq,
        F: 'static + Fn(&mut Memoizer<K, V>, &K) -> V,
    {
        let cache = Box::new(HashMap::new());
        let user_function = Rc::new(user);
        let memo_predicate = None;
        Memoizer {
            cache,
            user_function,
            memo_predicate,
        }
    }
    /// Creates a Memoizer based on a BTreeMap.
    pub fn new_ord<F>(user: F) -> Self
    where
        K: Ord,
        F: 'static + Fn(&mut Memoizer<K, V>, &K) -> V,
    {
        let cache = Box::new(BTreeMap::new());
        let user_function = Rc::new(user);
        let memo_predicate = None;
        Memoizer {
            cache,
            user_function,
            memo_predicate,
        }
    }
    /// Sets a memoization predicate for the Memoizer.
    ///
    /// When a `Memoizer` has a memoization predicate set, keys not matched by the predicate will
    /// be calculated as usual but not stored in the cache once calculated.
    ///
    /// When a memoization predicate is not set, it is as if a predicate that always returns `true`
    /// is used.  That is, all key-value pairs computed will be stored in the cache.
    ///
    /// If there is a class of keys for which directly computing their value takes the same effort
    /// as lookinng up a key and cloning a value, it makes sense to use a predicate to keep those
    /// keys out of the cache.
    pub fn set_memo_predicate<P>(&mut self, predicate: P)
    where
        P: 'static + Fn(&K) -> bool,
    {
        self.memo_predicate = Some(Box::new(predicate));
    }
    /// Looks up a key in the cache, calculating a value if necessary.
    ///
    /// # Panics
    ///
    /// This method will panic if a circular dependency is detected.
    ///
    /// Before the Memoizer starts calculating a value for a particular key, it places an
    /// "in-progress" marker in the cache for that key.  After that key's value is caculated, the
    /// "in-progress" marker is replaced with the finished value.  If an in-progress key is passed
    /// to `lookup()`, this indicates a circular dependency.
    ///
    pub fn lookup(&mut self, k: &K) -> V {
        let cachev = self.cache.get(k).unwrap_or_else(|| {
            let save = self.memo_predicate.as_ref().map(|p| p(k)).unwrap_or(true);
            if save {
                self.cache
                    .insert(k.clone(), MemoVal::InProgress)
                    .unwrap_or_else(|_| {
                        panic!("Did not expect to see a memo cacne entry for key {:?}", k)
                    });
            }
            let user = Rc::clone(&self.user_function);
            let v = (*user)(self, k);
            if save {
                self.cache
                    .get_mut(k)
                    .map(|vr| *vr = MemoVal::Finished(v.clone()));
            }
            MemoVal::Finished(v)
        });
        match cachev {
            MemoVal::InProgress => panic!("Memoizer: circular dependency on key {:?}", k),
            MemoVal::Finished(v) => v,
        }
    }

    /// Look up a key in the cache, but do not calculate it if it is not present.
    pub fn lookup_immut(&self, k: &K) -> Option<V> {
        self.cache.get(k).and_then(|mv| match mv {
            MemoVal::InProgress => None,
            MemoVal::Finished(v) => Some(v),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fibonacci(mem: &mut Memoizer<usize, usize>, k: &usize) -> usize
    {
        let k = *k;
        if k < 2 {
            k
        } else {
            mem.lookup(&(k - 1)) + mem.lookup(&(k - 2))
        }
    }

    #[test]
    fn fibs_ord() {
        let mut fib_cache = Memoizer::new_ord(fibonacci);
        assert_eq!(fib_cache.lookup(&0), 0);
        assert_eq!(fib_cache.lookup(&1), 1);
        assert_eq!(fib_cache.lookup(&2), 1);
        assert_eq!(fib_cache.lookup(&3), 2);
        assert_eq!(fib_cache.lookup(&20), 6765);
        assert_eq!(fib_cache.lookup(&30), 832040);
        assert_eq!(fib_cache.lookup(&40), 102334155);
    }
    #[test]
    fn fibs_hash() {
        let mut fib_cache = Memoizer::new_hash(fibonacci);
        assert_eq!(fib_cache.lookup(&0), 0);
        assert_eq!(fib_cache.lookup(&1), 1);
        assert_eq!(fib_cache.lookup(&2), 1);
        assert_eq!(fib_cache.lookup(&3), 2);
        assert_eq!(fib_cache.lookup(&20), 6765);
        assert_eq!(fib_cache.lookup(&30), 832040);
        assert_eq!(fib_cache.lookup(&40), 102334155);
    }
}
