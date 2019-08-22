//! Memoizer enables easy memoization of recursive user functions, based on keys that are hash or
//! Ord.
//!

#![deny(missing_docs)]
// TODO: Remove this
#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

// TODO: how can I remove the box on the iterator types?

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
        let memo_str = self.memo_predicate.as_ref().map(|_| "*present*").unwrap_or("*not present*");
        write!(f, "Memoizer {{ cache: {:?}, user_function: *unprintable*, memo_predicate: {} }}",
               self.cache, memo_str)
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
    /// By default, a key-value pair will be stored in the memoization cache for every distinct key
    /// passed to lookup.  This predicate allows the user to control which keys will be stored.
    /// This capability can be useful if there are a large number of cases that are simple enough
    /// that storing would be wasteful, or otherwise if the tradeoff between computation time and
    /// cache size calls for it.
    pub fn set_memo_predicate<F>(&mut self, memo: F)
    where
        F: 'static + Fn(&K) -> bool,
    {
        self.memo_predicate = Some(Box::new(memo));
    }
    /// Looks up a key in the cache, calculating a value if necessary.
    pub fn lookup(&mut self, k: &K) -> V {
        let cachev = self.cache.get(k).unwrap_or_else(|| {
            let save = self.memo_predicate.as_ref().map(|p| p(k)).unwrap_or(false);
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
