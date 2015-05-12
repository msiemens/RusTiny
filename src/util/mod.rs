//! Different helpers & utilities
pub use self::io::read_file;

mod io;


/*
// This cannot implemented without #[feature(std_misc)] as of rustc 1.0.0-beta
(9854143cb 2015-04-02).

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
*/