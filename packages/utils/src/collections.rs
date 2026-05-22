pub type HashMap<K, V> = std::collections::HashMap<K, V>;
// #[cfg(not(feature = "test-support"))]
pub type HashSet<T> = std::collections::HashSet<T>;

pub type IndexMap<K, V> = indexmap::IndexMap<K, V>;

pub type IndexSet<T> = indexmap::IndexSet<T>;

pub use rustc_hash::FxHasher;
pub use rustc_hash::{FxHashMap, FxHashSet};
pub use std::collections::*;
