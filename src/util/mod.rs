//! Different helpers & utilities

use std::collections::HashMap;
use std::collections::hash_state::HashState;
use std::collections::hash_map::Entry;
use std::hash::Hash;


pub use self::pretty::PrettyPrinter;
pub use self::io::read_file;


mod io;
mod pretty;
pub mod visit;


/// A helper that tries to insert a key/value into a hashmap and returns
/// an error if the key already exists.
pub trait TryInsert<K, V> {
    fn try_insert(&mut self, k: K, v: V) -> Result<(), ()>;
}

impl<K, V, S> TryInsert<K, V> for HashMap<K, V, S>
        where K: Eq + Hash, S: HashState {
    fn try_insert(&mut self, k: K, v: V) -> Result<(), ()> {
        match self.entry(k) {
            Entry::Occupied(_) => Err(()),
            Entry::Vacant(entry) => {
                entry.insert(v);
                Ok(())
            }
        }
    }
}