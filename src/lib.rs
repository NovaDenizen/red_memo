
// TODO: Remove this
#![allow(dead_code)]

use std::collections::{ BTreeMap, HashMap };
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

// TODO: how can I remove the box on the iterator types?


#[derive(Eq, Ord, PartialOrd, PartialEq, Debug, Clone, Copy)]
enum MemoVaal<V> {
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
    cache: Box<dyn MemoStruct<'a,K,V>>,
    user_function: Rc<dyn Fn(&mut Memoizer<K, V>, &K) -> V>,
    noisy: bool,
    key_to_string: Option<Box<dyn Fn(&K) -> String>>,
    value_to_string: Option<Box<dyn Fn(&V) -> String>>,
}





#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
