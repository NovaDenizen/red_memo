//! Memoizer enables easy memoization of recursive user functions, based on either hashable or
//! ordered keys.
//!
//! ```
//!
//! use newmemo::*;
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
//!     println!("fibonacci(20) = {}", fib_cache.lookup(&20));
//!     assert_eq!(fib_cache.lookup(&40), 102334155);
//! }
//!
//! ```
//!
//!

#![deny(missing_docs)]
// TODO: Remove this
#![allow(dead_code)]

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
