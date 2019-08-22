
// TODO: Remove this
#![allow(dead_code)]

use std::collections::{ BTreeMap, HashMap };
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

// TODO: how can I remove the box on the iterator types?


#[derive(Eq, Ord, PartialOrd, PartialEq, Debug, Clone, Copy)]
enum MemoVal<V> {
    InProgress,
    Value(V),
}


trait MemoStruct<'a, K: 'a, V: 'a + Clone>
{
    fn insert(&mut self, k: K, v: V) -> Result<(), V>;
    fn get(&self, k: &K) -> Option<V>;
    // TODO: remove get_mut?
    fn get_mut(&mut self, k: &K) -> Option<&mut V>;
    // TODO: Add iter() and into_iter() implementations somehow.
}

impl<'a, K: 'a, V: 'a + Clone> MemoStruct<'a, K, V> for HashMap<K,V>
where
    K: Hash + Eq,
{
    fn insert(&mut self, k: K, v: V) -> Result<(), V> {
        use std::collections::hash_map::Entry::*;
        match HashMap::entry(self, k) {
            Vacant(ve) => {
                ve.insert(v);
                Ok(())
            },
            Occupied(mut oe) => {
                let oldv = oe.insert(v);
                Err(oldv)
            },
        }
    }
    fn get(&self, k: &K) -> Option<V> {
        self.get(k).map(|vref| vref.clone())
    }
    fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        HashMap::get_mut(self, k)
    }
}

impl<'a, K: 'a, V: 'a + Clone> MemoStruct<'a,K,V> for BTreeMap<K,V>
where
    K: Ord,
{
    fn insert(&mut self, k: K, v: V) -> Result<(), V> {
        use std::collections::btree_map::Entry::*;
        match self.entry(k) {
            Vacant(ve) => {
                ve.insert(v);
                Ok(())
            },
            Occupied(mut oe) => {
                let oldv = oe.insert(v);
                Err(oldv)
            },
        }
    }
    fn get(&self, k: &K) -> Option<V>
    {
        BTreeMap::get(self, k).map(|vref| vref.clone())
    }
    fn get_mut(&mut self, k: &K) -> Option<&mut V>
    {
        BTreeMap::get_mut(self, k)
    }
}

pub struct Memoizer<'a, K: 'a, V: 'a + Clone> {
    cache: Box<dyn 'a + MemoStruct<'a,K,MemoVal<V>>>,
    user_function: Rc<dyn Fn(&mut Memoizer<K, V>, &K) -> V>,
    small_predicate: Option<Rc<dyn Fn(&K) -> bool>>,
}

impl<'a, K: 'a, V: 'a + Clone> Memoizer<'a, K, V> 
{
    pub fn new_hash<F>(user: F) -> Self 
    where
        K: Hash + Eq,
        F: 'static + Fn(&mut Memoizer<K,V>, &K) -> V,
    {
        let cache = Box::new(HashMap::new());
        let user_function = Rc::new(user);
        let small_predicate = None;
        Memoizer { cache, user_function, small_predicate }
    }
    pub fn new_ord<F>(user: F) -> Self 
    where
        K: Ord,
        F: 'static + Fn(&mut Memoizer<K,V>, &K) -> V,
    {
        let cache = Box::new(BTreeMap::new());
        let user_function = Rc::new(user);
        let small_predicate = None;
        Memoizer { cache, user_function, small_predicate }
    }
    pub fn set_small_predicate<F>(&mut self, small: F) 
    where
        F: 'static + Fn(&K) -> bool,
    {
        self.small_predicate = Some(Rc::new(small));
    }
}





#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
