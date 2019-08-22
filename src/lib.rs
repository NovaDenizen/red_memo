
// TODO: Remove this
#![allow(dead_code)]

use std::collections::BTreeMap;


#[derive(Eq, Ord, PartialOrd, PartialEq, Debug, Clone, Copy)]
enum MemoVaal<V> {
    InProgress,
    Value(V),
}


trait MemoStruct<K,V: Clone> {
    fn new() -> Self;
    fn insert(&mut self, k: K, v: V) -> Result<(), V>;
    fn get(&self, k: &K) -> Option<V>;
    fn get_mut(&mut self, k: &K) -> Option<&mut V>;
}

impl<K,V: Clone> MemoStruct<K,V> for BTreeMap<K,V>
where
    K: Ord,
{
    fn new() -> Self {
        BTreeMap::new()
    }
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



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
